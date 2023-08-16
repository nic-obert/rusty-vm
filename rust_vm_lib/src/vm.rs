

pub type Address = usize;
pub const ADDRESS_SIZE: usize = std::mem::size_of::<Address>();


#[derive(Clone, Copy)]
pub enum ErrorCodes {

    NoError = 0,

    EndOfFile,

    InvalidInput,

    ZeroDivision,

    // This has to be the last variant
    GenericError

}


impl ErrorCodes {

    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "NO_ERROR" => ErrorCodes::NoError,
            "END_OF_FILE" => ErrorCodes::EndOfFile,
            "INVALID_INPUT" => ErrorCodes::InvalidInput,
            "ZERO_DIVISION" => ErrorCodes::ZeroDivision,
            "GENERIC_ERROR" => ErrorCodes::GenericError,

            _ => return None,
        })
    }
}


const ERROR_CODES_COUNT: usize = {
    assert!((ErrorCodes::GenericError as usize) < 256);
    ErrorCodes::GenericError as usize + 1
};


const ERROR_CODE_REPR: [&str; ERROR_CODES_COUNT] = [
    "NO_ERROR",
    "END_OF_FILE",
    "INVALID_INPUT",
    "ZERO_DIVISION",
    "GENERIC_ERROR"
];


impl std::fmt::Display for ErrorCodes {

    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", ERROR_CODE_REPR[*self as usize])
    }

}


impl std::convert::From<u8> for ErrorCodes {

    fn from(code: u8) -> ErrorCodes {
        if code < ERROR_CODES_COUNT as u8 {
            unsafe { std::mem::transmute(code) }
        } else {
            panic!("Invalid error code: {}", code);
        }
    }

}


#[cfg(test)]
mod tests {

    #[test]
    fn assert_error_codes_repr_consistency() {
        for (i, repr) in super::ERROR_CODE_REPR.iter().enumerate() {
            assert_eq!(i, super::ErrorCodes::from_name(repr).unwrap() as usize);
        }
    }

}

