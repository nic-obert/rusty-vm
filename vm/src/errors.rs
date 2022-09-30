

pub enum ErrorCodes {
    NoError,

    EndOfFile,
    InvalidInput,
    GenericError,
}


impl std::fmt::Display for ErrorCodes {

    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ErrorCodes::NoError => write!(f, "No error"),
            ErrorCodes::EndOfFile => write!(f, "End of file"),
            ErrorCodes::InvalidInput => write!(f, "Invalid input"),
            ErrorCodes::GenericError => write!(f, "Generic error"),
        }
    }

}

