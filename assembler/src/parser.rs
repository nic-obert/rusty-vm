use rusty_vm_lib::assembly::ByteCode;

use crate::{assembler, error};
use crate::lang::{AsmInstructionNode, AsmNode, AsmNodeValue, AsmOperand, AsmValue, FunctionMacroDef, InlineMacroDef, Number, PseudoInstructionNode, PseudoInstructions, INCLUDE_SECTION_NAME};
use crate::tokenizer::{SourceToken, Token, TokenLines, TokenList, TokenValue};
use crate::symbol_table::SymbolTable;
use crate::module_manager::ModuleManager;

use std::collections::{HashMap, VecDeque};
use std::mem;
use std::path::Path;
use std::rc::Rc;


fn expand_inline_macros<'a>(tokens: &mut TokenList<'a>, symbol_table: &SymbolTable<'a>, module_manager: &ModuleManager<'a>) {

    let mut i: usize = 0;

    while let Some(token) = tokens.get(i) {

        if matches!(token.value, TokenValue::Equals) {

            let macro_name = tokens.get(i+1).unwrap_or_else(
                || error::parsing_error(&token.source, module_manager, "Missing macro name after `=`")
            );

            let name = if let TokenValue::Identifier(id) = macro_name.value {
                id
            } else {
                error::parsing_error(&macro_name.source, module_manager, "Expected a macro name after `=`")
            };

            let macro_def = symbol_table.get_inline_macro(name).unwrap_or_else(
                || error::undefined_macro(&macro_name.source, module_manager, symbol_table.inline_macros())
            );

            // symbol_table.

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
            
            TokenValue::CurrentPosition => push_op!(AsmValue::CurrentPosition(()), token),
            
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
                    
                    TokenValue::CurrentPosition => push_op!(AsmValue::CurrentPosition(()), token),

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


    macro_rules! check_arg_count {

        ($required:expr) => {
            if operands.len() != $required {
                error::parsing_error(&main_operator.source, module_manager, format!("Operator expects exactly {} arguments, but {} were given.", $required, operands.len()).as_str())
            }
        };

        ($required:expr, $operands:ident) => {
            if $operands.len() != $required {
                error::parsing_error(&main_operator.source, module_manager, format!("Operator expects exactly {} arguments, but {} were given.", $required, $operands.len()).as_str())
            }
        }
    }

    macro_rules! parse_label {
        ($position:literal) => {{

            let op = &operands[$position];

            if let AsmValue::Label(s) = op.value {
                s
            } else {
                error::parsing_error(&op.source, module_manager, format!("Operator `{}` expects an identifier argument", main_operator.source.string).as_str())
            }
        }};
    }


    match main_operator.value {

        TokenValue::LabelDef { export } => {

            check_arg_count!(1);

            let label = parse_label!(0);
            
            nodes.push(AsmNode {
                source: Rc::clone(&main_operator.source),
                value: AsmNodeValue::Label(label)
            });

            symbol_table.declare_label(
                label, 
                main_operator.source, 
                export
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
        TokenValue::CurrentPosition |
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
                    expand_function_macro(&mut line, &main_operator, &mut token_lines, module_manager, symbol_table);
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


fn parse_pseudo_instruction<'a>(instruction: PseudoInstructions, main_op: Rc<SourceToken<'a>>, line: &mut TokenList<'a>, nodes: &mut Vec<AsmNode<'a>>, module_manager: &'a ModuleManager<'a>) {

    match instruction {

        PseudoInstructions::DefineNumber => {

            let size_token = line.pop_front().unwrap_or_else(
                || error::parsing_error(&main_op, module_manager, "Missing number size specifier in static data declaration")
            );
            let size = if let TokenValue::Number(n) = size_token.value {
                n
            } else {
                error::parsing_error(&size_token.source, module_manager, "Expected a numeric size specifier in static data declaration");
            };

            let size = if let Number::UnsignedInt(s) = size {
                s as usize
            } else {
                error::parsing_error(&size_token.source, module_manager, "Data size must be an unsigned integer")
            };

            let number_token = line.pop_front().unwrap_or_else(
                || error::parsing_error(&size_token.source, module_manager, "Missing numeric value in static data declaration")
            );
            let number = if let TokenValue::Number(n) = number_token.value {
                n
            } else {
                error::parsing_error(&number_token.source, module_manager, "Expected a numeric value in static data declaration");
            };

            if !line.is_empty() {
                error::parsing_error(&line[0].source, module_manager, "Unexpected token in static data declaration")
            }

            nodes.push(AsmNode {
                source: main_op,
                value: AsmNodeValue::PseudoInstruction (
                    PseudoInstructionNode::DefineNumber { 
                        size: (size, size_token.source), 
                        data: (number, number_token.source) 
                    }
                )
            });
        },

        PseudoInstructions::DefineString => {

            let string_token = line.pop_front().unwrap_or_else(
                || error::parsing_error(&main_op, module_manager, "Missing string data in static data declaration")
            );
            let string = if let TokenValue::StringLiteral(s) = string_token.value {
                s
            } else {
                error::parsing_error(&string_token.source, module_manager, "Expected a string literal in static data declaration");
            };

            if !line.is_empty() {
                error::parsing_error(&line[0].source, module_manager, "Unexpected token in static data declaration")
            }

            nodes.push(AsmNode {
                source: main_op,
                value: AsmNodeValue::PseudoInstruction (
                    PseudoInstructionNode::DefineString {
                        data: (string, string_token.source)
                    }
                )
            });
        },

        PseudoInstructions::DefineBytes => todo!(),

    }

}


fn parse_section<'a>(nodes: &mut Vec<AsmNode<'a>>, line: &mut VecDeque<Token<'a>>, main_op: Rc<SourceToken<'a>>, module_manager: &'a ModuleManager<'a>, token_lines: &mut VecDeque<VecDeque<Token<'a>>>, symbol_table: &SymbolTable<'a>, bytecode: &mut ByteCode) {
    
    let name_token = line.pop_front().unwrap_or_else(
        || error::parsing_error(&main_op, module_manager, "Expected a section name")
    );
    let name = if let TokenValue::Identifier(name) = name_token.value {
        name
    } else {
        error::parsing_error(&name_token.source, module_manager, "Expected an identifier as section name");
    };

    let colon_token = line.pop_front().unwrap_or_else(
        || error::parsing_error(&main_op, module_manager, "Expected a trailing colon after section name in section delcaration.")
    );
    if !matches!(colon_token.value, TokenValue::Colon) {
        error::parsing_error(&colon_token.source, module_manager, "Expected a trailing colon after section name in section delcaration.");
    }
    if !line.is_empty() {
        error::parsing_error(&line.pop_front().unwrap().source, module_manager, "Unexpected token after section declaration. A section delcaration must end with a colon.");
    }

    // Declare the section label so that it can be used. Section labels are not exportable.
    symbol_table.declare_label(name, name_token.source, false);

    nodes.push(AsmNode {
        source: Rc::clone(&main_op),
        value: AsmNodeValue::Label(name)
    });

    if name == INCLUDE_SECTION_NAME {

        // Each line in the .include section is a string path to include
        while let Some(mut include_line) = token_lines.pop_front() {

            // Assume the line cannot be empty because macros aren't expanded in the .include section and empty lines are ignored.

            if matches!(include_line.front().unwrap().value, TokenValue::Dot) {
                // Start of another section
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
                error::io_error(err, format!("Failed to resolve path \"{}\"", include_path.display()).as_str())
            );

            assembler::assemble_included_unit(include_path, module_manager, bytecode);

            let exports = module_manager.get_unit_exports(include_path);

            symbol_table.import_symbols(exports, to_re_export, module_manager);
        }

    }
}


fn expand_function_macro<'a>(line: &mut VecDeque<Token<'a>>, main_operator: &Token<'a>, token_lines: &mut TokenLines<'a>, module_manager: &ModuleManager<'a>, symbol_table: &SymbolTable<'a>) {

    let macro_name_op = line.pop_front().unwrap_or_else(
        || error::parsing_error(&main_operator.source, module_manager, "Missing macro name after `!`")
    );
    let macro_name = if let TokenValue::Identifier(name) = macro_name_op.value {
        name
    } else {
        error::parsing_error(&macro_name_op.source, module_manager, "Expected a macro name after `!`");
    };

    let macro_def = symbol_table.get_function_macro(macro_name).unwrap_or_else(
        || error::undefined_macro(&macro_name_op.source, module_manager, symbol_table.function_macros())
    );

    if line.len() != macro_def.params.len() {
        error::parsing_error(&main_operator.source, module_manager, format!("Mismatched argument count in macro call: Expected {}, got {}", macro_def.params.len(), line.len()).as_str())
    }

    let expanded_macro = macro_def.body.iter().map(
        |original_line| {

            let mut expanded_line = TokenList::with_capacity(original_line.len());

            let mut i = 0;
            
            while let Some(token) = original_line.get(i) {
                
                // Start of a macro parameter. Syntax: `{param}`
                if matches!(token.value, TokenValue::CurlyOpen) {

                    i += 1;
                    let param_name_token = original_line.get(i).unwrap_or_else(
                        || error::parsing_error(&token.source, module_manager, "Missing macro parameter name after `{` inside macro body")
                    );
                    let param_name = if let TokenValue::Identifier(name) = param_name_token.value {
                        name
                    } else {
                        error::parsing_error(&param_name_token.source, module_manager, "Expected a macro parameter name after '{` inside macro body");
                    };

                    let arg_position = *macro_def.params.get(param_name).unwrap_or_else(
                        || error::parsing_error(&param_name_token.source, module_manager, "Undefined macro parameter name")
                    );

                    let substitute_token = &line[arg_position];

                    i += 1;
                    let closing_curly_token = original_line.get(i).unwrap_or_else(
                        || error::parsing_error(&token.source, module_manager, "Missing closing `}` after macro parameter name inside macro body")
                    );
                    if !matches!(closing_curly_token.value, TokenValue::CurlyClose) {
                        error::parsing_error(&closing_curly_token.source, module_manager, "Expected closing `}` after macro parameter name inside macro body");
                    }

                    expanded_line.push_back(substitute_token.clone());

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


fn parse_function_macro_def<'a>(line: &mut VecDeque<Token<'a>>, main_op: Rc<SourceToken<'a>>, module_manager: &ModuleManager<'a>, token_lines: &mut VecDeque<VecDeque<Token<'a>>>, symbol_table: &SymbolTable<'a>, export: bool) {
    
    let name_token = line.pop_front().unwrap_or_else(
        || error::parsing_error(&main_op, module_manager, "Expected a macro name")
    );
    let name = if let TokenValue::Identifier(name) = name_token.value {
        name
    } else {
        error::parsing_error(&name_token.source, module_manager, "Expected an identifier as macro name");
    };

    // The rest of the line is the macro parameters, except for the trailing semicolon

    let mut params = HashMap::new();

    for (index, param_token) in line.drain(..line.len()-1).enumerate() {

        let param_name = if let TokenValue::Identifier(name) = param_token.value {
            name
        } else {
            error::parsing_error(&param_token.source, module_manager, "Expected an identifier as macro parameter");
        };

        if let Some(omonym_index) = params.insert(param_name, index) {
            error::parsing_error(&param_token.source, module_manager, format!("Duplicated parameter name in macro definition. The parameter is mentioned before in position {}", omonym_index+1).as_str());
        }
    }

    let colon_token = line.get(0).unwrap_or_else(
        || error::parsing_error(&main_op, module_manager, "Expected a trailing colon in macro definition")
    );
    if !matches!(colon_token.value, TokenValue::Colon) {
        error::parsing_error(&colon_token.source, module_manager, "Expected a trailing colon in macro definition");
    }

    // The next lines until %endmacro are the body

    let mut body = Vec::new();
    loop {

        let mut body_line = token_lines.pop_front().unwrap_or_else(
            || error::parsing_error(&main_op, module_manager, "Missing %endmacro in macro definition")
        );

        /*
            TODO: this may also be true for function macros inside other macros
            TODO: this may also be true for inline macros inside other inline macros

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

    symbol_table.declare_function_macro(name, macro_def, export);
}


fn parse_inline_macro_def<'a>(line: &mut VecDeque<Token<'a>>, main_op: Rc<SourceToken<'a>>, module_manager: &ModuleManager<'a>, symbol_table: &SymbolTable<'a>, export: bool) {
    
    let name_token = line.pop_front().unwrap_or_else(
        || error::parsing_error(&main_op, module_manager, "Missing macro name in macro declaration")
    );
    let name = if let TokenValue::Identifier(name) = name_token.value {
        name
    } else {
        error::parsing_error(&name_token.source, module_manager, "Expected an identifier as macro name in macro declaration");
    };

    let colon_token = line.pop_front().unwrap_or_else(
        || error::parsing_error(&name_token.source, module_manager, "Missing `:` after macro name in macro declaration")
    );
    if !matches!(colon_token.value, TokenValue::Colon) {
        error::parsing_error(&colon_token.source, module_manager, "Expected a `:` after macro name in macro declaration");
    }

    let macro_def = InlineMacroDef {
        source: Rc::clone(&name_token.source),
        // The rest of the line is the macro definition
        def: line.drain(..).collect::<Vec<Token>>().into_boxed_slice()
    };

    symbol_table.declare_inline_macro(name, macro_def, export);
}

