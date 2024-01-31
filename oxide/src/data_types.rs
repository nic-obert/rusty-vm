use std::borrow::Cow;
use std::fmt::{Display, Debug};
use std::rc::Rc;

use crate::{match_unreachable, symbol_table::{StaticID, SymbolTable}};

use self::dt_macros::numeric_pattern;


#[derive(Clone, PartialEq)]
pub enum DataType {

    Bool,

    Char,
    RawString { length: usize },
    String,

    Array (Rc<DataType>),
    Ref { target: Rc<DataType>, mutable: bool },
    StringRef { length: usize },

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

    Function { params: Vec<DataType>, return_type: Rc<DataType> },

    Void,
    
    /// Only used internally for type inference.
    Unspecified
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
            DataType::Ref { .. }
        };
    }
    pub(crate) use pointer_pattern;

}

impl DataType {

    /// Return the size of the data type in bytes, if it is known at compile-time.
    #[allow(dead_code)]
    pub const fn static_size(&self) -> usize {
        match self {
            DataType::Bool => 1,
            DataType::Char => 1,
            DataType::Ref { .. } => rust_vm_lib::vm::ADDRESS_SIZE,
            DataType::I8 => 1,
            DataType::I16 => 2,
            DataType::I32 => 4,
            DataType::I64 => 8,
            DataType::U8 => 1,
            DataType::U16 => 2,
            DataType::U32 => 4,
            DataType::U64 => 8,
            DataType::F32 => 4,
            DataType::F64 => 8,
            DataType::Usize => rust_vm_lib::vm::ADDRESS_SIZE,
            DataType::Isize => rust_vm_lib::vm::ADDRESS_SIZE,
            DataType::RawString { length } => DataType::Char.static_size() * *length, // Char size * number of chars
            DataType::StringRef { length: _ } => rust_vm_lib::vm::ADDRESS_SIZE + DataType::Usize.static_size(), // Address size + length size
            
            DataType::Array(_) |
            DataType::String  // TODO: update with a rust-like string struct size
            => unimplemented!(),
            
            DataType::Function { .. } => unreachable!(),
            DataType::Unspecified => unreachable!(),
            DataType::Void => unreachable!(),
        }
    }


    pub fn is_castable_to(&self, target: &DataType) -> bool {

        self.is_implicitly_castable_to(target, None)
        || match self {

            // 1-byte integers are castable to chars and other numbers
            DataType::I8 | DataType::U8 => matches!(target, DataType::Char | numeric_pattern!()),
            // u64 is castable to pointers (and other numbers)
            DataType::U64 => matches!(target, numeric_pattern!() | DataType::Ref { .. }),    
            // A number is castable to any other number.
            #[allow(unreachable_patterns)]
            numeric_pattern!() => matches!(target, numeric_pattern!()),
            // A char is castable to 1-byte integers.
            DataType::Char => matches!(target, DataType::U8 | DataType::I8),
            // A pointer is castable to any other reference and to u64 (for pointer arithmetic).
            DataType::Ref { .. } => matches!(target, DataType::Ref { .. } | DataType::U64),
            
            DataType::Bool => matches!(target, integer_pattern!()),

            DataType::Array(element_type) => if let DataType::Array(target_element_type) = target {
                // An array is castable to another array if the element type is implicitly castable to the target element type.
                element_type.is_implicitly_castable_to(target_element_type, None)
            } else { false },

            // Other type casts are not allowed.
            _ => false
        }
    }


    /// Return whether `self` is implicitly castable to `target`.
    pub fn is_implicitly_castable_to(&self, target: &DataType, self_value: Option<&LiteralValue>) -> bool {
        // A data type is always implicitly castable to itself
        if self == target {
            return true;
        }

        // Can self be implicitly cast to target?
        match self {

            // String references are interchangeable since they always have the same size (wide pointers).
            DataType::StringRef { length: _ } => matches!(target, DataType::StringRef { length: _ }),

            DataType::Array(element_type)
             => matches!(**element_type, DataType::Void) && matches!(target, DataType::Array(_)) // An empty array can always be cast to another array type.
                || if let DataType::Array(target_element_type) = target {
                    // Check if the element type is implicitly castable to the target element type.
                    element_type.is_implicitly_castable_to(target_element_type, None)
                    // Else, check if all the items in the array are implicitly castable to the target element type.
                    || self_value.map(|value| match value {
                        LiteralValue::Array { items, .. } => items.iter().all(|item| 
                            // Is `element_type` implicitly castable to `target_element_type` if its value is `value`?
                            element_type.is_implicitly_castable_to(target_element_type, Some(item))
                        ),
                        _ => false
                    }).unwrap_or(false)
                } else { false },
    
            // If target type is x, what types can be implicitly cast to x?
            _ => match target {

                DataType::I8 => self_value.map(|value| match value {
                    LiteralValue::Numeric(Number::Int(n)) => *n >= std::i8::MIN as i64 && *n <= std::i8::MAX as i64,
                    LiteralValue::Numeric(Number::Uint(n)) => *n <= std::i8::MAX as u64,
                    _ => false
                }).unwrap_or(false),

                DataType::I16 => matches!(self, DataType::I8 | DataType::U8)
                    || self_value.map(|value| match value {
                        LiteralValue::Numeric(Number::Int(n)) => *n >= std::i16::MIN as i64 && *n <= std::i16::MAX as i64,
                        LiteralValue::Numeric(Number::Uint(n)) => *n <= std::i16::MAX as u64,
                        _ => false
                    }).unwrap_or(false),

                DataType::I32 => matches!(self, DataType::I8 | DataType::U8 | DataType::I16 | DataType::U16)
                    || self_value.map(|value| match value {
                        LiteralValue::Numeric(Number::Int(n)) => *n >= std::i32::MIN as i64 && *n <= std::i32::MAX as i64,
                        LiteralValue::Numeric(Number::Uint(n)) => *n <= std::i32::MAX as u64,
                        _ => false
                    }).unwrap_or(false),

                DataType::I64 => matches!(self, DataType::I8 | DataType::U8 | DataType::I16 | DataType::U16 | DataType::I32 | DataType::U32)
                    || self_value.map(|value| match value {
                        LiteralValue::Numeric(Number::Uint(n)) => *n <= std::i64::MAX as u64,
                        _ => false
                    }).unwrap_or(false),

                DataType::U8 => self_value.map(|value| match value {
                    LiteralValue::Numeric(Number::Int(n)) => *n >= 0 && *n <= std::u8::MAX as i64,
                    LiteralValue::Numeric(Number::Uint(n)) => *n <= std::u8::MAX as u64,
                    _ => false
                }).unwrap_or(false),

                DataType::U16 => matches!(self, DataType::U8)
                    || self_value.map(|value| match value {
                        LiteralValue::Numeric(Number::Int(n)) => *n >= 0 && *n <= std::u16::MAX as i64,
                        LiteralValue::Numeric(Number::Uint(n)) => *n <= std::u16::MAX as u64,
                        _ => false
                    }).unwrap_or(false),

                DataType::U32 => matches!(self, DataType::U8 | DataType::U16)
                    || self_value.map(|value| match value {
                        LiteralValue::Numeric(Number::Int(n)) => *n >= 0 && *n <= std::u32::MAX as i64,
                        LiteralValue::Numeric(Number::Uint(n)) => *n <= std::u32::MAX as u64,
                        _ => false
                    }).unwrap_or(false),

                DataType::U64 => matches!(self, DataType::U8 | DataType::U16 | DataType::U32)
                || self_value.map(|value| match value {
                    LiteralValue::Numeric(Number::Int(n)) => *n >= 0,
                    _ => false
                }).unwrap_or(false),

                DataType::F32 => matches!(self, DataType::I8 | DataType::U8 | DataType::I16 | DataType::U16 | DataType::I32 | DataType::U32 | DataType::I64 | DataType::U64)
                    || self_value.map(|value| match value {
                        LiteralValue::Numeric(Number::Float(n)) => *n >= std::f32::MIN as f64 && *n <= std::f32::MAX as f64,
                        _ => false
                    }).unwrap_or(false),
                
                // All numeric types can be cast to f64 because f64 is the largest numeric type.
                DataType::F64 => matches!(self, DataType::I8 | DataType::U8 | DataType::I16 | DataType::U16 | DataType::I32 | DataType::U32 | DataType::I64 | DataType::U64 | DataType::F32),

                _ => false
            }
        }
    }


    pub fn name(&self) -> Cow<'static, str> { // Uses Cow to avoid leaking owned strings 
        match self {
            DataType::Bool => Cow::Borrowed("bool"),
            DataType::Char => Cow::Borrowed("char"),
            DataType::String => Cow::Borrowed("String"),
            DataType::RawString { length } => Cow::Owned(format!("str[{}]", length)),
            DataType::Array(x) => Cow::Owned(format!("[{}]", x)),
            DataType::Ref { target, mutable } => Cow::Owned(format!("&{}{}", if *mutable { "mut " } else { "" }, target)),
            DataType::I8 => Cow::Borrowed("i8"),
            DataType::I16 => Cow::Borrowed("i16"),
            DataType::I32 => Cow::Borrowed("i32"),
            DataType::I64 => Cow::Borrowed("i64"),
            DataType::U8 => Cow::Borrowed("u8"),
            DataType::U16 => Cow::Borrowed("u16"),
            DataType::U32 => Cow::Borrowed("u32"),
            DataType::U64 => Cow::Borrowed("u64"),
            DataType::F32 => Cow::Borrowed("f32"),
            DataType::F64 => Cow::Borrowed("f64"),
            DataType::Function { params, return_type } => {
                let mut name = String::from("fn(");
                for (i, param) in params.iter().enumerate() {
                    name.push_str(&param.name());
                    if i < params.len() - 1 {
                        name.push_str(", ");
                    }
                }
                name.push_str(") -> ");
                name.push_str(&return_type.name());
                Cow::Owned(name)
            },
            DataType::Void => Cow::Borrowed("void"),
            DataType::Unspecified => Cow::Borrowed("Unspecified"),
            DataType::StringRef { length } => Cow::Owned(format!("str[{}]", length)),
            DataType::Usize => Cow::Borrowed("usize"),
            DataType::Isize => Cow::Borrowed("isize"),
        }
    }

}


impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name())
    }
}

impl Debug for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name())
    }
}


#[derive(Debug, Clone)]
pub enum Number {

    Int(i64),
    Uint(u64),
    Float(f64)

}

macro_rules! impl_binary_numeric_op {
    ($name:ident, $op:tt) => {
        pub fn $name(&self, other: &Number) -> Number {
            match (self, other) {
                (Number::Int(n1), Number::Int(n2)) => Number::Int(n1 $op n2),
                (Number::Uint(n1), Number::Uint(n2)) => Number::Uint(n1 $op n2),
                (Number::Float(n1), Number::Float(n2)) => Number::Float(n1 $op n2),
                _ => unreachable!("Cannot {} different numeric types {:?} and {:?}", stringify!($name), self, other)
            }
        }
    };
}

macro_rules! impl_binary_integer_op {
    ($name:ident, $op:tt) => {
        pub fn $name(&self, other: &Number) -> Number {
            match (self, other) {
                (Number::Int(n1), Number::Int(n2)) => Number::Int(n1 $op n2),
                (Number::Uint(n1), Number::Uint(n2)) => Number::Uint(n1 $op n2),
                _ => unreachable!("Cannot {} non-integer or different numeric types {:?} and {:?}", stringify!($name), self, other)
            }
        }
    };
}

macro_rules! impl_binary_numeric_non_zero_op {
    ($name:ident, $op:tt) => {
        pub fn $name(&self, other: &Number) -> Result<Number, ()> {
            match (self, other) {
                (Number::Int(n1), Number::Int(n2)) => if *n2 == 0 { Err(()) } else { Ok(Number::Int(n1 $op n2)) },
                (Number::Uint(n1), Number::Uint(n2)) => if *n2 == 0 { Err(()) } else { Ok(Number::Uint(n1 $op n2)) },
                (Number::Float(n1), Number::Float(n2)) => if *n2 == 0.0 { Err(()) } else { Ok(Number::Float(n1 $op n2)) },
                _ => unreachable!("Cannot {} different numeric types {:?} and {:?}", stringify!($name), self, other)
            }
        }
    };
}

macro_rules! impl_binary_numeric_cmp {
    ($name:ident, $op:tt) => {
        pub fn $name(&self, other: &Number) -> bool {
            match (self, other) {
                (Number::Int(n1), Number::Int(n2)) => n1 $op n2,
                (Number::Uint(n1), Number::Uint(n2)) => n1 $op n2,
                (Number::Float(n1), Number::Float(n2)) => n1 $op n2,
                _ => unreachable!("Cannot {} different numeric types {:?} and {:?}", stringify!($name), self, other)
            }
        }
    };
}

impl Number {

    // TODO: eventually, add overflow warnings (like different kinds of results)

    impl_binary_numeric_op!(add, +);
    impl_binary_numeric_op!(sub, -);
    impl_binary_numeric_op!(mul, *);
    impl_binary_numeric_non_zero_op!(div, /);
    impl_binary_numeric_non_zero_op!(modulo, %);

    impl_binary_integer_op!(bitwise_and, &);
    impl_binary_integer_op!(bitwise_or, |);
    impl_binary_integer_op!(bitwise_xor, ^);
    impl_binary_integer_op!(bitshift_left, <<);
    impl_binary_integer_op!(bitshift_right, >>);

    impl_binary_numeric_cmp!(greater, >);
    impl_binary_numeric_cmp!(less, <);
    impl_binary_numeric_cmp!(greater_equal, >=);
    impl_binary_numeric_cmp!(less_equal, <=);
    impl_binary_numeric_cmp!(equal, ==);

    pub fn bitwise_not(&self) -> Number {
        match self {
            Number::Int(n) => Number::Int(!n),
            Number::Uint(n) => Number::Uint(!n),
            _ => unreachable!("Cannot bitwise not non-integer numeric type {:?}", self)
        }
    }

}


impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Number::Int(n) => write!(f, "{}", n),
            Number::Uint(n) => write!(f, "{}", n),
            Number::Float(n) => write!(f, "{}", n),
        }
    }
}


#[derive(Debug, Clone)]
pub enum LiteralValue {

    Char (char),
    StaticString (StaticID),

    Array { element_type: Rc<DataType>, items: Vec<LiteralValue> },

    Numeric (Number),

    Bool (bool),

}


impl LiteralValue {

    /// Assumes that the source value is castable to the target type. This should have been checked during type resolution.
    /// 
    /// Assumes that the target type is not the source type.
    /// 
    /// This function can perform only compile-time casts.
    pub fn from_cast(src_value: LiteralValue, src_type: &DataType, target_type: &DataType) -> Self {
        
        assert!(src_type.is_castable_to(target_type));
        assert_ne!(src_type, target_type);

        match src_type {

            DataType::Bool => {
                let value = match_unreachable!(LiteralValue::Bool(value) = src_value, value);
                match target_type {
                    unsigned_integer_pattern!() => LiteralValue::Numeric(Number::Uint(value as u64)),
                    signed_integer_pattern!() => LiteralValue::Numeric(Number::Int(value as i64)),
                    _ => unreachable!("Cannot cast from {:?} to {:?}", src_type, target_type)
                }
            },

            DataType::Char => {
                let ch = match_unreachable!(LiteralValue::Char(ch) = src_value, ch);
                match target_type {
                    DataType::I8 => LiteralValue::Numeric(Number::Int(ch as i64)),
                    DataType::U8 => LiteralValue::Numeric(Number::Uint(ch as u64)),
                    _ => unreachable!("Cannot cast from {:?} to {:?}", src_type, target_type)
                }
            },

            unsigned_integer_pattern!() => {
                let value = match_unreachable!(LiteralValue::Numeric(Number::Uint(value)) = src_value, value);
                match target_type {
                    DataType::Bool => LiteralValue::Bool(value != 0),
                    DataType::Char => LiteralValue::Char(value as u8 as char),
                    DataType::I8 => LiteralValue::Numeric(Number::Int(value as i64)),
                    DataType::I16 => LiteralValue::Numeric(Number::Int(value as i64)),
                    DataType::I32 => LiteralValue::Numeric(Number::Int(value as i64)),
                    DataType::I64 => LiteralValue::Numeric(Number::Int(value as i64)),
                    DataType::U8 => LiteralValue::Numeric(Number::Uint(value)),
                    DataType::U16 => LiteralValue::Numeric(Number::Uint(value)),
                    DataType::U32 => LiteralValue::Numeric(Number::Uint(value)),
                    DataType::U64 => LiteralValue::Numeric(Number::Uint(value)),
                    DataType::F32 => LiteralValue::Numeric(Number::Float(value as f64)),
                    DataType::F64 => LiteralValue::Numeric(Number::Float(value as f64)),
                    _ => unreachable!("Cannot cast from {:?} to {:?}", src_type, target_type)
                }
            },

            signed_integer_pattern!() => {
                let value = match_unreachable!(LiteralValue::Numeric(Number::Int(value)) = src_value, value);
                match target_type {
                    DataType::Bool => LiteralValue::Bool(value != 0),
                    DataType::Char => LiteralValue::Char(value as u8 as char),
                    DataType::I8 => LiteralValue::Numeric(Number::Int(value)),
                    DataType::I16 => LiteralValue::Numeric(Number::Int(value)),
                    DataType::I32 => LiteralValue::Numeric(Number::Int(value)),
                    DataType::I64 => LiteralValue::Numeric(Number::Int(value)),
                    DataType::U8 => LiteralValue::Numeric(Number::Uint(value as u64)),
                    DataType::U16 => LiteralValue::Numeric(Number::Uint(value as u64)),
                    DataType::U32 => LiteralValue::Numeric(Number::Uint(value as u64)),
                    DataType::U64 => LiteralValue::Numeric(Number::Uint(value as u64)),
                    DataType::F32 => LiteralValue::Numeric(Number::Float(value as f64)),
                    DataType::F64 => LiteralValue::Numeric(Number::Float(value as f64)),
                    _ => unreachable!("Cannot cast from {:?} to {:?}", src_type, target_type)
                }
            },
            
            floating_point_pattern!() => {
                let value = match_unreachable!(LiteralValue::Numeric(Number::Float(value)) = src_value, value);
                match target_type {
                    DataType::Bool => LiteralValue::Bool(value != 0.0),
                    DataType::Char => LiteralValue::Char(value as u8 as char),
                    DataType::I8 => LiteralValue::Numeric(Number::Int(value as i64)),
                    DataType::I16 => LiteralValue::Numeric(Number::Int(value as i64)),
                    DataType::I32 => LiteralValue::Numeric(Number::Int(value as i64)),
                    DataType::I64 => LiteralValue::Numeric(Number::Int(value as i64)),
                    DataType::U8 => LiteralValue::Numeric(Number::Uint(value as u64)),
                    DataType::U16 => LiteralValue::Numeric(Number::Uint(value as u64)),
                    DataType::U32 => LiteralValue::Numeric(Number::Uint(value as u64)),
                    DataType::U64 => LiteralValue::Numeric(Number::Uint(value as u64)),
                    DataType::F32 => LiteralValue::Numeric(Number::Float(value as f64)),
                    DataType::F64 => LiteralValue::Numeric(Number::Float(value as f64)),
                    _ => unreachable!("Cannot cast from {:?} to {:?}", src_type, target_type)
                }
            },
            
            DataType::Array(src_element_type) => {
                let (target_element_type, items) = match_unreachable!(LiteralValue::Array { element_type: target_element_type, items } = src_value, (target_element_type, items));
                
                let res = items.into_iter().map(
                    |item| LiteralValue::from_cast(item, src_element_type, &target_element_type)
                ).collect();
                
                LiteralValue::Array { element_type: target_element_type, items: res }
            },
            
            _ => unreachable!("Cannot cast from {:?} to {:?} (or at least not at compile-time)", src_type, target_type)
        }
    }


    pub fn equal(&self, other: &LiteralValue) -> bool {
        match (self, other) {
            (LiteralValue::Char(c1), LiteralValue::Char(c2)) => c1 == c2,
            (LiteralValue::StaticString(s1), LiteralValue::StaticString(s2)) => s1 == s2,
            (LiteralValue::Array { element_type: dt1, items: items1 }, LiteralValue::Array { element_type: dt2, items: items2 }) => dt1 == dt2 && items1.len() == items2.len() && items1.iter().zip(items2.iter()).all(|(item1, item2)| item1.equal(item2)),
            (LiteralValue::Numeric(n1), LiteralValue::Numeric(n2)) => n1.equal(n2),
            (LiteralValue::Bool(b1), LiteralValue::Bool(b2)) => b1 == b2,
            _ => false
        }
    }


    pub fn data_type(&self, symbol_table: &SymbolTable<'_>) -> DataType {
        match self {
            LiteralValue::Char(_) => DataType::Char,
            LiteralValue::StaticString(id) => DataType::StringRef { length: symbol_table.get_static_string(*id).len() },
            LiteralValue::Array { element_type: dt, .. } => DataType::Array(dt.clone()),
            LiteralValue::Numeric(n) => match n {
                // Use a default 32-bit type for numbers. If the number is too big, use a 64-bit type.
                Number::Int(i) => if *i > std::i32::MAX as i64 || *i < std::i32::MIN as i64 { DataType::I64 } else { DataType::I32 },
                Number::Uint(u) => if *u > std::u32::MAX as u64 { DataType::U64 } else { DataType::U32 },
                Number::Float(f) => if *f > std::f32::MAX as f64 || *f < std::f32::MIN as f64 { DataType::F64 } else { DataType::F32 },
            },
            LiteralValue::Bool(_) => DataType::Bool,
        }
    }

}


impl Display for LiteralValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LiteralValue::Char(c) => write!(f, "'{}'", c),
            LiteralValue::StaticString(s) => write!(f, "\"StaticString({:?})\"", s),
            LiteralValue::Array { element_type: dt, items } => write!(f, "[{}]: [{:?}]", dt, items),
            LiteralValue::Numeric(n) => write!(f, "{}", n),
            LiteralValue::Bool(b) => write!(f, "{}", b),
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::data_types::{LiteralValue, Number};
    use std::rc::Rc;

    use super::DataType;

    macro_rules! assert_implicitly_castable {
        ($a:expr, $b:expr) => {
            assert!(DataType::is_implicitly_castable_to(&$a, &$b, None))
        };
    }

    macro_rules! assert_not_implicitly_castable {
        ($a:expr, $b:expr) => {
            assert!(!DataType::is_implicitly_castable_to(&$a, &$b, None))
        };
    }

    #[test]
    fn implicit_casts() {
        // Assert castable to larger types
        assert_implicitly_castable!(DataType::I8, DataType::I16);
        assert_implicitly_castable!(DataType::I8, DataType::I32);
        assert_implicitly_castable!(DataType::I8, DataType::I64);
        assert_implicitly_castable!(DataType::I16, DataType::I32);
        assert_implicitly_castable!(DataType::I16, DataType::I64);
        assert_implicitly_castable!(DataType::I32, DataType::I64);

        assert_implicitly_castable!(DataType::U8, DataType::U16);
        assert_implicitly_castable!(DataType::U8, DataType::U32);
        assert_implicitly_castable!(DataType::U8, DataType::U64);
        assert_implicitly_castable!(DataType::U16, DataType::U32);
        assert_implicitly_castable!(DataType::U16, DataType::U64);
        assert_implicitly_castable!(DataType::U32, DataType::U64);

        assert_implicitly_castable!(DataType::F32, DataType::F64);

        // Assert not castable to smaller types
        assert_not_implicitly_castable!(DataType::I16, DataType::I8);
        assert_not_implicitly_castable!(DataType::I32, DataType::I8);
        assert_not_implicitly_castable!(DataType::I64, DataType::I8);
        assert_not_implicitly_castable!(DataType::I32, DataType::I16);
        assert_not_implicitly_castable!(DataType::I64, DataType::I16);
        assert_not_implicitly_castable!(DataType::I64, DataType::I32);

        assert_not_implicitly_castable!(DataType::U16, DataType::U8);
        assert_not_implicitly_castable!(DataType::U32, DataType::U8);
        assert_not_implicitly_castable!(DataType::U64, DataType::U8);
        assert_not_implicitly_castable!(DataType::U32, DataType::U16);
        assert_not_implicitly_castable!(DataType::U64, DataType::U16);
        assert_not_implicitly_castable!(DataType::U64, DataType::U32);

        assert_not_implicitly_castable!(DataType::F64, DataType::F32);

        // Array implicit casts
        assert_implicitly_castable!(
            DataType::Array(Rc::new(DataType::I8)),
            DataType::Array(Rc::new(DataType::I16))
        );

        // Array of positive signed integers can be cast to array of unsigned integers.
        let a = LiteralValue::Array { element_type: Rc::new(DataType::I32), items: vec![
            LiteralValue::Numeric(Number::Int(1)),
            LiteralValue::Numeric(Number::Int(2)),
            LiteralValue::Numeric(Number::Int(3)),
        ]};
        assert!(DataType::Array(Rc::new(DataType::I32)).is_implicitly_castable_to(&DataType::Array(Rc::new(DataType::I64)), Some(&a)));
        assert!(DataType::Array(Rc::new(DataType::I32)).is_implicitly_castable_to(&DataType::Array(Rc::new(DataType::U8)), Some(&a)));
        assert!(DataType::Array(Rc::new(DataType::I32)).is_implicitly_castable_to(&DataType::Array(Rc::new(DataType::U16)), Some(&a)));
        assert!(DataType::Array(Rc::new(DataType::I32)).is_implicitly_castable_to(&DataType::Array(Rc::new(DataType::U32)), Some(&a)));
        assert!(DataType::Array(Rc::new(DataType::I32)).is_implicitly_castable_to(&DataType::Array(Rc::new(DataType::U64)), Some(&a)));

        // Array with negative integers can only be cast to array of signed integers, not unsigned integers.
        let b = LiteralValue::Array { element_type: Rc::new(DataType::I32), items: vec![
            LiteralValue::Numeric(Number::Int(1)),
            LiteralValue::Numeric(Number::Int(-2)),
            LiteralValue::Numeric(Number::Int(3)),
        ]};
        assert!(DataType::Array(Rc::new(DataType::I32)).is_implicitly_castable_to(&DataType::Array(Rc::new(DataType::I64)), Some(&b)));
        assert!(!DataType::Array(Rc::new(DataType::I32)).is_implicitly_castable_to(&DataType::Array(Rc::new(DataType::U8)), Some(&b)));
        assert!(!DataType::Array(Rc::new(DataType::I32)).is_implicitly_castable_to(&DataType::Array(Rc::new(DataType::U16)), Some(&b)));
        assert!(!DataType::Array(Rc::new(DataType::I32)).is_implicitly_castable_to(&DataType::Array(Rc::new(DataType::U32)), Some(&b)));
        assert!(!DataType::Array(Rc::new(DataType::I32)).is_implicitly_castable_to(&DataType::Array(Rc::new(DataType::U64)), Some(&b)));
    }

}

