use rusty_vm_lib::assembly::ByteCode;

use crate::{assembler, error};
use crate::lang::{ArrayData, AsmInstructionNode, AsmNode, AsmNodeValue, AsmOperand, AsmValue, DataType, FunctionMacroDef, InlineMacroDef, Number, NumberSize, PrimitiveData, PseudoInstructionNode, PseudoInstructions, INCLUDE_SECTION_NAME};
use crate::tokenizer::{SourceToken, Token, TokenLines, TokenList, TokenValue};
use crate::symbol_table::SymbolTable;
use crate::module_manager::ModuleManager;

use std::collections::{HashMap, VecDeque};
use std::mem;
use std::path::Path;
use std::rc::Rc;


/// Recursively expand inline macros in the given token line
fn expand_inline_macros<'a>(tokens: &mut TokenList<'a>, symbol_table: &SymbolTable<'a>, module_manager: &ModuleManager<'a>) {

    let mut i: usize = 0;

    while let Some(token) = tokens.get(i) {

        if matches!(token.value, TokenValue::Equals) {

            let macro_name = tokens.get(i+1).unwrap_or_else(
                || error::parsing_error(&token.source, module_manager, "Missing macro name after `=`")
            );

            let TokenValue::Identifier(name) = macro_name.value else {
                error::parsing_error(&macro_name.source, module_manager, "Expected a macro name after `=`")
            };

            let macro_def = symbol_table.get_inline_macro(name).unwrap_or_else(
                || error::undefined_macro(&macro_name.source, module_manager, symbol_table.inline_macros())
            );

            // Split the line in two
            let mut after = tokens.split_off(i);
            // Remove the macro call
            after.drain(0..2);
            // Extend the line with the expanded macro
            tokens.extend(macro_def.def.iter().cloned());
            // Stitch the line back together
            tokens.extend(after);

            /*
                Don't increment the index because we should check the expanded macro for other macros.
                A situation like this may occur:

                    %- FOO: r1
                    %- BAR: =FOO 90
                    mov8 =BAR
                
                Which should expand to

                    mov8 r1 90
            */
            continue;
        }

        i += 1;
    }
}


fn parse_operands<'a>(mut tokens: TokenList<'a>, module_manager: &ModuleManager<'a>) -> Box<[AsmOperand<'a>]> {

    let mut operands = Vec::with_capacity(tokens.len());

    // The parsing occurs on a left-to-right manner. In assembly, statements are not complex enough to require operator priorities.

    while let Some(token) = tokens.pop_front() {


        macro_rules! push_op {
            ($op:expr, $source:expr) => {
                operands.push(AsmOperand { value: $op, source: $source.source.clone() })
            }
        }

        match token.value {
            
            TokenValue::Number(n) => push_op!(AsmValue::Number(n.clone()), token),
            
            TokenValue::Identifier(id) => push_op!(AsmValue::Label(id), token),
            
            TokenValue::Register(reg) => push_op!(AsmValue::Register(reg), token),

            TokenValue::SquareOpen => {

                let addr_operand = tokens.pop_front().unwrap_or_else(
                    || error::parsing_error(&token.source, module_manager, "Missing operand after `[`."));

                let closing_square = tokens.pop_front().unwrap_or_else(
                    || error::parsing_error(&token.source, module_manager, "Missing closing `]`."));
                
                if !matches!(closing_square.value, TokenValue::SquareClose) {
                    error::parsing_error(&closing_square.source, module_manager, "Expected a closing `]`.");
                }

                match addr_operand.value {
                    
                    TokenValue::Register(reg) => push_op!(AsmValue::AddressInRegister(reg), token),

                    TokenValue::Number(n) => push_op!(AsmValue::AddressLiteral(n), token),

                    TokenValue::Identifier(name) => push_op!(AsmValue::AddressAtLabel(name), token),
                    
                    TokenValue::LabelDef { .. } |
                    TokenValue::FunctionMacroDef { .. } |
                    TokenValue::InlineMacroDef { .. } |
                    TokenValue::Endmacro |
                    TokenValue::Dot |
                    TokenValue::Comma |
                    TokenValue::SquareOpen |
                    TokenValue::SquareClose |
                    TokenValue::CurlyOpen |
                    TokenValue::CurlyClose |
                    TokenValue::StringLiteral(_) |
                    TokenValue::Instruction(_) |
                    TokenValue::PseudoInstruction(_) |
                    TokenValue::Colon 
                        => error::parsing_error(&addr_operand.source, module_manager, "Cannot address this token"),

                    TokenValue::Bang |
                    TokenValue::Equals 
                        => unreachable!("Macros are expanded before the main parsing")
                }
            },
            
            TokenValue::Instruction(_) |
            TokenValue::PseudoInstruction(_) |
            TokenValue::Dot |
            TokenValue::Endmacro |
            TokenValue::CurlyClose |
            TokenValue::LabelDef { .. } |
            TokenValue::SquareClose |
            TokenValue::FunctionMacroDef { .. } |
            TokenValue::InlineMacroDef { .. } |
            TokenValue::Comma |
            TokenValue::StringLiteral(_) |
            TokenValue::Colon
                => error::parsing_error(&token.source, module_manager, "Token cannot be used as here."),
            
            TokenValue::Bang |
            TokenValue::Equals |
            TokenValue::CurlyOpen
                => unreachable!("Macros are expanded before the main parsing"),

        }

    }

    operands.into_boxed_slice()
}



fn parse_line<'a>(main_operator: Token<'a>, operands: Box<[AsmOperand<'a>]>, nodes: &mut Vec<AsmNode<'a>>, module_manager: &ModuleManager<'a>, symbol_table: &SymbolTable<'a>) {

    match main_operator.value {

        TokenValue::LabelDef { export } => {

            if operands.len() != 1 {
                error::parsing_error(&main_operator.source, module_manager, format!("Label declaration expects exactly one identifier argument, but {} were given.", operands.len()).as_str())
            }

            let label_op = &operands[0];

            let AsmValue::Label(label) = label_op.value else {
                error::parsing_error(&label_op.source, module_manager, format!("Operator `{}` expects an identifier argument", main_operator.source.string).as_str())
            };
            
            nodes.push(AsmNode {
                source: Rc::clone(&main_operator.source),
                value: AsmNodeValue::Label(label)
            });

            symbol_table.declare_label(
                label, 
                Rc::clone(&label_op.source), 
                export
            )
            .err().map(
                |old_def| error::symbol_redeclaration(&old_def.source, &label_op.source, module_manager, "Label already declared")
            );
        },

        TokenValue::Instruction(instruction) => {

            let node = match AsmInstructionNode::build(instruction, operands) {

                Ok(node) => node,

                Err((faulty_token, hint))
                    => error::parsing_error(&faulty_token.unwrap_or(main_operator.source), module_manager, &hint)
            };
            
            nodes.push(AsmNode {
                source: main_operator.source,
                value: AsmNodeValue::Instruction(node)
            });
        },

        TokenValue::CurlyOpen |
        TokenValue::Endmacro |
        TokenValue::Number(_) |
        TokenValue::SquareOpen |
        TokenValue::SquareClose |
        TokenValue::Comma |
        TokenValue::CurlyClose |
        TokenValue::Register(_) |
        TokenValue::Identifier(_) |
        TokenValue::StringLiteral(_) |
        TokenValue::Colon
            => error::parsing_error(&main_operator.source, module_manager, "Token cannot be used as a main operator"),
        
        TokenValue::Dot |
        TokenValue::PseudoInstruction(_) |
        TokenValue::InlineMacroDef { .. } |
        TokenValue::FunctionMacroDef { .. } |
        TokenValue::Equals |
        TokenValue::Bang
            => unreachable!("Should have been handled before as special cases") 
    }
    
}


pub fn parse<'a>(mut token_lines: TokenLines<'a>, symbol_table: &SymbolTable<'a>, module_manager: &'a ModuleManager<'a>, bytecode: &mut ByteCode) -> Box<[AsmNode<'a>]> {

    // A good estimate for the number of nodes is the number of assembly lines. This is because an assembly line 
    // usually translates to a single instruction. This should avoid reallocations in most cases.
    let mut nodes = Vec::with_capacity(token_lines.len());

    while let Some(mut line) = token_lines.pop_front() {

        /*
            Perform a pre-pass to expand eventual inline macros.
            This is done here to avoid implementing a priority-based tree parser to evaluate macros
            inside delimiters like this:

                %- CONST_ADDRESS: 0xFF89178A
                mov8 [=CONST_ADDRESS] r1

            Performing a linear search for `=` tokens is faster than using a tree parser, which would have
            to search each time for the highest priority token inside a linked list.
            Also, since assembly lines are generally pretty short, shifting the elements when expanding the macro
            should not be too costly. This would be totally different in a more complex programming language.
        */
        expand_inline_macros(&mut line, symbol_table, module_manager);

        if let Some(main_operator) = line.pop_front() {

            // Macros are handled separately because they operate on Tokens, not AsmOperands

            match main_operator.value {

                TokenValue::InlineMacroDef { export } => {
                    parse_inline_macro_def(&mut line, main_operator.source, module_manager, symbol_table, export);
                },

                TokenValue::FunctionMacroDef { export } => {
                    parse_function_macro_def(&mut line, main_operator.source, module_manager, &mut token_lines, symbol_table, export);
                },

                TokenValue::Bang => {
                    expand_function_macro(&mut line, main_operator.source, &mut token_lines, module_manager, symbol_table);
                },

                TokenValue::Dot => {
                    /*
                        Handle ASM sections separately.
                        The .include section is special because it only contains strings
                        Other dedicated sections may be implemented
                        All other sections are treated as labels
                    */
                    parse_section(&mut nodes, &mut line, main_operator.source, module_manager, &mut token_lines, symbol_table, bytecode);
                },

                TokenValue::PseudoInstruction(instruction) => {
                    /*
                        Handle pseudo instructions separately.
                        Some pseudo instructions may work with a non-assembly-like syntax,
                        which would be a waste to include in the generic operand parser since it's only used 
                        with pseudo instructions
                    */
                    parse_pseudo_instruction(instruction, main_operator.source, &mut line, &mut nodes, module_manager);
                },

                _ => {
                    let operands = parse_operands(line, module_manager);

                    parse_line(main_operator, operands, &mut nodes, module_manager, symbol_table);
                }
            }
        }
    }

    nodes.into_boxed_slice()
}


macro_rules! declare_parsing_utils {
    
    ($module_manager:ident, $line:ident, $main_op:ident) => {

        macro_rules! pop_next {

            (let $token_symbol:ident => $missing_err:expr, let $required_pat:pat => $wrong_type_err:expr) => {
    
                pop_next!(
                    let $token_symbol => $missing_err
                );
                
                let $required_pat = $token_symbol.value else {
                    error::parsing_error(&$token_symbol.source, $module_manager, $wrong_type_err);
                };
    
            };
    
            (let $token_symbol:ident => $missing_err:expr) => {
    
                let $token_symbol = $line.pop_front().unwrap_or_else(
                    || error::parsing_error(&$main_op, $module_manager, $missing_err)
                );
            };
        }

        #[allow(unused_macros)]
        macro_rules! assert_empty_line {
            () => {
                if !$line.is_empty() {
                    error::parsing_error(&$line[0].source, $module_manager, "Unexpected token")
                }
            };
        }
    };
}


fn parse_pseudo_instruction<'a>(instruction: PseudoInstructions, main_op: Rc<SourceToken<'a>>, line: &mut TokenList<'a>, nodes: &mut Vec<AsmNode<'a>>, module_manager: &'a ModuleManager<'a>) {

    declare_parsing_utils!(module_manager, line, main_op);

    macro_rules! push_pseudo {
        ($pi:expr) => {
            nodes.push(AsmNode {
                source: main_op,
                value: AsmNodeValue::PseudoInstruction (
                    $pi
                )
            });
        };
    }

    match instruction {

        PseudoInstructions::DefineNumber => {
            // dn <size> <number>

            pop_next!(
                let size_token => "Missing number size specifier in static data declaration",
                let TokenValue::Number(Number::UnsignedInt(size)) => "Expected an unsigned integer as size specifier in static data declaration"
            );

            let Some(size) = NumberSize::new(size) else {
                error::parsing_error(&size_token.source, module_manager, "Invalid number size. The specified number size is expected to be an unsigned integer value among 1, 2, 4, 8");
            };

            pop_next!(
                let number_token => "Missing numeric value in static data declaration",
                let TokenValue::Number(number) => "Expected a numeric value in static data declaration"
            );

            assert_empty_line!();

            // The number size will be checked later, at code generation

            push_pseudo!(PseudoInstructionNode::DefineNumber { 
                size: (size, size_token.source), 
                number: (number, number_token.source) 
            });
        },

        PseudoInstructions::DefineString => {
            // ds <string>

            pop_next!(
                let string_token => "Missing string data in static data declaration",
                let TokenValue::StringLiteral(string) => "Expected a string literal in static data declaration"
            );

            assert_empty_line!();

            push_pseudo!(PseudoInstructionNode::DefineString {
                string: (string, string_token.source)
            });
        },

        PseudoInstructions::DefineBytes => {
            // db <byte array>

            pop_next!(
                let open_square_token => "Missing byte array in static data declaration",
                let TokenValue::SquareOpen => "Expected a byte array in static data declaration"
            );

            /*
                The syntax of a byte array is as follows:
                `
                    [43 54 0 1]
                `
                The -1 is because we don't count the closing square bracket in the
                number of array elements. 
                Note that the opening square bracket has already been popped.
            */
            let mut bytes: Vec<u8> = Vec::with_capacity(line.len() - 1);

            loop {

                pop_next!(
                    let token => "Unterminated byte array in static data declaration"
                );

                match token.value {
                    
                    TokenValue::Number(number) => {

                        let Number::UnsignedInt(value) = number else {
                            error::parsing_error(&token.source, module_manager, "Expected an unsigned integer as element of a byte array")
                        };

                        if value > u8::MAX as u64 {
                            error::invalid_number_size(&token.source, module_manager, number.least_bytes_repr(), mem::size_of::<u8>());
                        }

                        bytes.push(value as u8);
                    },

                    TokenValue::SquareClose
                        => break,

                    _ => error::parsing_error(&token.source, module_manager, "Unexpected token in byte array")
                }
            }

            assert_empty_line!();

            push_pseudo!(PseudoInstructionNode::DefineBytes {
                bytes: (bytes.into_boxed_slice(), open_square_token.source)
            });
        },

        PseudoInstructions::OffsetFrom => {
            // offsetfrom <label>
            
            pop_next!(
                let label_token => "Missing label name",
                let TokenValue::Identifier(label) => "Expected an identifier as label name"
            );

            assert_empty_line!();

            push_pseudo!(PseudoInstructionNode::OffsetFrom {
                label: (label, label_token.source)
            });
        },

        PseudoInstructions::DefineArray => {
            /*
                da <element type> <array>

                da u8 [1,2,3,4]
                da i32 [-3, 89, 1000, -1]
                da [i32: 2] [ [1,2], [3,4], [5,6], [7,8] ]
            */

            let element_type = parse_data_type(&main_op, line, module_manager);
            
            let array = parse_array_literal(element_type.0, &main_op, None, None, line, module_manager);

            assert_empty_line!();

            push_pseudo!(PseudoInstructionNode::DefineArray {
                array
            });
        },
        
        PseudoInstructions::PrintString => {
            /*
                printstr <string literal>
            */

            pop_next!(
                let string_token => "Missing string data",
                let TokenValue::StringLiteral(string) => "Expected a string literal"
            );

            assert_empty_line!();

            push_pseudo!(PseudoInstructionNode::PrintString {
                string: (string, string_token.source)
            });
        }

    }

}


fn parse_array_literal<'a>(element_type: DataType, main_op: &SourceToken<'a>, opening_delimiter: Option<Rc<SourceToken<'a>>>, expected_len: Option<usize>, line: &mut TokenList<'a>, module_manager: &'a ModuleManager<'a>) -> (ArrayData, Rc<SourceToken<'a>>) {
    declare_parsing_utils!(module_manager, line, main_op);

    let opening_delimiter = {
        if let Some(opening_delimiter) = opening_delimiter {
            opening_delimiter
        } else {
            pop_next!(
                let open_square => "Missing array literal",
                let TokenValue::SquareOpen => "Expected opening `[`"
            );
            open_square.source
        }
    };

    let mut array_elements = Vec::with_capacity(expected_len.unwrap_or_default());

    let mut needs_comma = false;
    
    loop {

        pop_next!(
            let token => "Missing literal value"
        );

        match token.value {

            TokenValue::SquareClose => break,

            TokenValue::SquareOpen => {

                if needs_comma {
                    error::parsing_error(&token.source, module_manager, "Expected a comma");
                }

                let DataType::Array { element_type: ref inner_elem_type, len } = element_type else {
                    error::parsing_error(&token.source, module_manager, format!("Expected element of type {element_type}, got an array").as_str())
                };
                
                let inner_array = parse_array_literal(*inner_elem_type.clone(), main_op, Some(token.source), Some(len), line, module_manager);

                array_elements.push(PrimitiveData::Array(inner_array.0));

                needs_comma = true;
            },

            TokenValue::Number(n) => {

                if needs_comma {
                    error::parsing_error(&token.source, module_manager, "Expected a comma");
                }

                // Check if the number is of the correct type
                // This ugly approach is fine, a good conversion table would be a bit overkill
                match element_type {

                    DataType::Int { size } => {

                        match n {
                            Number::SignedInt(_) => (),

                            Number::UnsignedInt(u) => {
                                if u > i64::MAX as u64 {
                                    error::parsing_error(&token.source, module_manager, format!("Integr too large to fit into a {element_type} type").as_str())
                                }
                            },

                            Number::Float(_)
                                => error::parsing_error(&token.source, module_manager, format!("Expected element of type {element_type}, got a float").as_str())
                        }

                        if n.least_bytes_repr() > size.as_usize() {
                            error::invalid_number_size(&token.source, module_manager, n.least_bytes_repr(), size.as_usize());
                        }
                    },

                    DataType::Uint { size } => {

                        if !matches!(n, Number::UnsignedInt(_)) {
                            error::parsing_error(&token.source, module_manager, format!("Expected element of type {element_type}").as_str()); // Kinda lazy error message, though
                        }

                        if n.least_bytes_repr() > size.as_usize() {
                            error::invalid_number_size(&token.source, module_manager, n.least_bytes_repr(), size.as_usize());
                        }
                    },

                    DataType::Float { size } => {

                        if !matches!(n, Number::Float(_)) {
                            error::parsing_error(&token.source, module_manager, format!("Expected element of type {element_type}").as_str()); // Kinda lazy error message, though
                        }

                        if n.least_bytes_repr() > size.as_usize() {
                            error::invalid_number_size(&token.source, module_manager, n.least_bytes_repr(), size.as_usize());
                        }
                    },

                    DataType::Array { .. }
                        => error::parsing_error(&token.source, module_manager, format!("Expected element of type {element_type}, got an array").as_str())
                }

                array_elements.push(PrimitiveData::Number(n));

                needs_comma = true;
            },

            TokenValue::Comma => {
                if !needs_comma {
                    error::parsing_error(&token.source, module_manager, "Unexpected comma");
                }
                needs_comma = false;
            },

            _ => error::parsing_error(&token.source, module_manager, "Unexpected token in array literal declaration")
        }

    }

    (
        ArrayData {
            array: array_elements.into_boxed_slice(),
            element_type
        },
        opening_delimiter
    )
}


fn parse_data_type<'a>(main_op: &SourceToken<'a>, line: &mut TokenList<'a>, module_manager: &'a ModuleManager<'a>) -> (DataType, Rc<SourceToken<'a>>) {
    declare_parsing_utils!(module_manager, line, main_op);

    pop_next!(
        let dt_token => "Missing data dype"
    );
    
    let data_type = match dt_token.value {
    
        TokenValue::Identifier(name) => {
            DataType::from_name_not_array(name).unwrap_or_else(
                || error::parsing_error(&dt_token.source, module_manager, "Invalid data type name")
            )
        },

        TokenValue::SquareOpen => {
            // This is an array

            let element_type = parse_data_type(main_op, line, module_manager);

            pop_next!(
                let colon_token => "Missing colon",
                let TokenValue::Colon => "Expected a colon to specify the array length"
            );

            pop_next!(
                let len_token => "Missing array length",
                let TokenValue::Number(Number::UnsignedInt(len)) => "Expected an unsigned integer as array length specifier"
            );

            pop_next!(
                let closing_square => "Missing closing `]`",
                let TokenValue::SquareClose => "Expected closing `]`"
            );

            DataType::Array {
                element_type: Box::new(element_type.0),
                len: len as usize
            }
        },

        _ => error::parsing_error(&dt_token.source, module_manager, "Invalid data dype")
    };

    (
        data_type,
        dt_token.source
    )
}


fn parse_section<'a>(nodes: &mut Vec<AsmNode<'a>>, line: &mut TokenList<'a>, main_op: Rc<SourceToken<'a>>, module_manager: &'a ModuleManager<'a>, token_lines: &mut VecDeque<VecDeque<Token<'a>>>, symbol_table: &SymbolTable<'a>, bytecode: &mut ByteCode) {
    
    declare_parsing_utils!(module_manager, line, main_op);

    pop_next!(
        let name_token => "Missing section name",
        let TokenValue::Identifier(name) => "Expected an identifier as section name"
    );

    pop_next!(
        let colon_token => "Expected a trailing colon after section name in section delcaration.",
        let TokenValue::Colon => "Expected a trailing colon after section name in section delcaration."
    );
    
    assert_empty_line!();

    // Declare the section label so that it can be used. Section labels are not exportable.
    symbol_table.declare_label(
        name, 
        Rc::clone(&name_token.source),
        false
    )
    .err().map(
        |old_def| error::symbol_redeclaration(&old_def.source, &name_token.source, module_manager, "Label already declared")
    );

    nodes.push(AsmNode {
        source: Rc::clone(&main_op),
        value: AsmNodeValue::Label(name)
    });

    if name == INCLUDE_SECTION_NAME {

        // Each line in the .include section is a string path to include
        while let Some(mut include_line) = token_lines.pop_front() {
            
            // Assume the line cannot be empty because macros aren't expanded in the .include section and empty lines are discarded by the tokenizer.
            if let Token { value: TokenValue::Dot, .. } = include_line.front().unwrap() {
                /*
                    Start of another section.
                    This self-call does not cause the parser to recursively parse the whole
                    ASM unit because this function calls itself only if the current section is the include section.
                    Note that there cannot be multiple include sections in the same ASM unit.
                */
                let Token { source, .. } = include_line.pop_front().unwrap(); // To satisfy the borrow checker
                parse_section(nodes, &mut include_line, source, module_manager, token_lines, symbol_table, bytecode);

                break;
            }

            let to_re_export = if let Some(token) = include_line.front() {
                // `@@` in front of the include path
                if matches!(token.value, TokenValue::LabelDef { export: true }) { 
                    include_line.pop_front();
                    true
                } else {
                    false
                }
            } else {
                error::parsing_error(&include_line[0].source, module_manager, "Expected an include path string");
            };

            if include_line.len() != 1 {
                error::parsing_error(&include_line[0].source, module_manager, "Expected a single include path string");
            }

            let include_path = if let TokenValue::StringLiteral(s) = &mut include_line[0].value {
                Path::new(
                    /*
                        Leak is acceptable beacause the include path will remain in use until the module manager is dropped, and that is at the end of the program.
                        We cannot just take a reference to the include path because it's a Cow<str> and not a &str.
                        In case the string contained special escape characters, the escaped string would be owned by
                        the token, which will be dropped after parsing.
                    */
                    Box::new(mem::take(s).into_owned())
                        .leak()
                )
            } else {
                error::parsing_error(&include_line[0].source, module_manager, "Expected a string literal as include path")
            };

            // Shadow the previous `include_path` to avoid confusion with the variables
            let include_path = module_manager.resolve_include_path(
                main_op.unit_path.as_path().parent().unwrap_or(Path::new("")),
                include_path
            )
            .unwrap_or_else(|err| 
                error::io_error(err, Some(main_op.unit_path), format!("Failed to resolve path \"{}\"", include_path.display()).as_str())
            );

            assembler::assemble_included_unit(include_path, module_manager, bytecode);

            let exports = module_manager.get_unit_exports(include_path);

            symbol_table.import_symbols(exports, to_re_export, module_manager);
        }

    }
}


type MacroArgGroup<'a> = Box<[*const Token<'a>]>;


fn group_macro_args<'a>(args: &TokenList<'a>) -> Box<[MacroArgGroup<'a>]> {

    // Set initial capacity to avoid reallocations.
    // If the line contains only one-token-groups, `args.len()` is perfect.
    // If the line contains one or more mutli-token-groups, `args.len()` allocates slightly more than necessary.
    let mut groups: Vec<MacroArgGroup> = Vec::with_capacity(args.len());

    let mut iter = args.iter();

    while let Some(token) = iter.next() {

        match token.value {
            
            TokenValue::SquareOpen => {
                /*
                    If the square brackets enclose some tokens, handle the whole brackets and content as a single arg group:
                    `
                        !println_int [ADDR]       
                    `
                    In other cases, push each token as a separate arg group:
                    `
                        !println_int [42 92
                        !println_int [
                        !println_int [[
                    `
                    The latter examples are weird and don't reflect any real-world application, but it's syntactically allowed.
                    It's up to the programmer to provide the correct macro arguments
                */

                let mut group = vec![token as *const Token];

                while let Some(inner) = iter.next() {

                    group.push(inner);

                    if matches!(inner.value, TokenValue::SquareClose) {
                        break;
                    }
                }

                if let TokenValue::SquareClose = unsafe { 
                                                    &**group.last()
                                                        .expect("Always has at least one element (the opening `[` token)")
                                                }.value
                {
                    // The arg group is a token list enclosed in square brackets
                    groups.push(MacroArgGroup::from(group));
                } else {
                    /*
                        Note that pushing the so-far-seen tokens this way is not valid if tokens other than `[` are
                        handled as special tokens because they may lead a token group.
                    */
                    groups.extend(
                        group.iter().map(
                            |token| MacroArgGroup::from([*token])
                        )
                    );
                }
            },
            
            // Inline macros are expanded before the operators are evaluated
            TokenValue::Equals |
            _ => groups.push(MacroArgGroup::from([token as *const Token]))

        }

    }

    groups.into_boxed_slice()
}


fn expand_function_macro<'a>(line: &mut TokenList<'a>, main_op: Rc<SourceToken<'a>>, token_lines: &mut TokenLines<'a>, module_manager: &ModuleManager<'a>, symbol_table: &SymbolTable<'a>) {

    declare_parsing_utils!(module_manager, line, main_op);

    pop_next!(
        let macro_name_op => "Missing macro name after `!`",
        let TokenValue::Identifier(macro_name) => "Expected an identifier as macro name after `!`"
    );

    let macro_def = symbol_table.get_function_macro(macro_name).unwrap_or_else(
        || error::undefined_macro(&macro_name_op.source, module_manager, symbol_table.function_macros())
    );

    let args = group_macro_args(&line);

    if args.len() != macro_def.params.len() {
        error::parsing_error(&main_op, module_manager, format!("Mismatched argument count in macro call: Expected {}, got {}", macro_def.params.len(), args.len()).as_str())
    }

    let expanded_macro = macro_def.body.iter().map(
        |original_line| {

            // Setting the starting capacity to the original line's capacity avoids reallocations if the line doesn't contain macro arguments
            let mut expanded_line = TokenList::with_capacity(original_line.len());

            let mut i = 0;
            
            while let Some(token) = original_line.get(i) {
                
                // Start of a macro parameter. Syntax: `{param}`
                if matches!(token.value, TokenValue::CurlyOpen) {

                    i += 1;
                    let param_name_token = original_line.get(i).unwrap_or_else(
                        || error::parsing_error(&token.source, module_manager, "Missing macro parameter name after `{` inside macro body")
                    );
                    let TokenValue::Identifier(param_name) = param_name_token.value else {
                        error::parsing_error(&param_name_token.source, module_manager, "Expected a macro parameter name after '{` inside macro body");
                    };

                    let arg_position = *macro_def.params.get(param_name).unwrap_or_else(
                        || error::parsing_error(&param_name_token.source, module_manager, "Undefined macro parameter name")
                    );

                    i += 1;
                    let closing_curly_token = original_line.get(i).unwrap_or_else(
                        || error::parsing_error(&token.source, module_manager, "Missing closing `}` after macro parameter name inside macro body")
                    );
                    if !matches!(closing_curly_token.value, TokenValue::CurlyClose) {
                        error::parsing_error(&closing_curly_token.source, module_manager, "Expected closing `}` after macro parameter name inside macro body");
                    }

                    expanded_line.extend(
                        args[arg_position].iter().map(|&token| unsafe { &*token }.clone() )
                    );

                    i += 1;
                    continue;
                }

                // TODO: devise a more efficient way to expand a macro
                expanded_line.push_back(token.clone());
                i += 1;
            }

            expanded_line
        }
    );

    // Push the expanded macro in reverse order to preserve line order.
    for line in expanded_macro.rev() {
        token_lines.push_front(line);
    }
}


fn parse_function_macro_def<'a>(line: &mut TokenList<'a>, main_op: Rc<SourceToken<'a>>, module_manager: &ModuleManager<'a>, token_lines: &mut VecDeque<VecDeque<Token<'a>>>, symbol_table: &SymbolTable<'a>, export: bool) {
    
    declare_parsing_utils!(module_manager, line, main_op);

    pop_next!(
        let name_token => "Expected a macro name",
        let TokenValue::Identifier(name) => "Expected an identifier as macro name"
    );

    // The rest of the line is the macro parameters, except for the trailing semicolon

    let mut params = HashMap::new();

    for (index, param_token) in line.drain(..line.len()-1).enumerate() {

        let TokenValue::Identifier(param_name) = param_token.value else {
            error::parsing_error(&param_token.source, module_manager, "Expected an identifier as macro parameter");
        };

        if let Some(omonym_index) = params.insert(param_name, index) {
            error::parsing_error(&param_token.source, module_manager, format!("Duplicated parameter name in macro definition. The parameter is mentioned before in position {}", omonym_index+1).as_str());
        }
    }

    pop_next!(
        let colon_token => "Expected a trailing colon in macro definition",
        let TokenValue::Colon => "Expected a trailing colon in macro definition"
    );

    assert_empty_line!();

    // The next lines until %endmacro are the body

    let mut body = Vec::new();
    loop {

        let mut body_line = token_lines.pop_front().unwrap_or_else(
            || error::parsing_error(&main_op, module_manager, "Missing %endmacro in macro definition")
        );

        /*
            The function macro body may contain some inline macros that are not exported.
            Consider the following example:

            foo.asm
            `
                %- SOME_INLINE_MACRO: 43

                %%- EXPORTED_FUNCTION_MACRO: 

                    mov1 r1 =SOME_INLINE_MACRO

                %endmacro
            `

            bar.asm
            `
                .include:

                    "foo.asm"

                .text:

                    !EXPORTED_FUNCTION_MACRO <--- issue
            `

            The issue is that `EXPORTED_FUNCTION_MACRO` gets exported by `foo.asm`
            and into `bar.asm`, but `SOME_INLINE_MACRO` does not.
            When `bar.asm` tries to invoke the imported function macro, the expansion is triggered.
            However, after expansion, `bar.asm` will have to resolve the inline macro `SOME_INLINE_MACRO`,
            which is actually defined inside `foo.asm` and is private.
            For this reason, inline macros must be readily expanded when parsing the function macro definition.
        */
        expand_inline_macros(&mut body_line, symbol_table, module_manager);

        if let Some(Token { value: TokenValue::Endmacro, .. }) = body_line.front() {
            break;
        }

        body.push(
            body_line.drain(..).collect::<Vec<Token>>().into_boxed_slice()
        );
    }

    let macro_def = FunctionMacroDef {
        source: Rc::clone(&name_token.source),
        params,
        body: body.into_boxed_slice(),
    };

    symbol_table.declare_function_macro(
        name,
        macro_def,
        export
    )
    .err().map(
        |old_def| error::symbol_redeclaration(&old_def.source, &name_token.source, module_manager, "Function macro name already declared")
    );
}


fn parse_inline_macro_def<'a>(line: &mut TokenList<'a>, main_op: Rc<SourceToken<'a>>, module_manager: &ModuleManager<'a>, symbol_table: &SymbolTable<'a>, export: bool) {
    
    declare_parsing_utils!(module_manager, line, main_op);

    pop_next!(
        let name_token => "Missing macro name in macro declaration",
        let TokenValue::Identifier(name) => "Expected an identifier as macro name in macro declaration"
    );
    
    pop_next!(
        let colon_token => "Missing a colon `:` after macro name in macro declaration",
        let TokenValue::Colon => "Expected a colon `:` after macro name in macro declaration"
    );

    let macro_def = InlineMacroDef {
        source: Rc::clone(&name_token.source),
        // The rest of the line is the macro definition
        def: line.drain(..).collect::<Vec<Token>>().into_boxed_slice()
    };

    symbol_table.declare_inline_macro(
        name,
        macro_def,
        export
    )
    .err().map(
        |old_def| error::symbol_redeclaration(&old_def.source, &name_token.source, module_manager, "Inline macro name already declared")
    );
}

