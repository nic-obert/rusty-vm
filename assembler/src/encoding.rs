use bytes::{BytesMut, BufMut, Bytes};


/// Returns the number of bytes needed to represent the number.
pub const fn number_size(mut number: u64) -> usize {
    if number == 0 {
        return 1;
    }

    let mut size: usize = 0;
    while number != 0 {
        number = number / 256;
        size += 1;
    }

    size
}


/// Returns the bytes representation of the number using little endian.
pub fn number_to_bytes(mut number: u64, size: usize) -> Result<Bytes, String> {
    
    if number_size(number) > size {
        return Err(format!("Number {} is too big to fit in {} bytes", number, size));
    }

    let mut value = BytesMut::with_capacity(size);
    // Perform a reverse loop for little endian
    for _ in 0..size {
        value.put_u8((number % 256) as u8);
        number /= 256;
    }

    Ok(value.freeze())
}

