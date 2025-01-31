use std::borrow::Cow;
use std::rc::Rc;
use std::str::FromStr;
use std::mem;

use crate::compiler::CompilationPhases;
use crate::lang::{DataType, DataTypeName, LiteralValue, Number};
use crate::module_manager::Module;
use crate::tokenizer::{SourceToken, TokenList, TokenValue, Value};
use crate::lang::errors::{print_errors_and_exit, CompilationError, ErrorKind};

use super::{Priority, TokenPriority, Token};


const SIZE_OF_QUOTE_CHAR: usize = "\"".len();
const SIZE_OF_NEWLINE_CHAR: usize = "\n".len();
const SIZE_OF_ESCAPE_CHAR: usize = "\\".len();


fn may_be_expression(token: Option<&Token>) -> bool {
    token.map(|t| matches!(t.value,
        TokenValue::ParClose |
        TokenValue::ScopeClose |
        TokenValue::Value(_)
    )).unwrap_or(false)
}


// fn is_data_type_precursor(token: &Token) -> bool {
//     matches!(token.value, TokenKind::DataType(_) | TokenKind::ArrayTypeOpen | TokenKind::Colon | TokenKind::Arrow | TokenKind::RefType | TokenKind::As)
// }


/// Useful struct to keep track of the status of the tokenizer
struct TokenizerStatus {

    pub parenthesis_depth: usize,
    pub square_depth: usize,
    pub curly_depth: usize,
    pub base_priority: TokenPriority,
    pub line_index: usize,
    pub line_start_index: usize,

}

impl TokenizerStatus {

    pub fn new() -> TokenizerStatus {
        TokenizerStatus {
            parenthesis_depth: 0,
            square_depth: 0,
            curly_depth: 0,
            base_priority: 0,
            line_index: 0,
            line_start_index: 0,
        }
    }

    pub fn enter_parenthesis(&mut self) {
        self.parenthesis_depth += 1;
        self.base_priority += Priority::Delimiter as TokenPriority;
    }

    pub fn leave_parenthesis(&mut self) -> Result<(), ()> {
        self.base_priority -= Priority::Delimiter as TokenPriority;
        if let Some(res) = self.parenthesis_depth.checked_sub(1) {
            self.parenthesis_depth = res;
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn enter_square(&mut self) {
        self.square_depth += 1;
        self.base_priority += Priority::Delimiter as TokenPriority;
    }

    pub fn leave_square(&mut self) -> Result<(), ()> {
        self.base_priority -= Priority::Delimiter as TokenPriority;
        if let Some(res) = self.square_depth.checked_sub(1) {
            self.square_depth = res;
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn enter_curly(&mut self) {
        self.curly_depth += 1;
        self.base_priority = Priority::Delimiter as TokenPriority;
    }

    pub fn leave_curly(&mut self) -> Result<(), ()> {
        self.base_priority -= Priority::Delimiter as TokenPriority;
        if let Some(res) = self.curly_depth.checked_sub(1) {
            self.curly_depth = res;
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn newline(&mut self, newline_index: usize) {
        self.line_index += 1;
        self.line_start_index = newline_index + SIZE_OF_NEWLINE_CHAR;
    }

    pub fn column_index(&self, char_index: usize) -> usize {
        char_index - self.line_start_index
    }

}


enum StringStatus {
    Raw,
    Escaped { string: String, escape_on: bool }
}


pub fn tokenize<'a>(module: &Module<'a>) -> TokenList<'a> {

    let mut phase_errors: Vec<CompilationError> = Vec::new();

    let mut tokens = TokenList::new();

    let mut ts = TokenizerStatus::new();

    let raw_source = module.raw_source();
    let mut chars_iter = raw_source.char_indices().peekable();
    let mut unget_buf: Option<(usize, char)> = None;

    let mut token_is_negative_number: Option<usize> = None;

    /// Useful macro that gets the next character in the stream and updates eventual tokenizer information.
    macro_rules! get_next_char {
        () => {{
            if let Some(x) = unget_buf.take() {
                Some(x)
            } else if let Some((index, ch)) = chars_iter.next() {
                if ch == '\n' {
                    ts.newline(index);
                }
                Some((index, ch))
            } else {
                None
            }
        }};
    }
    macro_rules! skip_next_char {
        () => {
            assert!(chars_iter.next().is_some())
        }
    }
    macro_rules! unget_char {
        ($item:expr) => {
            assert!(unget_buf.replace($item).is_none());
        }
    }
    macro_rules! peek_char {
        () => {{
            if let Some(x) = &unget_buf {
                Some(x)
            } else {
                chars_iter.peek()
            }
        }}
    }

    while let Some((base_char_index, ch)) = get_next_char!() {
        match ch {

            _ if ch.is_ascii_digit() => {
                let start_line_index = ts.line_index;
                let (base_char_index, is_negative) =
                    if let Some(minus_index) = token_is_negative_number.take() {
                        (minus_index, true)
                    } else {
                        (base_char_index, false)
                    };
                let start_column_index = ts.column_index(base_char_index);
                let mut number_end_index = base_char_index + ch.len_utf8();
                let mut is_float = false;

                while let Some((number_index, digit_ch)) = get_next_char!() {
                    if digit_ch == '.' {
                        if is_float {
                            // Number is already a float, so this point should be an indirection operator
                            unget_char!((number_index, digit_ch));
                            break;
                        } else {
                            is_float = true;
                            number_end_index += digit_ch.len_utf8();
                        }
                    } else if !digit_ch.is_ascii_digit() {
                        unget_char!((number_index, digit_ch));
                        break;
                    } else {
                        number_end_index += digit_ch.len_utf8();
                    }
                }

                let string_tok = unsafe {
                    std::str::from_raw_parts(
                        raw_source.as_ptr().byte_add(base_char_index),
                        number_end_index - base_char_index
                    )
                };
                let source = SourceToken::new(
                    start_line_index,
                    start_column_index,
                    string_tok
                );

                let number =
                    if is_float {
                        let f = string_tok.parse::<f64>().expect("Should not fail");
                        Number::F64(f)
                    } else if is_negative {
                        let n = string_tok.parse::<i64>().expect("Should not fail");
                        Number::I64(n)
                    } else {
                        let n = string_tok.parse::<u64>().expect("Should not fail");
                        Number::U64(n)
                    };

                tokens.push(
                    Rc::new(source),
                    TokenValue::Value(Value::Literal(LiteralValue::Number(number))),
                    ts.base_priority
                );
            },

            _ if ch.is_alphabetic() || ch == '_' => {
                let start_line_index = ts.line_index;
                let start_column_index = ts.column_index(base_char_index);
                let mut symbol_end_index = base_char_index + ch.len_utf8();

                while let Some((symbol_char_index, symbol_ch)) = get_next_char!() {
                    if !symbol_ch.is_alphanumeric() && symbol_ch != '_' {
                        symbol_end_index = symbol_char_index;
                        unget_char!((symbol_char_index, symbol_ch));
                        break;
                    }
                }

                let string_tok = unsafe {
                    std::str::from_raw_parts(
                        raw_source.as_ptr().byte_add(base_char_index),
                        symbol_end_index - base_char_index
                    )
                };
                let source = SourceToken::new(
                    start_line_index,
                    start_column_index,
                    string_tok
                );
                let token = match string_tok {

                    // Reserved keywords
                    "fn" => TokenValue::Fn,
                    "return" => TokenValue::Return,
                    "let" => TokenValue::Let,
                    "mut" => TokenValue::Mut,
                    "as" => TokenValue::As,
                    "if" => TokenValue::If,
                    "else" => TokenValue::Else,
                    "while" => TokenValue::While,
                    "loop" => TokenValue::Loop,
                    "true" => TokenValue::Value(Value::Literal(LiteralValue::Bool(true))),
                    "false" => TokenValue::Value(Value::Literal(LiteralValue::Bool(false))),
                    "const" => TokenValue::Const,
                    "break" => TokenValue::Break,
                    "continue" => TokenValue::Continue,
                    "typedef" => TokenValue::TypeDef,
                    "do" => TokenValue::DoWhile,
                    "static" => TokenValue::Static,

                    // Data types
                    "i8" => TokenValue::BuiltinType(DataType::I8),
                    "i16" => TokenValue::BuiltinType(DataType::I16),
                    "i32" => TokenValue::BuiltinType(DataType::I32),
                    "i64" => TokenValue::BuiltinType(DataType::I64),
                    "u8" => TokenValue::BuiltinType(DataType::U8),
                    "u16" => TokenValue::BuiltinType(DataType::U16),
                    "u32" => TokenValue::BuiltinType(DataType::U32),
                    "u64" => TokenValue::BuiltinType(DataType::U64),
                    "f32" => TokenValue::BuiltinType(DataType::F32),
                    "f64" => TokenValue::BuiltinType(DataType::F64),
                    "char" => TokenValue::BuiltinType(DataType::Char),
                    "str" => TokenValue::BuiltinType(DataType::RawString),
                    "void" => TokenValue::BuiltinType(DataType::Void),
                    "bool" => TokenValue::BuiltinType(DataType::Bool),
                    "usize" => TokenValue::BuiltinType(DataType::Usize),
                    "isize" => TokenValue::BuiltinType(DataType::Isize),

                    _ => TokenValue::Value(Value::Symbol(string_tok))
                };
                tokens.push(
                    Rc::new(source),
                    token,
                    ts.base_priority
                );
            },

            '+' => {
                if peek_char!().map(|(_, next_ch)| *next_ch == '=').unwrap_or(false) {
                    skip_next_char!();
                    let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "+=");
                    tokens.push(
                        Rc::new(source),
                        TokenValue::AddAssign,
                        ts.base_priority
                    );
                } else {
                    let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "+");
                    tokens.push(
                        Rc::new(source),
                        TokenValue::Add,
                        ts.base_priority
                    );
                }
            },

            '-' => {
                if peek_char!().map(|(_, next_ch)| *next_ch == '>').unwrap_or(false) {
                    skip_next_char!();
                    let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "->");
                    tokens.push(
                        Rc::new(source),
                        TokenValue::Arrow,
                        ts.base_priority
                    );
                } else if peek_char!().map(|(_, next_ch)| *next_ch == '=').unwrap_or(false) {
                    skip_next_char!();
                    let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "-=");
                    tokens.push(
                        Rc::new(source),
                        TokenValue::SubAssign,
                        ts.base_priority
                    );
                } else if !may_be_expression(tokens.iter_lex_tokens_rev().next()) {
                    // Unary minus or part of a numeric token
                    if peek_char!().map(|(_, next_ch)| next_ch.is_ascii_digit()).unwrap_or(false) {
                        // Part of a numeric token
                        token_is_negative_number = Some(get_next_char!().unwrap().0);
                    } else {
                        // Unary minus
                        let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "-");
                        tokens.push(
                            Rc::new(source),
                            TokenValue::UnaryMinus,
                            ts.base_priority
                        );
                    }
                } else {
                    let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "-");
                    tokens.push(
                        Rc::new(source),
                        TokenValue::Sub,
                        ts.base_priority
                    );
                }
            },

            '*' => {
                if peek_char!().map(|(_, next_ch)| *next_ch == '*').unwrap_or(false) {
                    skip_next_char!();
                    let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "*=");
                    tokens.push(
                        Rc::new(source),
                        TokenValue::MulAssign,
                        ts.base_priority
                    );
                } else if may_be_expression(tokens.iter_lex_tokens_rev().next()) {
                    let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "*");
                    tokens.push(
                        Rc::new(source),
                        TokenValue::Mul,
                        ts.base_priority
                    );
                } else {
                    // Previous token is not an expression, so this is a unary dereference operator.
                    let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "*");
                    tokens.push(
                        Rc::new(source),
                        TokenValue::Deref,
                        ts.base_priority
                    );
                }
            },

            '&' => {
                if peek_char!().map(|(_, next_ch)| *next_ch == '&').unwrap_or(false) {
                    skip_next_char!();
                    let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "&&");
                    tokens.push(
                        Rc::new(source),
                        TokenValue::LogicalAnd,
                        ts.base_priority
                    );
                } else if !may_be_expression(tokens.iter_lex_tokens_rev().next()) {
                    let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "&");
                    tokens.push(
                        Rc::new(source),
                        TokenValue::TakeRef,
                        ts.base_priority
                    );
                } else {
                    // Previous token is not an expression, so this is a unary dereference operator.
                    let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "&");
                    tokens.push(
                        Rc::new(source),
                        TokenValue::BitAnd,
                        ts.base_priority
                    );
                }
            },

            '%' => {
                if peek_char!().map(|(_, next_ch)| *next_ch == '=').unwrap_or(false) {
                    skip_next_char!();
                    let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "%=");
                    tokens.push(
                        Rc::new(source),
                        TokenValue::ModAssign,
                        ts.base_priority
                    );
                } else {
                    let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "%");
                    tokens.push(
                        Rc::new(source),
                        TokenValue::Mod,
                        ts.base_priority
                    );
                }
            },

            '!' => {
                if peek_char!().map(|(_, next_ch)| *next_ch == '=').unwrap_or(false) {
                    skip_next_char!();
                    let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "!=");
                    tokens.push(
                        Rc::new(source),
                        TokenValue::NotEqual,
                        ts.base_priority
                    );
                } else {
                    let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "!");
                    tokens.push(
                        Rc::new(source),
                        TokenValue::LogicalNot,
                        ts.base_priority
                    );
                }
            },

            '|' => {
                if peek_char!().map(|(_, next_ch)| *next_ch == '|').unwrap_or(false) {
                    skip_next_char!();
                    let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "||");
                    tokens.push(
                        Rc::new(source),
                        TokenValue::LogicalOr,
                        ts.base_priority
                    );
                } else {
                    let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "|");
                    tokens.push(
                        Rc::new(source),
                        TokenValue::BitOr,
                        ts.base_priority
                    );
                }
            },

            '<' => {
                if peek_char!().map(|(_, next_ch)| *next_ch == '<').unwrap_or(false) {
                    skip_next_char!();
                    let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "<<");
                    tokens.push(
                        Rc::new(source),
                        TokenValue::BitShiftLeft,
                        ts.base_priority
                    );
                } else if peek_char!().map(|(_, next_ch)| *next_ch == '=').unwrap_or(false) {
                    skip_next_char!();
                    let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "<=");
                    tokens.push(
                        Rc::new(source),
                        TokenValue::LessEqual,
                        ts.base_priority
                    );
                } else {
                    let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "<");
                    tokens.push(
                        Rc::new(source),
                        TokenValue::Less,
                        ts.base_priority
                    );
                }
            },

            '>' => {
                if peek_char!().map(|(_, next_ch)| *next_ch == '>').unwrap_or(false) {
                    skip_next_char!();
                    let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), ">>");
                    tokens.push(
                        Rc::new(source),
                        TokenValue::BitShiftRight,
                        ts.base_priority
                    );
                } else if peek_char!().map(|(_, next_ch)| *next_ch == '=').unwrap_or(false) {
                    skip_next_char!();
                    let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), ">=");
                    tokens.push(
                        Rc::new(source),
                        TokenValue::GreaterEqual,
                        ts.base_priority
                    );
                } else {
                    let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), ">");
                    tokens.push(
                        Rc::new(source),
                        TokenValue::Greater,
                        ts.base_priority
                    );
                }
            },

            ',' => {
                let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), ",");
                tokens.push(
                    Rc::new(source),
                    TokenValue::Comma,
                    ts.base_priority
                );
            },

            ':' => {
                let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), ":");
                tokens.push(
                    Rc::new(source),
                    TokenValue::Colon,
                    ts.base_priority
                );
            },

            '~' => {
                let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "~");
                tokens.push(
                    Rc::new(source),
                    TokenValue::BitNot,
                    ts.base_priority
                );
            },

            '^' => {
                let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "^");
                tokens.push(
                    Rc::new(source),
                    TokenValue::BitXor,
                    ts.base_priority
                );
            },

            '/' => {
                let next = peek_char!().map(|(_, next)| *next);
                if next.map(|next| next == '/').unwrap_or(false) {
                    skip_next_char!();
                    // Inline comment
                    // Skip all characters until newline and register the newline
                    while let Some((i, next)) = get_next_char!() {
                        if next == '\n' {
                            ts.newline(i);
                            break;
                        }
                    }
                } else if next.map(|next| next == '=').unwrap_or(false) {
                    skip_next_char!();
                    let source = SourceToken::new(
                        ts.line_index,
                        ts.column_index(base_char_index),
                        "/="
                    );
                    tokens.push(
                        Rc::new(source),
                        TokenValue::DivAssign,
                        ts.base_priority
                    );
                } else if next.map(|next| next == '*').unwrap_or(false) {
                    skip_next_char!();
                    // Mutliline comment
                    let mut found_astherix = false;
                    let mut comment_correctly_terminated = false;
                    let start_line_index = ts.line_index;
                    let start_column_index = ts.column_index(base_char_index);
                    while let Some((i, next)) = get_next_char!() {

                        if next == '\n' {
                            ts.newline(i);
                            found_astherix = false;
                        } else if found_astherix {
                            if next == '/' {
                                // end
                                comment_correctly_terminated = true;
                                break;
                            } else if next != '*' {
                                found_astherix = false;
                            }
                        } else if next == '*' {
                            found_astherix = true;
                        }
                    }
                    if !comment_correctly_terminated {
                        let source = SourceToken::new(start_line_index, start_column_index, "/*");
                        phase_errors.push(CompilationError {
                            source: Rc::new(source),
                            kind: ErrorKind::UnmatchedDelimiter { delimiter: "/*" },
                            hint: "Multiline comments must be terminated with `*/`"
                        });
                    }
                } else {
                    let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "/");
                    tokens.push(
                        Rc::new(source),
                        TokenValue::Div,
                        ts.base_priority
                    );
                }
            },

            '=' => {
                if peek_char!().map(|(_, next)| *next == '=').unwrap_or(false) {
                    skip_next_char!();
                    let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "==");
                    tokens.push(
                        Rc::new(source),
                        TokenValue::Equal,
                        ts.base_priority
                    );
                } else {
                    let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "=");
                    tokens.push(
                        Rc::new(source),
                        TokenValue::Assign,
                        ts.base_priority
                    );
                }
            },

            '"' => {
                let start_line_index = ts.line_index;
                let start_column_index = ts.column_index(base_char_index);
                let mut found_closing_quote = false;
                let mut status = StringStatus::Raw;
                while let Some((string_char_index, string_ch)) = get_next_char!() {
                    match &mut status {
                        StringStatus::Raw => {
                            // Copy the string until the escape character
                            if string_ch == '\\' {
                                status = StringStatus::Escaped {
                                    string: String::from_str(unsafe {
                                            std::str::from_raw_parts(
                                                raw_source.as_ptr().byte_add(base_char_index + SIZE_OF_QUOTE_CHAR),
                                                string_char_index - (base_char_index + SIZE_OF_ESCAPE_CHAR)
                                            )
                                        }).expect("Infallible"),
                                    escape_on: true
                                };
                            } else if string_ch == '"' {
                                let source = SourceToken::new(
                                    start_line_index,
                                    start_column_index,
                                    unsafe {
                                        std::str::from_raw_parts(
                                            raw_source.as_ptr().byte_add(base_char_index),
                                            string_char_index + SIZE_OF_QUOTE_CHAR - base_char_index
                                        ) }
                                );
                                tokens.push(
                                    Rc::new(source),
                                    TokenValue::Value(Value::Literal(LiteralValue::StaticString(Cow::Borrowed(unsafe {
                                        std::str::from_raw_parts( // Like the whole string token, but excluding the delimiting quotes
                                            raw_source.as_ptr().byte_add(base_char_index + SIZE_OF_QUOTE_CHAR),
                                            string_char_index - (base_char_index + SIZE_OF_QUOTE_CHAR)
                                        ) })))),
                                    ts.base_priority
                                );
                                found_closing_quote = true;
                                break;
                            }
                        },
                        StringStatus::Escaped { string, escape_on } => {
                            let char_value =
                                if *escape_on {
                                    *escape_on = false;
                                    let escaped_char = match string_ch {
                                        't' => Ok('\t'),
                                        'n' => Ok('\n'),
                                        'r' => Ok('\r'),
                                        '0' => Ok('\0'),
                                        '"' => Ok('"'),
                                        _ => Err(())
                                    };
                                    if let Ok(escaped_char) = escaped_char {
                                        escaped_char
                                    } else {
                                        let source = SourceToken::new(ts.line_index, ts.column_index(string_char_index), "");
                                        phase_errors.push(CompilationError {
                                            source: Rc::new(source),
                                            kind: ErrorKind::InvalidEscapeSequence { invalid_character: Some(string_ch) },
                                            hint: "Valid esape sequences in string literals are: '\\t', '\\n', '\\r', '\\0', '\"'"
                                        });
                                        '\\'
                                    }
                                } else if string_ch == '"' {
                                    let source = SourceToken::new(
                                        start_line_index,
                                        start_column_index,
                                        unsafe {
                                            std::str::from_raw_parts(
                                                raw_source.as_ptr().byte_add(base_char_index),
                                                string_char_index + SIZE_OF_QUOTE_CHAR - base_char_index
                                            )
                                        }
                                    );
                                    tokens.push(
                                        Rc::new(source),
                                        TokenValue::Value(Value::Literal(LiteralValue::StaticString(Cow::Owned(mem::take(string))))),
                                        ts.base_priority
                                    );
                                    found_closing_quote = true;
                                    break;
                                } else {
                                    string_ch
                                };
                            string.push(char_value);
                        },
                    }
                }

                if !found_closing_quote {
                    let source = SourceToken::new(start_line_index, start_column_index, "");
                    phase_errors.push(CompilationError {
                        source: Rc::new(source),
                        kind: ErrorKind::UnmatchedDelimiter { delimiter: "\"" },
                        hint: "String literal must be enclosed in double quotes. Did you forget a closing '\"'?"
                    });
                }
            },

            '\'' => {
                let start_line_index = ts.line_index;
                let start_column_index = ts.column_index(base_char_index);

                let Some((i_char_value, char_value)) = get_next_char!() else {
                    let source = SourceToken::new(start_line_index, start_column_index, "");
                    phase_errors.push(CompilationError {
                        source: Rc::new(source), // TODO: does this need to be an Rc?
                        kind: ErrorKind::UnmatchedDelimiter { delimiter: "'" },
                        hint: "Invalid character literal"
                    });
                    continue;
                };

                let char_value =
                    if char_value == '\\' {
                        if let Some((i, char_to_escape)) = get_next_char!() {
                            let escaped_char = match char_to_escape {
                                't' => Ok('\t'),
                                'n' => Ok('\n'),
                                'r' => Ok('\r'),
                                '0' => Ok('\0'),
                                '"' => Ok('"'),
                                _ => Err(())
                            };
                            if let Ok(escaped_char) = escaped_char {
                                escaped_char
                            } else {
                                let source = SourceToken::new(ts.line_index, ts.column_index(i), "");
                                phase_errors.push(CompilationError {
                                    source: Rc::new(source),
                                    kind: ErrorKind::InvalidEscapeSequence { invalid_character: Some(char_to_escape) },
                                    hint: "Valid esape sequences in character literals are: '\\t', '\\n', '\\r', '\\0', '\\''"
                                });
                                '\\'
                            }
                        } else {
                            let source = SourceToken::new(ts.line_index, ts.column_index(i_char_value), "");
                            phase_errors.push(CompilationError {
                                source: Rc::new(source),
                                kind: ErrorKind::InvalidEscapeSequence { invalid_character: None },
                                hint: "Character to escape not specified"
                            });
                            // Return a valid char value to continue checking
                            '\\'
                        }
                    } else {
                        char_value
                    };

                let Some((i_closing_delimiter, closing_delimiter)) = get_next_char!() else {
                    let source = SourceToken::new(ts.line_index, ts.column_index(i_char_value), "");
                    phase_errors.push(CompilationError {
                        source: Rc::new(source),
                        kind: ErrorKind::UnmatchedDelimiter { delimiter: "'" },
                        hint: "Character literals must be enclosed in single quotes. Did you forget a closing `'`?"
                    });
                    continue;
                };
                if closing_delimiter == '\'' {
                    let token_str = unsafe {
                        std::str::from_raw_parts(
                            raw_source.as_ptr().byte_add(base_char_index),
                            i_closing_delimiter + SIZE_OF_QUOTE_CHAR - base_char_index
                        )
                    };
                    let source = SourceToken::new(start_line_index, start_column_index, token_str);
                    tokens.push(
                        Rc::new(source),
                        TokenValue::Value(Value::Literal(LiteralValue::Char(char_value))),
                        ts.base_priority
                    );
                } else {
                    let mut found_closing_quote = false;
                    let mut length: usize = 1;
                    while let Some((_ ,next_ch)) = get_next_char!() {
                        if next_ch == '\'' {
                            found_closing_quote = true;
                            break;
                        } else {
                            length += 1;
                        }
                    }
                    let source = SourceToken::new(start_line_index, start_column_index, "");
                    if found_closing_quote {
                        phase_errors.push(CompilationError {
                            source: Rc::new(source),
                            kind: ErrorKind::CharLiteralTooLong { length },
                            hint: "Character literals must be exactly one character long. Did you forget a closing `'`?"
                        });
                    } else {
                        phase_errors.push(CompilationError {
                            source: Rc::new(source),
                            kind: ErrorKind::UnmatchedDelimiter { delimiter: "'" },
                            hint: "Character literals must be enclosed in single quotes. Did you forget a closing `'`?"
                        });
                    }
                }
            },

            ';' => {
                let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), ";");
                tokens.push(
                    Rc::new(source),
                    TokenValue::Semicolon,
                    ts.base_priority
                );
            },

            '(' => {
                let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "(");

                let token_kind = {
                    let mut iter = tokens.iter_lex_tokens_rev();
                    let last = iter.next();

                    if may_be_expression(last) {

                        // Unwrap is safe because `last` is an expression
                        let last_token = last.unwrap();

                        if matches!(last_token.value, TokenValue::Value(Value::Symbol { .. })) && iter.next().map(|token| matches!(token.value, TokenValue::Fn)).unwrap_or(false) {
                            // Syntax: fn <symbol> (
                            TokenValue::FunctionParamsOpen
                        } else {
                            // Syntax: <value-like> (
                            TokenValue::FunctionCallOpen
                        }
                    } else {
                        // Syntax: <not-a-value> (
                        TokenValue::ParOpen
                    }
                };

                tokens.push(
                    Rc::new(source),
                    token_kind,
                    ts.base_priority
                );
                ts.enter_parenthesis();
            },

            ')' => {
                let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), ")");

                if ts.leave_parenthesis().is_err() {
                    phase_errors.push(CompilationError {
                        source: Rc::new(source),
                        kind: ErrorKind::UnmatchedDelimiter { delimiter: ")" },
                        hint: "Unexpected closing delimiter. Did you forget a '('?"
                    });
                } else {
                    tokens.push(
                        Rc::new(source),
                        TokenValue::ParClose,
                        ts.base_priority
                    );
                }
            },

            '{' => {
                let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "{");
                tokens.push(
                    Rc::new(source),
                    TokenValue::ScopeOpen,
                    ts.base_priority
                );
                ts.enter_curly();
            },

            '}' => {
                let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "}");

                if ts.leave_curly().is_err() {
                    phase_errors.push(CompilationError {
                        source: Rc::new(source),
                        kind: ErrorKind::UnmatchedDelimiter { delimiter: "}" },
                        hint: "Unexpected closing delimiter. Did you forget a '{'?"
                    });
                } else {
                    tokens.push(
                        Rc::new(source),
                        TokenValue::ScopeClose,
                        ts.base_priority
                    );
                }
            },

            '[' => {
                let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "[");

                let token_kind = {
                    if may_be_expression(tokens.iter_lex_tokens_rev().next()) {
                        TokenValue::ArrayIndexOpen
                    } else {
                        TokenValue::ArrayOpen
                    }
                };

                tokens.push(
                    Rc::new(source),
                    token_kind,
                    ts.base_priority
                );
                ts.enter_square();
            },

            ']' => {
                let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "]");

                if ts.leave_square().is_err() {
                    phase_errors.push(CompilationError {
                        source: Rc::new(source),
                        kind: ErrorKind::UnmatchedDelimiter { delimiter: "]" },
                        hint: "Unexpected closing delimiter. Did you forget a '['?"
                    });
                } else {
                    tokens.push(
                        Rc::new(source),
                        TokenValue::SquareClose,
                        ts.base_priority
                    );
                }
            },

            ' ' | '\t' | '\n' => {
                // Ignore these characters
            },

            _ => {
                let source = SourceToken::new(ts.line_index, ts.column_index(base_char_index), "");
                phase_errors.push(CompilationError {
                    source: Rc::new(source),
                    kind: ErrorKind::UnexpectedCharacter { ch },
                    hint: "This character was not expected in this context"
                });
            }
        }
    }

    if !phase_errors.is_empty() {
        print_errors_and_exit(CompilationPhases::Tokenization, &phase_errors, module);
    }

    tokens
}

//             "[" => {
//                 ts.enter_square();

//                 if let Some(last_token) = tokens.last_token() {
//                     if is_data_type_precursor(last_token) {
//                         // Syntax: <data-type-precursor> [
//                         TokenKind::ArrayTypeOpen
//                     } else if may_be_expression(Some(last_token)) {
//                         // Syntax: <possible-array-type> [
//                         TokenKind::ArrayIndexOpen
//                     } else {
//                         // Syntax: <not-a-data-type-precursor> [
//                         TokenKind::ArrayOpen
//                     }
//                 } else {
//                     // Syntax: <nothing> [
//                     TokenKind::ArrayOpen
//                 }
//             },
//
