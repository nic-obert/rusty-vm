use std::fmt::Display;



#[derive(Debug, Clone, PartialEq)]
pub enum DataType {

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
            
        }
    }
}

