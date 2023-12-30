use std::fmt::{Display, Debug};

use self::dt_macros::numeric_pattern;


#[derive(Clone, PartialEq)]
pub enum DataType {

    Bool,

    Char,
    String,

    Array (Box<DataType>),
    Ref (Box<DataType>),

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

    Function { params: Vec<DataType>, return_type: Box<DataType> },

    Void

}

/// Useful macros for working with data types.
#[macro_use]
pub mod dt_macros {

    macro_rules! floating_point_pattern {
        () => {
            DataType::F32 | DataType::F64
        };
    }
    pub(crate) use floating_point_pattern;

    macro_rules! unsigned_integer_pattern {
        () => {
            DataType::U8 | DataType::U16 | DataType::U32 | DataType::U64
        };
    }
    pub(crate) use unsigned_integer_pattern;


    macro_rules! signed_integer_pattern {
        () => {
            DataType::I8 | DataType::I16 | DataType::I32 | DataType::I64
        };
    }
    pub(crate) use signed_integer_pattern;

    macro_rules! integer_pattern {
        () => {
            signed_integer_pattern!() | unsigned_integer_pattern!()
        };
    }
    pub(crate) use integer_pattern;

    macro_rules! numeric_pattern {
        () => {
            integer_pattern!() | floating_point_pattern!()
        };
    }
    pub(crate) use numeric_pattern;

    macro_rules! pointer_pattern {
        () => {
            DataType::Ref(_)
        };
    }
    pub(crate) use pointer_pattern;

}

impl DataType {

    pub fn is_castable_to(&self, target: &DataType) -> bool {
        // A data type is always castable to itself.
        if self == target {
            return true;
        }

        match self {

            // 1-byte integers are castable to chars.
            DataType::I8 | DataType::U8 => matches!(target, DataType::Char),            
            // A number is castable to any other number.
            #[allow(unreachable_patterns)] // i8 and u8 types overlap with numeric types
            numeric_pattern!() => matches!(target, numeric_pattern!()),
            // A char is castable to 1-byte integers.
            DataType::Char => matches!(target, DataType::U8 | DataType::I8),
            // A reference is castable to any other reference.
            DataType::Ref(_) => matches!(target, DataType::Ref(_)),

            // Other type casts are not allowed.
            _ => false
        }
    }

    pub fn is_implicitly_castable_to(&self, target: &DataType) -> bool {
        // A data type is always castable to itself.
        if self == target {
            return true;
        }

        match self {

            // An empty array can always be cast to another array type.
            DataType::Array(element_type) => matches!(**element_type, DataType::Void) && matches!(target, DataType::Array(_)),
        
            _ => false
        }
    }

    /// May return leaked strings, but it's ok because this function is only used before the program exits to print errors.
    pub fn name(&self) -> &str {
        match self {
            DataType::Bool => "bool",
            DataType::Char => "char",
            DataType::String => "str",
            DataType::Array(x) => Box::new(format!("[{}]", x)).leak(),
            DataType::Ref(x) => Box::new(format!("&{}", x)).leak(),
            DataType::I8 => "i8",
            DataType::I16 => "i16",
            DataType::I32 => "i32",
            DataType::I64 => "i64",
            DataType::U8 => "u8",
            DataType::U16 => "u16",
            DataType::U32 => "u32",
            DataType::U64 => "u64",
            DataType::F32 => "f32",
            DataType::F64 => "f64",
            DataType::Function { params, return_type } => {
                let mut name = String::from("fn(");
                for (i, param) in params.iter().enumerate() {
                    name.push_str(param.name());
                    if i < params.len() - 1 {
                        name.push_str(", ");
                    }
                }
                name.push_str(") -> ");
                name.push_str(return_type.name());
                Box::leak(name.into_boxed_str())
            },
            DataType::Void => "void"
        }
    }

}


impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl Debug for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

