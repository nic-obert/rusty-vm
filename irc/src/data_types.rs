use std::fmt::Display;

use self::dt_macros::numeric;


#[derive(Debug, Clone, PartialEq)]
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

    /// Combine multiple comma-separated types into a matchable pattern.
    macro_rules! types {
        ($($t:path),*) => {
            $($t)|*
        };
    }
    pub(crate) use types;

    macro_rules! floating_point {
        () => {
            types!(
                DataType::F32, DataType::F64
            )
        };
    }
    pub(crate) use floating_point;

    macro_rules! unsigned_integer {
        () => {
            types!(
                DataType::U8, DataType::U16, DataType::U32, DataType::U64
            )
        };
    }
    pub(crate) use unsigned_integer;

    macro_rules! signed_integer {
        () => {
            types!(
                DataType::I8, DataType::I16, DataType::I32, DataType::I64
            )
        };
    }
    pub(crate) use signed_integer;

    macro_rules! integer {
        () => {
            signed_integer!() | unsigned_integer!()
        };
    }
    pub(crate) use integer;

    macro_rules! numeric {
        () => {
            integer!() | floating_point!()
        };
    }
    pub(crate) use numeric;

    macro_rules! pointer {
        () => {
            DataType::Ref(_)
        };
    }
    pub(crate) use pointer;

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
            numeric!() => matches!(target, numeric!()),
            // A char is castable to 1-byte integers.
            DataType::Char => matches!(target, DataType::U8 | DataType::I8),
            // A reference is castable to any other reference.
            DataType::Ref(_) => matches!(target, DataType::Ref(_)),

            // Other type casts are not allowed.
            _ => false
        }
    }

}


impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Char => write!(f, "char"),
            DataType::String => write!(f, "str"),
            DataType::Array(dt) => write!(f, "[{}]", dt),
            DataType::Ref(dt) => write!(f, "&{}", dt),
            DataType::I8 => write!(f, "i8"),
            DataType::I16 => write!(f, "i16"),
            DataType::I32 => write!(f, "i32"),
            DataType::I64 => write!(f, "i64"),
            DataType::U8 => write!(f, "u8"),
            DataType::U16 => write!(f, "u16"),
            DataType::U32 => write!(f, "u32"),
            DataType::U64 => write!(f, "u64"),
            DataType::F32 => write!(f, "f32"),
            DataType::F64 => write!(f, "f64"),
            DataType::Void => write!(f, "void"),
            DataType::Function { params, return_type } => {
                write!(f, "fn(")?;
                for (i, param) in params.iter().enumerate() {
                    write!(f, "{}", param)?;
                    if i < params.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ") -> {}", return_type)
            },
            DataType::Bool => write!(f, "bool"),
            
        }
    }
}

