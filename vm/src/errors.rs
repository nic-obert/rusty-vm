

const ERROR_CODES_COUNT: usize = 4;


#[derive(Clone, Copy)]
pub enum ErrorCodes {
    NoError = 0,

    EndOfFile,
    InvalidInput,
    GenericError,
}


impl std::fmt::Display for ErrorCodes {

    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ErrorCodes::NoError => write!(f, "No Error"),
            ErrorCodes::EndOfFile => write!(f, "End of File"),
            ErrorCodes::InvalidInput => write!(f, "Invalid Input"),
            ErrorCodes::GenericError => write!(f, "Generic Error"),
        }
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

