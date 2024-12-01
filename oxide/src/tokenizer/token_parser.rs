use core::str;
use std::path::Path;
use std::slice;

use regex::Regex;
use lazy_static::lazy_static;

use crate::module_manager::{self, Module};
use crate::tokenizer::SourceToken;
use crate::lang::errors::{print_errors_and_exit, CompilationError, ErrorKind};


lazy_static! {

    static ref TOKEN_REGEX: Regex = Regex::new(
        r#"(?m)'(?:\\'|[^'])*'|"(?:\\"|[^"])*"|[_a-zA-Z]\w*|-?\d+[.]\d*|-?[.]?\d+|->|==|<=|>=|!=|&&|\|\||//|/\*|\*/|[-+*/%\[\](){}=:<>!^&|~]|\S"#
    ).expect("Regex should compile");

}


fn escape_string_copy(string_to_escape: &str, checked_until: usize, token: &SourceToken, source: &SourceCode) -> Result<String, CompilationError> {
    // use -1 because the escape character won't be copied
    let mut escaped_string = String::with_capacity(string_to_escape.len() - 1);

    // Copy the part of the string before the escape character
    escaped_string.push_str(&string_to_escape[..checked_until]);

    let mut escape = true;

    for (i, c) in string_to_escape[checked_until + 1..].chars().enumerate() {
        if escape {
            escape = false;
            escaped_string.push(match c {
                'n' => '\n',
                'r' => '\r',
                '0' => '\0',
                't' => '\t',
                '\\' => '\\',
                c => error::invalid_escape_character(token.unit_path, c, token.column + checked_until + i + 2, token.line_index(), source, "Invalid escape character")
            })
        } else if c == '\\' {
            escape = true;
        } else {
            escaped_string.push(c);
        }
    }

    Ok(escaped_string)
}


fn escape_string<'a>(string: &'a str, token: &SourceToken, source: &SourceCode) -> Cow<'a, str> {
    // Ignore the enclosing quote characters
    let string = &string[1..string.len() - 1];

    for (i, c) in string.chars().enumerate() {
        if c == '\\' {
            let copied_string = escape_string_copy(string, i, token, source);
            return Cow::Owned(copied_string);
        }
    }

    Cow::Borrowed(string)
}


fn is_numeric(c: char) -> bool {
    c.is_numeric() || c == '-' || c == '+' || c == '.'
}


fn may_be_expression(token: Option<&Token>) -> bool {
    token.map(|t| matches!(t.value,
        TokenKind::ParClose |
        TokenKind::SquareClose |
        TokenKind::Value(_)
    )).unwrap_or(false)
}


fn is_symbol_name(name: &str) -> bool {
    let mut chars = name.chars();

    if let Some(c) = chars.next() {
        if !(c.is_alphabetic() || c == '_') {
            return false;
        }
    }

    for c in chars {
        if !(c.is_alphanumeric() || c == '_') {
            return false;
        }
    }

    true
}


fn is_data_type_precursor(token: &Token) -> bool {
    matches!(token.value, TokenKind::DataType(_) | TokenKind::ArrayTypeOpen | TokenKind::Colon | TokenKind::Arrow | TokenKind::RefType | TokenKind::As)
}


/// Useful struct to keep track of the status of the tokenizer
struct TokenizerStatus {

    pub parenthesis_depth: usize,
    pub square_depth: usize,
    pub curly_depth: usize,
    pub base_priority: TokenPriority,
    priority_delta: TokenPriority,

}

impl TokenizerStatus {

    pub fn new() -> TokenizerStatus {
        TokenizerStatus {
            parenthesis_depth: 1,
            square_depth: 1,
            curly_depth: 1,
            base_priority: 0,
            priority_delta: 0,
        }
    }

    pub fn update(&mut self) {
        self.base_priority += self.priority_delta;
        self.priority_delta = 0;
    }

    pub fn enter_parenthesis(&mut self) {
        self.parenthesis_depth += 1;
        self.priority_delta += Priority::Delimiter as TokenPriority;
    }

    pub fn leave_parenthesis(&mut self) -> Result<(), ()> {
        self.parenthesis_depth -= 1;
        self.priority_delta -= Priority::Delimiter as TokenPriority;

        if self.parenthesis_depth == 0 {
            Err(())
        } else {
            Ok(())
        }
    }

    pub fn enter_square(&mut self) {
        self.square_depth += 1;
        self.priority_delta += Priority::Delimiter as TokenPriority;
    }

    pub fn leave_square(&mut self) -> Result<(), ()> {
        self.square_depth -= 1;
        self.priority_delta -= Priority::Delimiter as TokenPriority;

        if self.square_depth == 0 {
            Err(())
        } else {
            Ok(())
        }
    }

    pub fn enter_curly(&mut self) {
        self.curly_depth += 1;
        self.priority_delta = Priority::Delimiter as TokenPriority;
    }

    pub fn leave_curly(&mut self) -> Result<(), ()> {
        self.curly_depth -= 1;
        self.priority_delta = - (Priority::Delimiter as TokenPriority);

        if self.curly_depth == 0 {
            Err(())
        } else {
            Ok(())
        }
    }

}


/// Divide the source code into meaningful string tokens
fn lex<'a>(source: &[&'a str], unit_path: &'a Path) -> impl Iterator<Item = SourceToken<'a>> {

    enum MultilineObject {
        /// No multiline object is currently being parsed
        None,
        /// A multiline comment
        Comment,
        /// A potentially multiline string
        String { token: SourceToken, token_start_ptr: *const u8 }
    }

    let mut multiline_object = MultilineObject::None;

    source.iter().enumerate().flat_map(
        move |(line_index, line)| {

            if line.trim().is_empty() && !multiline_comment {
                return Vec::new();
            }

            let mut matches = Vec::new();

            for mat in TOKEN_REGEX.find_iter(line) {

                match multiline_object {

                    MultilineObject::None => {
                        match mat.as_str() {
                            "//" => break,
                            "/*" => {
                                multiline_object = MultilineObject::Comment;
                            },
                            "\"" => {
                                multiline_object = MultilineObject::String {
                                    token: SourceToken::new(line_index, mat.start(), "", unit_path),
                                    token_start_ptr: mat.as_str().as_ptr()
                                };
                            },
                            string => matches.push(
                                SourceToken::new(
                                    line_index,
                                    mat.start(),
                                    string,
                                    unit_path
                                )
                            )
                        }
                    },

                    MultilineObject::Comment => {
                        if mat.as_str() == "*/" {
                            // End of the multiline comment
                            multiline_object = MultilineObject::None;
                        }
                    },

                    MultilineObject::String { token, token_start_ptr } => {
                        // Ignore all other matches because they are part of the multiline string.
                        // A better approach should be used where the regex is applied to the whole source code
                        // at the same time. Or one manual forward-iteration could be the solution.
                        if mat.as_str() == "\"" {
                            // Found the closing delimiter
                            matches.push(SourceToken::new(
                                token.line_index(),
                                token.column_index,
                                unsafe {
                                    // Construct the string slice from the pointer to the opening delimiter
                                    // and the pointer to its closing delimiter
                                    let token_end_ptr = mat.as_str().as_ptr().byte_add(mat.as_str().len()) as usize;
                                    let length_bytes = token_end_ptr - token_start_ptr as usize;
                                    let raw_slice = slice::from_raw_parts(token_start_ptr, length_bytes);
                                    let string = str::from_utf8(raw_slice).expect("Slice should be valid utf-8");
                                    string
                                },
                                token.module_path,
                            ));
                            multiline_object = MultilineObject::None;
                        }
                    }
                }
            }
            matches
        }
    )
}


pub fn tokenize<'a>(module: &Module<'a>) -> () {

    let mut phase_errors: Vec<CompilationError> = Vec::new();

    let raw_tokens = lex(module.lines(), module.path);

    let mut tokens = TokenParsingList::new();

    let mut ts = TokenizerStatus::new();

    for token in raw_tokens {

        let token_kind = match token.string {

            "->" => TokenKind::Arrow,
            "{" => {
                ts.enter_curly();
                TokenKind::ScopeOpen
            },
            "}" => {
                if ts.leave_curly().is_err() {
                    phase_errors.push(CompilationError {
                        source: Rc::new(token),
                        kind: ErrorKind::UnmatchedDelimiter { delimiter: '}' },
                        hint: "Unexpected closing delimiter. Did you forget a '{'?"
                    });
                    continue;
                }

                TokenKind::ScopeClose
            },
            "(" => {
                ts.enter_parenthesis();

                let mut iter = tokens.iter_lex_tokens().rev();
                let last = iter.next();

                if may_be_expression(last) {

                    // Unwrap is safe because `last_node` is an expression
                    let last_token = last.unwrap();

                    if matches!(last_token.value, TokenKind::Value(Value::Symbol { .. })) && iter.next().map(|token| matches!(token.value, TokenKind::Fn)).unwrap_or(false) {
                        // Syntax: fn <symbol> (
                        TokenKind::FunctionParamsOpen
                    } else {
                        // Syntax: <value-like> (
                        TokenKind::FunctionCallOpen
                    }
                } else {
                    // Syntax: <not-a-value> (
                    TokenKind::ParOpen
                }
            },
            ")" => {
                if ts.leave_parenthesis().is_err() {
                    phase_errors.push(CompilationError {
                        source: Rc::new(token),
                        kind: ErrorKind::UnmatchedDelimiter { delimiter: ')' },
                        hint: "Unexpected closing delimiter. Did you forget a '('?"
                    });
                    continue;
                }

                TokenKind::ParClose
            },
            "[" => {
                ts.enter_square();

                if let Some(last_token) = tokens.last_token() {
                    if is_data_type_precursor(last_token) {
                        // Syntax: <data-type-precursor> [
                        TokenKind::ArrayTypeOpen
                    } else if may_be_expression(Some(last_token)) {
                        // Syntax: <possible-array-type> [
                        TokenKind::ArrayIndexOpen
                    } else {
                        // Syntax: <not-a-data-type-precursor> [
                        TokenKind::ArrayOpen
                    }
                } else {
                    // Syntax: <nothing> [
                    TokenKind::ArrayOpen
                }
            },
            "]" => {
                if ts.leave_square().is_err() {
                    phase_errors.push(CompilationError {
                        source: Rc::new(token),
                        kind: ErrorKind::UnmatchedDelimiter { delimiter: ']' },
                        hint: "Unexpected closing delimiter. Did you forget a '['?"
                    });
                    continue;
                }

                TokenKind::SquareClose
            },
            ":" => TokenKind::Colon,
            ";" => TokenKind::Semicolon,
            "," => TokenKind::Comma,
            "+" => TokenKind::Add,
            "-" => TokenKind::Sub,
            "*" => {
                let last_token = tokens.last_token();
                if may_be_expression(last_token) { TokenKind::Mul } else { TokenKind::Deref }
            },
            "/" => TokenKind::Div,
            "%" => TokenKind::Mod,
            "=" => TokenKind::Assign,
            "==" => TokenKind::Equal,
            "!=" => TokenKind::NotEqual,
            "<" => TokenKind::Less,
            ">" => TokenKind::Greater,
            "<=" => TokenKind::LessEqual,
            ">=" => TokenKind::GreaterEqual,
            "&" => {
                let last_token = tokens.last_token();
                if may_be_expression(last_token) {
                    // Syntax: <value-like> &
                    TokenKind::BitwiseAnd
                } else if let Some(last_token) = last_token {
                    if is_data_type_precursor(last_token) {
                        // Syntax: <data-type-precursor> &
                        TokenKind::RefType
                    } else {
                        // Syntax: <not-a-data-type-precursor> &
                        TokenKind::Ref
                    }
                } else {
                    // Syntax: <nothing> &
                    TokenKind::Ref
                }
            },
            "^" => TokenKind::BitwiseXor,
            "<<" => TokenKind::BitShiftLeft,
            ">>" => TokenKind::BitShiftRight,
            "~" => TokenKind::BitwiseNot,
            "!" => TokenKind::LogicalNot,
            "&&" => TokenKind::LogicalAnd,
            "||" => TokenKind::LogicalOr,
            "|" => TokenKind::BitwiseOr,

            // Numbers
            string if string.starts_with(is_numeric) => {
                if string.contains('.') {
                    TokenKind::Value(Value::Literal { value: LiteralValue::Numeric(Number::try_parse_float(string).unwrap_or_else(
                        |e| error::invalid_number(&token, source, e.to_string().as_str())
                    )).into() })
                } else if string.starts_with('-') {
                    TokenKind::Value(Value::Literal { value: LiteralValue::Numeric(Number::try_parse_signed_int(string).unwrap_or_else(
                        |e| error::invalid_number(&token, source, e.to_string().as_str())
                    )).into() })
                } else {
                    TokenKind::Value(Value::Literal { value: LiteralValue::Numeric(Number::try_parse_unsigned_int(string).unwrap_or_else(
                        |e| error::invalid_number(&token, source, e.to_string().as_str())
                    )).into() })
                }
            },

            // Strings within ""
            string if string.starts_with('"') => {
                if string.len() == 1 {
                    error::unmatched_delimiter('"', &token, source, "Unexpected closing delimiter. Did you forget a '\"'?")
                }

                let string = escape_string(string, &token, source);

                // Add the string to the static string table, store the id in the token
                let static_id = symbol_table.add_static_string(string);

                TokenKind::Value(Value::Literal {
                    value: LiteralValue::StaticString(static_id).into()
                })
            },

            // Characters within ''
            string if string.starts_with('\'') => {
                if string.len() == 1 {
                    error::unmatched_delimiter('\'', &token, source, "Unexpected closing delimiter. Did you forget a \"'?\"?")
                }

                let s = escape_string(string, &token, source);
                if s.len() != 1 {
                    error::invalid_char_literal(&s, &token, source, "Character literals can only be one character long")
                }

                TokenKind::Value(Value::Literal {
                    value: LiteralValue::Char(s.chars().next().unwrap()).into()
                })
            },

            // Reserved keywords
            "fn" => TokenKind::Fn,
            "return" => TokenKind::Return,
            "let" => TokenKind::Let,
            "mut" => TokenKind::Mut,
            "as" => TokenKind::As,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "loop" => TokenKind::Loop,
            "true" => TokenKind::Value(Value::Literal { value: LiteralValue::Bool(true).into() }),
            "false" => TokenKind::Value(Value::Literal { value: LiteralValue::Bool(false).into() }),
            "const" => TokenKind::Const,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "typedef" => TokenKind::TypeDef,
            "do" => TokenKind::DoWhile,
            "static" => TokenKind::Static,

            // Data types
            "i8" => TokenKind::DataType(DataType::I8.into()),
            "i16" => TokenKind::DataType(DataType::I16.into()),
            "i32" => TokenKind::DataType(DataType::I32.into()),
            "i64" => TokenKind::DataType(DataType::I64.into()),
            "u8" => TokenKind::DataType(DataType::U8.into()),
            "u16" => TokenKind::DataType(DataType::U16.into()),
            "u32" => TokenKind::DataType(DataType::U32.into()),
            "u64" => TokenKind::DataType(DataType::U64.into()),
            "f32" => TokenKind::DataType(DataType::F32.into()),
            "f64" => TokenKind::DataType(DataType::F64.into()),
            "char" => TokenKind::DataType(DataType::Char.into()),
            "str" => TokenKind::DataType(DataType::RawString { length: 0 }.into()), // Set the length temporarily to 0
            "String" => TokenKind::DataType(DataType::String.into()),
            "void" => TokenKind::DataType(DataType::Void.into()),
            "bool" => TokenKind::DataType(DataType::Bool.into()),
            "usize" => TokenKind::DataType(DataType::Usize.into()),
            "isize" => TokenKind::DataType(DataType::Isize.into()),

            // Variable names
            string => {
                if !is_symbol_name(string) {
                    error::invalid_token(&token, source, "Not a valid symbol name.")
                }

                TokenKind::Value(Value::Symbol { name: string, scope_discriminant: ScopeDiscriminant::default() })
            }
        };

        tokens.push_token(
            Token::new(token_kind, token, ts.base_priority)
        );

        ts.update();
    }

    if !phase_errors.is_empty() {
        // TODO: use a global enum with phase names?
        print_errors_and_exit("tokenization", &phase_errors);
    }
}
