use std::rc::Rc;

use crate::symbol_table::StaticID;

use super::DataType;


pub enum LiteralValue {

    Char (char),
    StaticString (StaticID),
    Array { element_type: Rc<DataType>, elements: Box<[LiteralValue]> },
    Number (Number),
    Bool (bool),
    StaticRef (Box<LiteralValue>)

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
