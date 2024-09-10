use std::rc::Rc;

use rusty_vm_lib::vm::ADDRESS_SIZE;


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


pub enum DataType {
    Bool,
    Char,
    Array { element_type: Rc<DataType>, length: Option<usize> },
    Slice,
    StringRef,
    // RawString, ??
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


pub struct FunctionSignature {
    pub params: Box<[Rc<DataType>]>,
    pub return_type: Rc<DataType>
}
