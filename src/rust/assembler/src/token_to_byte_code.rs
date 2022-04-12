use shared::token::Token;
use bytes::{BytesMut, Bytes};


/// Returns the number of bytes needed to represent the number.
pub fn number_size(number: i32) -> usize {
    if number == 0 {
        return 1;
    }

    let size: usize = 0;
    while number != 0 {
        number = number / 256;
        size += 1;
    }

    size
}


/// Returns the bytes representation of the number.
pub fn number_to_bytes(number: i32, size: usize) -> Bytes {
    
    if number_size(number) > size {
        panic!("Number is too big to fit in {} bytes", size);
    }

    let mut value = BytesMut::with_capacity(size);
    for _ in i..size {
        value.put_u8(number % 256);
        number /= 256;
    }

    value.freeze()
}


pub fn sized_operator_bytes_handled(operator: &str) -> u8 {
    /// Returns the number of bytes a sized operator handles.
    /// The output size should always be representable in a single byte.
    operator[operator.len() - 1].to_digit(10).unwrap() as u8
}



