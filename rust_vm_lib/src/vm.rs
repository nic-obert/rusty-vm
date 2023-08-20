

pub type Address = usize;
pub const ADDRESS_SIZE: usize = std::mem::size_of::<Address>();


#[derive(Clone, Copy)]
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

    "Generic error"
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

