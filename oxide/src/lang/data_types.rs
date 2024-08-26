use std::borrow::Cow;
use std::fmt::{Display, Debug};
use std::num::{ParseFloatError, ParseIntError};
use std::rc::Rc;

use num_traits::ToBytes;

use crate::match_unreachable;
use crate::symbol_table::{StaticID, SymbolTable};

use rusty_vm_lib::vm::ADDRESS_SIZE;

use self::dt_macros::numeric_pattern;


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


#[derive(Clone, PartialEq)]
pub enum DataType {

    Bool,

    Char,
    RawString { length: usize },
    String,

    Array { element_type: Rc<DataType>, size: Option<usize> },
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

    Function { params: Vec<Rc<DataType>>, return_type: Rc<DataType> },

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
            DataType::U8 | DataType::U16 | DataType::U32 | DataType::U64 | DataType::Usize
        };
    }
    pub(crate) use unsigned_integer_pattern;


    macro_rules! signed_integer_pattern {
        () => {
            DataType::I8 | DataType::I16 | DataType::I32 | DataType::I64 | DataType::Isize
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

}


impl DataType {

    /// Return the size of the data type in bytes, if it is known at compile-time.
    pub fn static_size(&self) -> Result<usize, ()> {

        Ok(match self {
            DataType::Bool => BOOL_SIZE,
            DataType::Char => CHAR_SIZE,
            DataType::I8 => I8_SIZE,
            DataType::I16 => I16_SIZE,
            DataType::I32 => I32_SIZE,
            DataType::I64 => I64_SIZE,
            DataType::U8 => U8_SIZE,
            DataType::U16 => U16_SIZE,
            DataType::U32 => U32_SIZE,
            DataType::U64 => U64_SIZE,
            DataType::F32 => F32_SIZE,
            DataType::F64 => F64_SIZE,
            DataType::Usize => USIZE_SIZE,
            DataType::Isize => ISIZE_SIZE,

            DataType::RawString { length } => CHAR_SIZE * *length,
            DataType::Array { element_type, size} => element_type.static_size()? * size.ok_or(())?,

            DataType::StringRef { length: _ } => ADDRESS_SIZE + USIZE_SIZE,
            DataType::Ref { .. } => ADDRESS_SIZE,

            DataType::String  // TODO: update with a rust-like string struct size
            => todo!("String size is not yet implemented"),

            DataType::Void |
            DataType::Function { .. }
             => 0,

            DataType::Unspecified => unreachable!("Unspecified data type size for {:?}", self),
        })
    }


    pub fn is_castable_to(&self, target: &DataType) -> bool {

        self.is_implicitly_castable_to(target, None, false)
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

            DataType::Ref { mutable: true, target: _ }
            // A mutable pointer is castable to any other reference and to u64 (for pointer arithmetic).
                => matches!(target, DataType::Ref { mutable: _, target: _ } | DataType::U64),

            // Only allow casting an immutable referene to another immutable reference or to u64 (which is unsafe, though).
            DataType::Ref { mutable: false, target: _ }
                => matches!(target, DataType::Ref { mutable: false, target: _ } | DataType::U64),

            DataType::Bool => matches!(target, integer_pattern!()),

            DataType::Array { element_type, size }
             => if let DataType::Array {element_type: target_element_type, size: target_size } = target {
                    // An array is castable to another array if the element type is implicitly castable to the target element type.
                    // Also, the arrays must be of the same size.
                    size == target_size
                    && element_type.is_implicitly_castable_to(target_element_type, None, false)
                } else { false },

            // Other type casts are not allowed.
            _ => false
        }
    }


    /// Return whether `self` is implicitly castable to `target`.
    /// Literal values have a more flexible implicit casting because they're supposed to conform to the symbol they're assigned to.
    pub fn is_implicitly_castable_to(&self, target: &DataType, self_value: Option<&LiteralValue>, is_literal_value: bool) -> bool {

        // A data type is always implicitly castable to itself
        if self == target {
            return true;
        }

        // Can self be implicitly cast to target?
        match self {

            // String references are interchangeable since they always have the same size (wide pointers).
            // TODO: use an Option<usize> to allow for unspecified length.
            DataType::StringRef { length: _ } => matches!(target, DataType::StringRef { length: _ }),

            DataType::Array { element_type, size: src_size }
            => if let DataType::Array { element_type: target_element_type, size: target_size } = target {
                src_size == target_size
                && (
                    // Uninitialized array
                    matches!(**element_type, DataType::Void) // TODO: this should be DataType::Unspecified
                    // Check if the element type is implicitly castable to the target element type.
                    || element_type.is_implicitly_castable_to(target_element_type, None, false)
                    // Else, check if all the items in the array are implicitly castable to the target element type.
                    || self_value.map(|value| match value {
                        LiteralValue::Array { items, .. } => items.iter().all(|item|
                            // Is `element_type` implicitly castable to `target_element_type` if its value is `value`?
                            element_type.is_implicitly_castable_to(target_element_type, Some(item), false)
                        ),
                        _ => false
                    }).unwrap_or(false)
                )
            } else {
                false
            },

            // If target type is x, what types can be implicitly cast to x?
            _ if is_literal_value => match target {

                DataType::I8 => self_value.map(|value| match value {
                        LiteralValue::Numeric(Number::U8(n)) => *n <= i8::MAX as u8,
                        _ => false
                    }).unwrap_or(false),

                DataType::I16 => matches!(self, DataType::I8 | DataType::U8)
                    || self_value.map(|value| match value {
                        LiteralValue::Numeric(Number::U16(n)) => *n <= std::i16::MAX as u16,
                        _ => false
                    }).unwrap_or(false),

                DataType::I32 => matches!(self, DataType::I8 | DataType::U8 | DataType::I16 | DataType::U16)
                    || self_value.map(|value| match value {
                        LiteralValue::Numeric(Number::U32(n)) => *n <= std::i32::MAX as u32,
                        _ => false
                    }).unwrap_or(false),

                DataType::Isize |
                DataType::I64
                => matches!(self, DataType::I8 | DataType::U8 | DataType::I16 | DataType::U16 | DataType::I32 | DataType::U32)
                    || self_value.map(|value| match value {
                        LiteralValue::Numeric(Number::U64(n)) => *n <= std::i64::MAX as u64,
                        _ => false
                    }).unwrap_or(false),

                // Unsigned integer types won't need to be implicitly cast from a strictly literal signed integer because the tokenizer will interpret those numbers as unsigned integers.
                // For example, the tokenizer won't treat the literal number `4` as a signed integer.
                DataType::U8 => false,

                DataType::U16 => matches!(self, DataType::U8),

                DataType::U32 => matches!(self, DataType::U8 | DataType::U16),

                DataType::U64 |
                DataType::Usize
                    => matches!(self, DataType::U8 | DataType::U16 | DataType::U32),

                DataType::F64 => matches!(self, DataType::F32),


                _ => false
            },

            _ => false
        }
    }


    pub fn name(&self) -> Cow<'static, str> { // Uses Cow to avoid leaking owned strings
        match self {
            DataType::Bool => Cow::Borrowed("bool"),
            DataType::Char => Cow::Borrowed("char"),
            DataType::String => Cow::Borrowed("String"),
            DataType::RawString { length } => Cow::Owned(format!("str[{}]", length)),
            DataType::Array { element_type, size } => Cow::Owned(if let Some(size) = size { format!("[{}, {}]", element_type.name(), size) } else { format!("[{}]", element_type.name()) }),
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
            DataType::StringRef { length } => Cow::Owned(format!("str[{}]", length)),
            DataType::Usize => Cow::Borrowed("usize"),
            DataType::Isize => Cow::Borrowed("isize"),
            DataType::Unspecified => Cow::Borrowed("Unspecified"),
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

macro_rules! impl_binary_numeric_op {
    ($name:ident, $op:tt) => {
        pub fn $name(&self, other: &Number) -> Number {
            match (self, other) {
                (Number::I8(n1), Number::I8(n2)) => Number::I8(n1 $op n2),
                (Number::I16(n1), Number::I16(n2)) => Number::I16(n1 $op n2),
                (Number::I32(n1), Number::I32(n2)) => Number::I32(n1 $op n2),
                (Number::I64(n1), Number::I64(n2)) => Number::I64(n1 $op n2),
                (Number::U8(n1), Number::U8(n2)) => Number::U8(n1 $op n2),
                (Number::U16(n1), Number::U16(n2)) => Number::U16(n1 $op n2),
                (Number::U32(n1), Number::U32(n2)) => Number::U32(n1 $op n2),
                (Number::U64(n1), Number::U64(n2)) => Number::U64(n1 $op n2),
                (Number::F32(n1), Number::F32(n2)) => Number::F32(n1 $op n2),
                (Number::F64(n1), Number::F64(n2)) => Number::F64(n1 $op n2),
                _ => unreachable!("Cannot perform operation {} on different numeric types {:?} and {:?}", stringify!($name), self, other)
            }
        }
    };
}

macro_rules! impl_binary_integer_op {
    ($name:ident, $op:tt) => {
        pub fn $name(&self, other: &Number) -> Number {
            match (self, other) {
                (Number::I8(n1), Number::I8(n2)) => Number::I8(n1 $op n2),
                (Number::I16(n1), Number::I16(n2)) => Number::I16(n1 $op n2),
                (Number::I32(n1), Number::I32(n2)) => Number::I32(n1 $op n2),
                (Number::I64(n1), Number::I64(n2)) => Number::I64(n1 $op n2),
                (Number::U8(n1), Number::U8(n2)) => Number::U8(n1 $op n2),
                (Number::U16(n1), Number::U16(n2)) => Number::U16(n1 $op n2),
                (Number::U32(n1), Number::U32(n2)) => Number::U32(n1 $op n2),
                (Number::U64(n1), Number::U64(n2)) => Number::U64(n1 $op n2),
                _ => unreachable!("Cannot perform operation {} on non-integer types or different numeric types {:?} and {:?}", stringify!($name), self, other)
            }
        }
    };
}

macro_rules! impl_binary_numeric_non_zero_op {
    ($name:ident, $op:tt) => {
        pub fn $name(&self, other: &Number) -> Result<Number, ()> {
            match (self, other) {
                (Number::I8(n1), Number::I8(n2)) => if *n2 == 0 { Err(()) } else { Ok(Number::I8(n1 $op n2)) },
                (Number::I16(n1), Number::I16(n2)) => if *n2 == 0 { Err(()) } else { Ok(Number::I16(n1 $op n2)) },
                (Number::I32(n1), Number::I32(n2)) => if *n2 == 0 { Err(()) } else { Ok(Number::I32(n1 $op n2)) },
                (Number::I64(n1), Number::I64(n2)) => if *n2 == 0 { Err(()) } else { Ok(Number::I64(n1 $op n2)) },
                (Number::U8(n1), Number::U8(n2)) => if *n2 == 0 { Err(()) } else { Ok(Number::U8(n1 $op n2)) },
                (Number::U16(n1), Number::U16(n2)) => if *n2 == 0 { Err(()) } else { Ok(Number::U16(n1 $op n2)) },
                (Number::U32(n1), Number::U32(n2)) => if *n2 == 0 { Err(()) } else { Ok(Number::U32(n1 $op n2)) },
                (Number::U64(n1), Number::U64(n2)) => if *n2 == 0 { Err(()) } else { Ok(Number::U64(n1 $op n2)) },
                (Number::F32(n1), Number::F32(n2)) => if *n2 == 0.0 { Err(()) } else { Ok(Number::F32(n1 $op n2)) },
                (Number::F64(n1), Number::F64(n2)) => if *n2 == 0.0 { Err(()) } else { Ok(Number::F64(n1 $op n2)) },
                _ => unreachable!("Cannot perform operation {} on different numeric types {:?} and {:?}", stringify!($name), self, other)
            }
        }
    };
}

macro_rules! impl_binary_numeric_cmp {
    ($name:ident, $op:tt) => {
        pub fn $name(&self, other: &Number) -> bool {
            match (self, other) {
                (Number::I8(n1), Number::I8(n2)) => n1 $op n2,
                (Number::I16(n1), Number::I16(n2)) => n1 $op n2,
                (Number::I32(n1), Number::I32(n2)) => n1 $op n2,
                (Number::I64(n1), Number::I64(n2)) => n1 $op n2,
                (Number::U8(n1), Number::U8(n2)) => n1 $op n2,
                (Number::U16(n1), Number::U16(n2)) => n1 $op n2,
                (Number::U32(n1), Number::U32(n2)) => n1 $op n2,
                (Number::U64(n1), Number::U64(n2)) => n1 $op n2,
                (Number::F32(n1), Number::F32(n2)) => n1 $op n2,
                (Number::F64(n1), Number::F64(n2)) => n1 $op n2,
                _ => unreachable!("Cannot perform operation {} on different numeric types {:?} and {:?}", stringify!($name), self, other)
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
            Number::I8(n) => Number::I8(!n),
            Number::I16(n) => Number::I16(!n),
            Number::I32(n) => Number::I32(!n),
            Number::I64(n) => Number::I64(!n),
            Number::U8(n) => Number::U8(!n),
            Number::U16(n) => Number::U16(!n),
            Number::U32(n) => Number::U32(!n),
            Number::U64(n) => Number::U64(!n),
            _ => unreachable!("Cannot perform operation bitwise-not on non-integer numeric type {:?}", self)
        }
    }


    pub fn assume_usize_like(&self) -> usize {
        match self {
            Number::U8(n) => *n as usize,
            Number::U16(n) => *n as usize,
            Number::U32(n) => *n as usize,
            Number::U64(n) => *n as usize,
            _ => unreachable!("Cannot assume unsigned integer value from {:?}", self)
        }
    }


    pub const fn data_type(&self) -> DataType {
        match self {
            Number::I8(_) => DataType::I8,
            Number::I16(_) => DataType::I16,
            Number::I32(_) => DataType::I32,
            Number::I64(_) => DataType::I64,
            Number::U8(_) => DataType::U8,
            Number::U16(_) => DataType::U16,
            Number::U32(_) => DataType::U32,
            Number::U64(_) => DataType::U64,
            Number::F32(_) => DataType::F32,
            Number::F64(_) => DataType::F64,
        }
    }


    pub fn try_parse_unsigned_int(string: &str) -> Result<Self, ParseIntError> {

        let n = string.parse::<u64>()?;

        Ok(
            if n <= u8::MAX as u64 {
                Number::U8(n as u8)
            } else if n <= u16::MAX as u64 {
                Number::U16(n as u16)
            } else if n <= u32::MAX as u64 {
                Number::U32(n as u32)
            } else {
                Number::U64(n)
            }
        )
    }


    pub fn try_parse_signed_int(string: &str) -> Result<Self, ParseIntError> {

        let n = string.parse::<i64>()?;

        Ok(
            if n >= i8::MIN as i64 && n <= i8::MAX as i64 {
                Number::I8(n as i8)
            } else if n >= i16::MIN as i64 && n <= i16::MAX as i64 {
                Number::I16(n as i16)
            } else if n >= i32::MIN as i64 && n <= i32::MAX as i64 {
                Number::I32(n as i32)
            } else {
                Number::I64(n)
            }
        )
    }


    pub fn try_parse_float(string: &str) -> Result<Self, ParseFloatError> {

        let n = string.parse::<f64>()?;

        Ok(
            if n >= f32::MIN as f64 && n <= f32::MAX as f64 {
                Number::F32(n as f32)
            } else {
                Number::F64(n)
            }
        )
    }

}

impl ToBytes for &Number {
    type Bytes = Box<[u8]>;

    // TODO: Heap allocations are overkill for this
    fn to_le_bytes(&self) -> Self::Bytes {
        match self {
            Number::I8(n) => n.to_le_bytes().into(),
            Number::I16(n) => n.to_le_bytes().into(),
            Number::I32(n) => n.to_le_bytes().into(),
            Number::I64(n) => n.to_le_bytes().into(),
            Number::U8(n) => n.to_le_bytes().into(),
            Number::U16(n) => n.to_le_bytes().into(),
            Number::U32(n) => n.to_le_bytes().into(),
            Number::U64(n) => n.to_le_bytes().into(),
            Number::F32(n) => n.to_le_bytes().into(),
            Number::F64(n) => n.to_le_bytes().into(),
        }
    }

    fn to_be_bytes(&self) -> Self::Bytes {
        unimplemented!()
    }

    fn to_ne_bytes(&self) -> Self::Bytes {
        unimplemented!()
    }

}

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Number::I8(n) => write!(f, "{}", n),
            Number::I16(n) => write!(f, "{}", n),
            Number::I32(n) => write!(f, "{}", n),
            Number::I64(n) => write!(f, "{}", n),
            Number::U8(n) => write!(f, "{}", n),
            Number::U16(n) => write!(f, "{}", n),
            Number::U32(n) => write!(f, "{}", n),
            Number::U64(n) => write!(f, "{}", n),
            Number::F32(n) => write!(f, "{}", n),
            Number::F64(n) => write!(f, "{}", n),
        }
    }
}


#[derive(Debug, Clone)]
pub enum LiteralValue {

    Char (char),
    StaticString (StaticID),

    Array { element_type: Rc<DataType>, items: Box<[Rc<LiteralValue>]> },

    Numeric (Number),

    Bool (bool),

    // TODO: probably we should use a RefCell here to allow for runtime mutability when evaluating constant operations.
    Ref { target: Rc<LiteralValue>, mutable: bool }

}

impl LiteralValue {

    pub fn assume_array(&self) -> (&Rc<DataType>, &[Rc<LiteralValue>]) {
        match self {
            LiteralValue::Array { element_type, items } => (element_type, items),
            _ => unreachable!("Cannot assume array value from {:?}", self)
        }
    }

    pub fn assume_numeric(&self) -> &Number {
        match self {
            LiteralValue::Numeric(n) => n,
            _ => unreachable!("Cannot assume numeric value from {:?}", self)
        }
    }

    pub fn assume_bool(&self) -> bool {
        match self {
            LiteralValue::Bool(b) => *b,
            _ => unreachable!("Cannot assume bool value from {:?}", self)
        }
    }

    pub fn assume_ref(&self) -> (&Rc<LiteralValue>, bool) {
        match self {
            LiteralValue::Ref { target, mutable } => (target, *mutable),
            _ => unreachable!("Cannot assume reference value from {:?}", self)
        }
    }

    /// Assumes that the source value is castable to the target type. This should have been checked during type resolution.
    ///
    /// This function can perform only compile-time casts.
    pub fn from_cast(src_value: &LiteralValue, src_type: &DataType, target_type: &DataType) -> Rc<Self> {

        assert!(src_type.is_castable_to(target_type));

        // This is checked before calling the function.
        // if src_type == target_type {
        //     return src_value;
        // }

        macro_rules! impl_numeric_cast {
            ($numeric_type:ident, INCLUDE_BOOL) => {{
                let value = match_unreachable!(LiteralValue::Numeric(Number::$numeric_type(value)) = src_value, *value);
                match target_type {
                    DataType::Bool => LiteralValue::Bool(value != 0),
                    DataType::I8 => LiteralValue::Numeric(Number::I8(value as i8)),
                    DataType::I16 => LiteralValue::Numeric(Number::I16(value as i16)),
                    DataType::I32 => LiteralValue::Numeric(Number::I32(value as i32)),
                    DataType::I64 => LiteralValue::Numeric(Number::I64(value as i64)),
                    DataType::U8 => LiteralValue::Numeric(Number::U8(value as u8)),
                    DataType::U16 => LiteralValue::Numeric(Number::U16(value as u16)),
                    DataType::U32 => LiteralValue::Numeric(Number::U32(value as u32)),
                    DataType::U64 => LiteralValue::Numeric(Number::U64(value as u64)),
                    DataType::F32 => LiteralValue::Numeric(Number::F32(value as f32)),
                    DataType::F64 => LiteralValue::Numeric(Number::F64(value as f64)),
                    _ => unreachable!("Cannot cast from {:?} to {:?}", src_type, target_type)
                }
            }};
            ($numeric_type:ident, EXCLUDE_BOOL) => {{
                let value = match_unreachable!(LiteralValue::Numeric(Number::$numeric_type(value)) = src_value, *value);
                match target_type {
                    DataType::I8 => LiteralValue::Numeric(Number::I8(value as i8)),
                    DataType::I16 => LiteralValue::Numeric(Number::I16(value as i16)),
                    DataType::I32 => LiteralValue::Numeric(Number::I32(value as i32)),
                    DataType::I64 => LiteralValue::Numeric(Number::I64(value as i64)),
                    DataType::U8 => LiteralValue::Numeric(Number::U8(value as u8)),
                    DataType::U16 => LiteralValue::Numeric(Number::U16(value as u16)),
                    DataType::U32 => LiteralValue::Numeric(Number::U32(value as u32)),
                    DataType::U64 => LiteralValue::Numeric(Number::U64(value as u64)),
                    DataType::F32 => LiteralValue::Numeric(Number::F32(value as f32)),
                    DataType::F64 => LiteralValue::Numeric(Number::F64(value as f64)),
                    _ => unreachable!("Cannot cast from {:?} to {:?}", src_type, target_type)
                }
            }}
        }

        match src_type {

            DataType::Bool => {
                let value = match_unreachable!(LiteralValue::Bool(value) = src_value, *value);
                match target_type {
                    DataType::I8 => LiteralValue::Numeric(Number::I8(value as i8)),
                    DataType::I16 => LiteralValue::Numeric(Number::I16(value as i16)),
                    DataType::I32 => LiteralValue::Numeric(Number::I32(value as i32)),
                    DataType::I64 => LiteralValue::Numeric(Number::I64(value as i64)),
                    DataType::U8 => LiteralValue::Numeric(Number::U8(value as u8)),
                    DataType::U16 => LiteralValue::Numeric(Number::U16(value as u16)),
                    DataType::U32 => LiteralValue::Numeric(Number::U32(value as u32)),
                    DataType::U64 => LiteralValue::Numeric(Number::U64(value as u64)),
                    _ => unreachable!("Cannot cast from {:?} to {:?}", src_type, target_type)
                }
            },

            DataType::Char => {
                let ch = match_unreachable!(LiteralValue::Char(ch) = src_value, *ch);
                match target_type {
                    DataType::I8 => LiteralValue::Numeric(Number::I8(ch as i8)),
                    DataType::I16 => LiteralValue::Numeric(Number::I16(ch as i16)),
                    DataType::I32 => LiteralValue::Numeric(Number::I32(ch as i32)),
                    DataType::I64 => LiteralValue::Numeric(Number::I64(ch as i64)),
                    DataType::U8 => LiteralValue::Numeric(Number::U8(ch as u8)),
                    DataType::U16 => LiteralValue::Numeric(Number::U16(ch as u16)),
                    DataType::U32 => LiteralValue::Numeric(Number::U32(ch as u32)),
                    DataType::U64 => LiteralValue::Numeric(Number::U64(ch as u64)),
                    _ => unreachable!("Cannot cast from {:?} to {:?}", src_type, target_type)
                }
            },

            // Handle the u8 as char special case before implementing the more generic numeric casts
            DataType::U8 if matches!(target_type, DataType::Char) => {
                let value = match_unreachable!(LiteralValue::Numeric(Number::U8(value)) = src_value, *value);
                LiteralValue::Char(value as char)
            },

            DataType::I8 => impl_numeric_cast!(I8, INCLUDE_BOOL),
            DataType::I16 => impl_numeric_cast!(I16, INCLUDE_BOOL),
            DataType::I32 => impl_numeric_cast!(I32, INCLUDE_BOOL),
            DataType::I64 => impl_numeric_cast!(I64, INCLUDE_BOOL),
            DataType::U8 => impl_numeric_cast!(U8, INCLUDE_BOOL),
            DataType::U16 => impl_numeric_cast!(U16, INCLUDE_BOOL),
            DataType::U32 => impl_numeric_cast!(U32, INCLUDE_BOOL),
            DataType::U64 => impl_numeric_cast!(U64, INCLUDE_BOOL),
            DataType::F32 => impl_numeric_cast!(F32, EXCLUDE_BOOL),
            DataType::F64 => impl_numeric_cast!(F64, EXCLUDE_BOOL),

            DataType::Array { element_type: src_element_type, size } => {
                let (target_element_type, items) = match_unreachable!(LiteralValue::Array { element_type: target_element_type, items } = src_value, (target_element_type, items));

                assert_eq!(items.len(), size.expect("Array size is not known at compile-time"));

                let res = items.iter().map(
                    |item| LiteralValue::from_cast(item, src_element_type, target_element_type)
                ).collect();

                LiteralValue::Array {
                    element_type: Rc::clone(target_element_type),
                    items: res
                }
            },

            _ => unreachable!("Cannot cast from {:?} to {:?} (or at least not at compile-time)", src_type, target_type)

        }.into()
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


    /// Get the data type of the literal value.
    /// Since this function may need to perform recursion for arrays, it's recommended that the result is saved if repeated usage is necessary.
    pub fn data_type(&self, symbol_table: &SymbolTable<'_>) -> DataType {
        match self {

            LiteralValue::Char(_) => DataType::Char,

            LiteralValue::StaticString(id)
                => DataType::StringRef { length: symbol_table.get_static_string(*id).len() },

            LiteralValue::Array { element_type: dt, items }
                => DataType::Array { element_type: Rc::clone(dt), size: Some(items.len()) },

            LiteralValue::Numeric(n) => n.data_type(),

            LiteralValue::Bool(_) => DataType::Bool,

            LiteralValue::Ref { target, mutable }
                => DataType::Ref { target: target.data_type(symbol_table).into(), mutable: *mutable },
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
            LiteralValue::Ref { target, mutable } => write!(f, "&{}{:?}", if *mutable { "mut " } else { "" }, target),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use super::DataType;

    macro_rules! assert_implicitly_castable {
        ($a:expr, $b:expr, $value:expr, $is_literal_value:expr) => {
            assert!(DataType::is_implicitly_castable_to(&$a, &$b, $value, $is_literal_value))
        };
    }

    macro_rules! assert_not_implicitly_castable {
        ($a:expr, $b:expr, $is_literal_value:expr) => {
            assert!(!DataType::is_implicitly_castable_to(&$a, &$b, None, $is_literal_value))
        };
    }


    #[test]
    fn check_number_size_consistency() {

        assert_eq!(Number::I8(0).data_type().static_size().unwrap(), DataType::I8.static_size().unwrap());
        assert_eq!(Number::I16(0).data_type().static_size().unwrap(), DataType::I16.static_size().unwrap());
        assert_eq!(Number::I32(0).data_type().static_size().unwrap(), DataType::I32.static_size().unwrap());
        assert_eq!(Number::I64(0).data_type().static_size().unwrap(), DataType::I64.static_size().unwrap());
        assert_eq!(Number::I64(0).data_type().static_size().unwrap(), DataType::Isize.static_size().unwrap());
        assert_eq!(Number::U8(0).data_type().static_size().unwrap(), DataType::U8.static_size().unwrap());
        assert_eq!(Number::U16(0).data_type().static_size().unwrap(), DataType::U16.static_size().unwrap());
        assert_eq!(Number::U32(0).data_type().static_size().unwrap(), DataType::U32.static_size().unwrap());
        assert_eq!(Number::U64(0).data_type().static_size().unwrap(), DataType::U64.static_size().unwrap());
        assert_eq!(Number::U64(0).data_type().static_size().unwrap(), DataType::Usize.static_size().unwrap());
        assert_eq!(Number::F32(0.0).data_type().static_size().unwrap(), DataType::F32.static_size().unwrap());
        assert_eq!(Number::F64(0.0).data_type().static_size().unwrap(), DataType::F64.static_size().unwrap());

    }


    #[test]
    fn implicit_casts() {

        assert_not_implicitly_castable!(DataType::I8, DataType::I16, false);
        assert_not_implicitly_castable!(DataType::I8, DataType::I32, false);
        assert_not_implicitly_castable!(DataType::I8, DataType::I64, false);
        assert_not_implicitly_castable!(DataType::I16, DataType::I32, false);
        assert_not_implicitly_castable!(DataType::I16, DataType::I64, false);
        assert_not_implicitly_castable!(DataType::I32, DataType::I64, false);

        assert_not_implicitly_castable!(DataType::U8, DataType::U16, false);
        assert_not_implicitly_castable!(DataType::U8, DataType::U32, false);
        assert_not_implicitly_castable!(DataType::U8, DataType::U64, false);
        assert_not_implicitly_castable!(DataType::U16, DataType::U32, false);
        assert_not_implicitly_castable!(DataType::U16, DataType::U64, false);
        assert_not_implicitly_castable!(DataType::U32, DataType::U64, false);

        assert_not_implicitly_castable!(DataType::F32, DataType::F64, false);

        assert_not_implicitly_castable!(DataType::I16, DataType::I8, false);
        assert_not_implicitly_castable!(DataType::I32, DataType::I8, false);
        assert_not_implicitly_castable!(DataType::I64, DataType::I8, false);
        assert_not_implicitly_castable!(DataType::I32, DataType::I16, false);
        assert_not_implicitly_castable!(DataType::I64, DataType::I16, false);
        assert_not_implicitly_castable!(DataType::I64, DataType::I32, false);

        assert_not_implicitly_castable!(DataType::U16, DataType::U8, false);
        assert_not_implicitly_castable!(DataType::U32, DataType::U8, false);
        assert_not_implicitly_castable!(DataType::U64, DataType::U8, false);
        assert_not_implicitly_castable!(DataType::U32, DataType::U16, false);
        assert_not_implicitly_castable!(DataType::U64, DataType::U16, false);
        assert_not_implicitly_castable!(DataType::U64, DataType::U32, false);

        assert_not_implicitly_castable!(DataType::F64, DataType::F32, false);

        assert_implicitly_castable!(DataType::I32, DataType::I64, Some(&LiteralValue::Numeric(Number::I32(312))), true);

        // Array implicit casts
        // assert_not_implicitly_castable!(
        //     DataType::Array { element_type: Rc::new(DataType::I8), size: Some(3) },
        //     DataType::Array { element_type: Rc::new(DataType::I16), size: Some(3) },
        //     false
        // );

        // // Array of positive signed integers can be cast to array of unsigned integers.
        // let a = LiteralValue::Array { element_type: Rc::new(DataType::I32), items: vec![
        //     LiteralValue::Numeric(Number::I32(1)).into(),
        //     LiteralValue::Numeric(Number::I32(2)).into(),
        //     LiteralValue::Numeric(Number::I32(3)).into(),
        // ].into_boxed_slice()};
        //     assert!(DataType::Array { element_type: Rc::new(DataType::I32), size: Some(3) }.is_implicitly_castable_to(&DataType::Array { element_type: Rc::new(DataType::U8), size: Some(3) }, Some(&a)));
        //     assert!(DataType::Array { element_type: Rc::new(DataType::I32), size: Some(3) }.is_implicitly_castable_to(&DataType::Array { element_type: Rc::new(DataType::U16), size: Some(3) }, Some(&a)));
        //     assert!(DataType::Array { element_type: Rc::new(DataType::I32), size: Some(3) }.is_implicitly_castable_to(&DataType::Array { element_type: Rc::new(DataType::U32), size: Some(3) }, Some(&a)));
        //     assert!(DataType::Array { element_type: Rc::new(DataType::I32), size: Some(3) }.is_implicitly_castable_to(&DataType::Array { element_type: Rc::new(DataType::U64), size: Some(3) }, Some(&a)));

        //     // Array with negative integers can only be cast to array of signed integers, not unsigned integers.
        //     let b = LiteralValue::Array { element_type: Rc::new(DataType::I32), items: vec![
        //         LiteralValue::Numeric(Number::I32(1)).into(),
        //         LiteralValue::Numeric(Number::I32(-2)).into(),
        //         LiteralValue::Numeric(Number::I32(3)).into(),
        //     ].into_boxed_slice()};
        //     assert!(DataType::Array { element_type: Rc::new(DataType::I32), size: Some(3) }.is_implicitly_castable_to(&DataType::Array { element_type: Rc::new(DataType::I64), size: Some(3) }, Some(&b)));
        //     assert!(!DataType::Array { element_type: Rc::new(DataType::I32), size: Some(3) }.is_implicitly_castable_to(&DataType::Array { element_type: Rc::new(DataType::U8), size: Some(3) }, Some(&b)));
        //     assert!(!DataType::Array { element_type: Rc::new(DataType::I32), size: Some(3) }.is_implicitly_castable_to(&DataType::Array { element_type: Rc::new(DataType::U16), size: Some(3) }, Some(&b)));
        //     assert!(!DataType::Array { element_type: Rc::new(DataType::I32), size: Some(3) }.is_implicitly_castable_to(&DataType::Array { element_type: Rc::new(DataType::U32), size: Some(3) }, Some(&b)));
        //     assert!(!DataType::Array { element_type: Rc::new(DataType::I32), size: Some(3) }.is_implicitly_castable_to(&DataType::Array { element_type: Rc::new(DataType::U64), size: Some(3) }, Some(&b)));

    }

}
