use std::rc::Rc;
use std::borrow::Cow;

use super::DataType;


#[derive(Debug)]
pub enum LiteralValue<'a> {

    Char (char),
    StaticString (Cow<'a, str>),
    Array { element_type: Rc<DataType>, elements: Box<[LiteralValue<'a>]> },
    Number (Number),
    Bool (bool),
    StaticRef (Box<LiteralValue<'a>>)

}


#[derive(Debug, Clone)]
pub enum Number {

    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64)

}
