use std::rc::Rc;

use rusty_vm_lib::vm::ADDRESS_SIZE;


pub const VOID_SIZE: usize = 0;
pub const BOOL_SIZE: usize = 1;
pub const CHAR_SIZE: usize = 1;
pub const I8_SIZE: usize = 1;
pub const I16_SIZE: usize = 2;
pub const I32_SIZE: usize = 4;
pub const I64_SIZE: usize = 8;
pub const U8_SIZE: usize = 1;
pub const U16_SIZE: usize = 2;
pub const U32_SIZE: usize = 4;
pub const U64_SIZE: usize = 8;
pub const F32_SIZE: usize = 4;
pub const F64_SIZE: usize = 8;
pub const USIZE_SIZE: usize = ADDRESS_SIZE;
pub const ISIZE_SIZE: usize = ADDRESS_SIZE;
pub const WIDE_POINTER_SIZE: usize = ADDRESS_SIZE + USIZE_SIZE;


#[derive(Debug)]
pub enum DataType {
    Bool,
    Char,
    Array { element_type: Rc<DataType>, length: usize },
    Slice { element_type: Rc<DataType> },
    StringRef,
    RawString,
    Ref,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Usize,
    Isize,
    Void,
    Function { signature: FunctionSignature }
}

impl DataType {

    pub fn static_size(&self) -> Option<usize> {
        match self {
            DataType::Bool => Some(BOOL_SIZE),
            DataType::Char => Some(CHAR_SIZE),
            DataType::Array { element_type, length } => element_type.static_size().map(|size| size * length),
            DataType::Slice { .. } => Some(WIDE_POINTER_SIZE),
            DataType::StringRef => Some(WIDE_POINTER_SIZE),
            DataType::Ref => Some(ADDRESS_SIZE),
            DataType::I8 => Some(I8_SIZE),
            DataType::I16 => Some(I16_SIZE),
            DataType::I32 => Some(I32_SIZE),
            DataType::I64 => Some(I64_SIZE),
            DataType::U8 => Some(U8_SIZE),
            DataType::U16 => Some(U16_SIZE),
            DataType::U32 => Some(U32_SIZE),
            DataType::U64 => Some(U64_SIZE),
            DataType::F32 => Some(F32_SIZE),
            DataType::F64 => Some(F64_SIZE),
            DataType::Usize => Some(USIZE_SIZE),
            DataType::Isize => Some(ISIZE_SIZE),
            DataType::Void => Some(VOID_SIZE),
            DataType::Function { .. } => Some(ADDRESS_SIZE),

            DataType::RawString => None
        }
    }

}


#[derive(Debug)]
pub struct FunctionSignature {
    pub params: Box<[Rc<DataType>]>,
    pub return_type: Rc<DataType>
}


#[derive(Debug)]
pub enum DataTypeName {
    Bool,
    Char,
    RawString,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Usize,
    Isize,
    Void,
}
