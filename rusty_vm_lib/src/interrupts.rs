use std::mem;


#[repr(u8)]
pub enum Interrupts {

    PrintSigned,
    PrintUnsigned,
    PrintFloat,
    PrintChar,
    PrintString,
    PrintBytes,
    InputSignedInt,
    InputUnsignedInt,
    InputByte,
    InputString,
    Random,
    HostTimeNanos,
    ElapsedTimeNanos,
    DiskRead,
    DiskWrite,
    Terminal,
    SetTimerNanos,
    FlushStdout,
    HostFs,

}


impl From<u8> for Interrupts {
    fn from(value: u8) -> Self {
        unsafe {
            mem::transmute(value)
        }
    }
}
