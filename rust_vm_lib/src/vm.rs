use std::any::Any;
use std::io;
use std::fmt::{self, Display};
use std::mem;



pub type Address = usize;
pub const ADDRESS_SIZE: usize = mem::size_of::<Address>();


#[derive(Clone, Copy, Debug)]
pub enum ErrorCodes {

    NoError = 0,

    EndOfFile,
    InvalidInput,
    ZeroDivision,
    AllocationTooLarge,
    StackOverflow,
    HeapOverflow,
    DoubleFree,
    OutOfBounds,
    UnalignedAddress,
    PermissionDenied,
    TimedOut,
    NotFound,
    AlreadyExists,
    InvalidData,
    Interrupted,
    OutOfMemory,
    WriteZero,
    ModuleUnavailable,

    // This has to be the last variant
    GenericError

}


const ERROR_CODES_COUNT: usize = {
    assert!((ErrorCodes::GenericError as usize) < 256);
    ErrorCodes::GenericError as usize + 1
};


const ERROR_CODE_REPR: [&str; ERROR_CODES_COUNT] = [
    "No error",
    "End of file",
    "Invalid input",
    "Zero division",
    "Allocation too large",
    "Stack overflow",
    "Heap overflow",
    "Double free",
    "Out of bounds",
    "Unaligned address",
    "Permission denied",
    "Timed out",
    "Not found",
    "Already exists",
    "Invalid data",
    "Interrupted",
    "Out of memory",
    "Write zero",
    "Module unavailable",

    "Generic error"
];


impl Display for ErrorCodes {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", ERROR_CODE_REPR[*self as usize])
    }

}


impl From<u8> for ErrorCodes {

    fn from(code: u8) -> ErrorCodes {
        if code < ERROR_CODES_COUNT as u8 {
            unsafe { mem::transmute(code) }
        } else {
            panic!("Invalid error code: {}", code);
        }
    }

}


impl From<io::Error> for ErrorCodes {

    fn from(value: io::Error) -> Self {
        use std::io::ErrorKind;

        match value.kind() {

            ErrorKind::NotFound => ErrorCodes::NotFound,

            ErrorKind::PermissionDenied => ErrorCodes::PermissionDenied,

            ErrorKind::ConnectionRefused
            | ErrorKind::ConnectionReset
            | ErrorKind::ConnectionAborted
            | ErrorKind::NotConnected
            | ErrorKind::AddrInUse
            | ErrorKind::AddrNotAvailable
            | ErrorKind::BrokenPipe
            | ErrorKind::WouldBlock
            | ErrorKind::Unsupported
            => unimplemented!(),

            ErrorKind::AlreadyExists => ErrorCodes::AlreadyExists,

            ErrorKind::InvalidInput => ErrorCodes::InvalidInput,

            ErrorKind::InvalidData => ErrorCodes::InvalidData,

            ErrorKind::TimedOut => ErrorCodes::TimedOut,

            ErrorKind::WriteZero => ErrorCodes::WriteZero,

            ErrorKind::Interrupted => ErrorCodes::Interrupted,

            ErrorKind::UnexpectedEof => ErrorCodes::EndOfFile,

            ErrorKind::OutOfMemory => ErrorCodes::OutOfMemory,

            ErrorKind::Other => ErrorCodes::GenericError,
            
            _ => ErrorCodes::GenericError,
        }
    }

}


impl<T: Any> From<io::Result<T>> for ErrorCodes {

    fn from(value: io::Result<T>) -> Self {
        match value {
            Ok(_) => ErrorCodes::NoError,
            Err(e) => ErrorCodes::from(e),
        }
    }

}

