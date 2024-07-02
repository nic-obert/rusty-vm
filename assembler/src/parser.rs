use rusty_vm_lib::assembly::ByteCode;

use crate::{assembler, error};
use crate::lang::{AsmInstructionNode, AsmNode, AsmNodeValue, AsmOperand, AsmValue, FunctionMacroDef, InlineMacroDef, INCLUDE_SECTION_NAME};
use crate::tokenizer::{Token, TokenLines, TokenList, TokenValue};
use crate::symbol_table::SymbolTable;
use crate::module_manager::ModuleManager;

use std::collections::VecDeque;
use std::mem;
use std::path::Path;
use std::rc::Rc;


#[inline]
fn expand_inline_macros<'a>(tokens: &mut TokenList<'a>, symbol_table: &SymbolTable<'a>, module_manager: &ModuleManager<'a>) {

    let mut i: usize = 0;

    while let Some(token) = tokens.get(i) {

        // ! is the first token in the line, this is a function macro, not an inline macro
        if matches!(token.value, TokenValue::Bang) && i != 0 {

            let macro_name = tokens.get(i+1).unwrap_or_else(
                || error::parsing_error(&token.source, module_manager, "Missing macro name after `!`.")
            );

            let name = if let TokenValue::Identifier(id) = macro_name.value {
                id
            } else {
                error::parsing_error(&macro_name.source, module_manager, "Expected a macro name after `!`.")
            };

            let macro_def = symbol_table.get_inline_macro(name).unwrap_or_else(
                || error::parsing_error(&macro_name.source, module_manager, "Undefined macro name")
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
                    %- BAR: !FOO 90
                    mov8 !BAR
                
                Which should expand to

                    mov8 r1 90
            */
            continue;
        }

        i += 1;
    }
}


#[inline]
fn parse_operands<'a>(mut tokens: TokenList<'a>, module_manager: &'a ModuleManager<'a>) -> Box<[AsmOperand<'a>]> {

    let mut operands = Vec::with_capacity(tokens.len());

    // The parsing occurs on a left-to-right manner. In assembly, statements are not complex enough to require operator priorities.

    while let Some(token) = tokens.pop_front() {


        macro_rules! push_op {
            ($op:expr, $source:expr) => {
                operands.push(AsmOperand { value: $op, source: $source.source.clone() })
            }
        }

        match token.value {
            
            TokenValue::StringLiteral(s) => push_op!(AsmValue::StringLiteral(s), token),
            
            TokenValue::Number(n) => push_op!(AsmValue::Number(n.clone()), token),
            
            TokenValue::Identifier(id) => push_op!(AsmValue::Label(id), token),
            
            TokenValue::CurrentPosition => push_op!(AsmValue::CurrentPosition(()), token),
            
            TokenValue::Register(reg) => push_op!(AsmValue::Register(reg), token),

            TokenValue::CurlyOpen => {

                let macro_param_name = tokens.pop_front().unwrap_or_else(
                    || error::parsing_error(&token.source, module_manager, "Missing macro parameter name after `{`."));

                let symbol_id = if let TokenValue::Identifier(id) = macro_param_name.value {
                    id
                } else {
                    error::parsing_error(&macro_param_name.source, module_manager, "Expected a macro parameter name after `{`.")
                };

                let closing_curly = tokens.pop_front().unwrap_or_else(
                    || error::parsing_error(&token.source, module_manager, "Missing closing `}` after macro parameter name."));
                
                if !matches!(closing_curly.value, TokenValue::CurlyClose) {
                    error::parsing_error(&closing_curly.source, module_manager, "Expected a closing `}` after macro parameter name.");
                }

                push_op!(AsmValue::MacroParameter(symbol_id), token);
            },

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
                    TokenValue::Colon
                        => error::parsing_error(&addr_operand.source, module_manager, "Cannot address this token"),

                    TokenValue::Bang => unreachable!("Macros are expanded before the main parsing")
                }
            },
            
            TokenValue::Instruction(_) |
            TokenValue::Dot |
            TokenValue::Endmacro |
            TokenValue::CurlyClose |
            TokenValue::LabelDef { .. } |
            TokenValue::SquareClose |
            TokenValue::FunctionMacroDef { .. } |
            TokenValue::InlineMacroDef { .. } |
            TokenValue::Comma |
            TokenValue::Colon
                => error::parsing_error(&token.source, module_manager, "Token cannot be used as here."),
            
            TokenValue::Bang => unreachable!("Macros are expanded before the main parsing"),

        }

    }

    operands.into_boxed_slice()
}


#[inline]
fn parse_line<'a>(main_operator: Token<'a>, operands: Box<[AsmOperand<'a>]>, nodes: &mut Vec<AsmNode<'a>>, module_manager: &'a ModuleManager<'a>, symbol_table: &SymbolTable<'a>) {


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

        
        TokenValue::Bang => todo!(),    
        
        TokenValue::Instruction(instruction) => {

            let node = match AsmInstructionNode::build(instruction, operands) {
                Ok(node) => node,
                Err((faulty_token, hint))
                    => error::parsing_error(&faulty_token.unwrap_or(main_operator.source), module_manager, &hint),
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
        TokenValue::InlineMacroDef { .. } |
        TokenValue::FunctionMacroDef { .. } 
            => unreachable!() // Handled before as special cases
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
                mov8 [!CONST_ADDRESS] r1

            Performing a linear search for ! tokens is faster than using a tree parser, which would have
            to search each time for the highest priority token inside a linked list.
            Also, since assembly lines are generally pretty short, shifting the elements when expanding the macro
            should not be too costly. This would be totally different in a more complex programming language.
        */
        expand_inline_macros(&mut line, symbol_table, module_manager);

        if let Some(main_operator) = line.pop_front() {

            // Macro declarations are done first because they require their definitions to be made of Tokens and not AsmOperands

            if let TokenValue::InlineMacroDef { export } = main_operator.value {
                parse_inline_macro(&mut line, &main_operator, module_manager, symbol_table, export);
                continue;
            }

            if let TokenValue::FunctionMacroDef { export } = main_operator.value {
                parse_function_macro(&mut line, &main_operator, module_manager, &mut token_lines, symbol_table, export);
                continue;
            }

            /*
                Handle ASM sections separately
                The .include section is special because it only contains strings
                Other dedicated sections may be implemented
                All other sections are treated as labels
            */
            if matches!(main_operator.value, TokenValue::Dot) {
                parse_section(&mut nodes, &mut line, &main_operator, module_manager, &mut token_lines, symbol_table, bytecode);
                continue;
            }

            let operands = parse_operands(line, module_manager);

            parse_line(main_operator, operands, &mut nodes, module_manager, symbol_table);
        }
    }

    nodes.into_boxed_slice()
}


fn parse_section<'a>(nodes: &mut Vec<AsmNode<'a>>, line: &mut VecDeque<Token<'a>>, main_operator: &Token<'a>, module_manager: &'a ModuleManager<'a>, token_lines: &mut VecDeque<VecDeque<Token<'a>>>, symbol_table: &SymbolTable<'a>, bytecode: &mut ByteCode) {
    
    let name_token = line.pop_front().unwrap_or_else(
        || error::parsing_error(&main_operator.source, module_manager, "Expected a section name")
    );
    let name = if let TokenValue::Identifier(name) = name_token.value {
        name
    } else {
        error::parsing_error(&name_token.source, module_manager, "Expected an identifier as section name");
    };

    let colon_token = line.pop_front().unwrap_or_else(
        || error::parsing_error(&main_operator.source, module_manager, "Expected a trailing colon after section name in section delcaration.")
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
        source: Rc::clone(&main_operator.source),
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
                    Box::new(mem::take(s).into_owned()).leak()
                )
            } else {
                error::parsing_error(&include_line[0].source, module_manager, "Expected a string literal as include path")
            };

            let exports = assembler::assemble_included_unit(&main_operator.source.unit_path, include_path, module_manager, bytecode);

            symbol_table.import_symbols(exports, to_re_export, module_manager);
        }

    }
}


fn parse_function_macro<'a>(line: &mut VecDeque<Token<'a>>, main_operator: &Token<'a>, module_manager: &ModuleManager<'a>, token_lines: &mut VecDeque<VecDeque<Token<'a>>>, symbol_table: &SymbolTable<'a>, export: bool) {
    
    let name_token = line.pop_front().unwrap_or_else(
        || error::parsing_error(&main_operator.source, module_manager, "Expected a macro name")
    );
    let name = if let TokenValue::Identifier(name) = name_token.value {
        name
    } else {
        error::parsing_error(&name_token.source, module_manager, "Expected an identifier as macro name");
    };

    // The rest of the line is the macro parameters, except for the trailing semicolon

    let args = line.drain(..line.len()-1).map(
        |token| {
            if let TokenValue::Identifier(name) = token.value {
                name
            } else {
                error::parsing_error(&token.source, module_manager, "Expected an identifier as macro parameter");
            }
        }
        ).collect::<Vec<&str>>()
        .into_boxed_slice();

    let colon_token = line.get(0).unwrap_or_else(
        || error::parsing_error(&main_operator.source, module_manager, "Expected a trailing colon in macro definition")
    );
    if !matches!(colon_token.value, TokenValue::Colon) {
        error::parsing_error(&colon_token.source, module_manager, "Expected a trailing colon in macro definition");
    }

    // The next lines until %endmacro are the body

    let mut body = Vec::new();
    loop {

        let mut body_line = token_lines.pop_front().unwrap_or_else(
            || error::parsing_error(&main_operator.source, module_manager, "Missing %endmacro in macro definition")
        );

        if let Some(token) = body_line.front() {
            if matches!(token.value, TokenValue::Endmacro) {
                break;
            }
        }

        body.push(
            body_line.drain(..).collect::<Vec<Token>>().into_boxed_slice()
        );
    }

    let macro_def = FunctionMacroDef {
        source: Rc::clone(&main_operator.source),
        args,
        body: body.into_boxed_slice(),
    };

    symbol_table.declare_function_macro(name, macro_def, export);
}


fn parse_inline_macro<'a>(line: &mut VecDeque<Token<'a>>, main_operator: &Token<'a>, module_manager: &ModuleManager<'a>, symbol_table: &SymbolTable<'a>, export: bool) {
    
    let name_token = line.pop_front().unwrap_or_else(
        || error::parsing_error(&main_operator.source, module_manager, "Expected a macro name")
    );
    let name = if let TokenValue::Identifier(name) = name_token.value {
        name
    } else {
        error::parsing_error(&name_token.source, module_manager, "Expected an identifier as macro name");
    };

    let macro_def = InlineMacroDef {
        source: Rc::clone(&main_operator.source),
        // The rest of the line is the macro definition
        def: line.drain(..).collect::<Vec<Token>>().into_boxed_slice()
    };

    symbol_table.declare_inline_macro(name, macro_def, export);
}

