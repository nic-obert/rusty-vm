use std::any::Any;
use std::io;
use std::fmt::{self, Display};
use std::mem;



pub type Address = usize;
pub const ADDRESS_SIZE: usize = mem::size_of::<Address>();
pub const HANDLED_SIZE_SPECIFIER: usize = 1;
pub const REGISTER_ID_SIZE: usize = 1;


macro_rules! declare_errors {
    ($($name:ident),+) => {

#[derive(Clone, Copy, Debug)]
pub enum ErrorCodes {

    $($name),+

}


const ERROR_CODES_COUNT: usize = mem::variant_count::<ErrorCodes>();


impl Display for ErrorCodes {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
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


impl ErrorCodes {

    pub fn name(&self) -> &'static str {
        match self {
            $(Self::$name => stringify!($name)),+
        }
    }

}

    };
}

declare_errors! {

    NoError,

    EndOfFile,
    InvalidInput,
    ZeroDivision,
    StackOverflow,
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
    GenericError

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

