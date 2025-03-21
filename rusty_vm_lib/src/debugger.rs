use std::time::Duration;
use std::mem;

use crate::registers::CPURegisters;


pub const DEBUGGER_ATTACH_SLEEP: Duration = Duration::from_millis(200);
pub const DEBUGGER_COMMAND_WAIT_SLEEP: Duration = Duration::from_millis(50);
pub const DEBUGGER_UPDATE_WAIT_SLEEP: Duration = Duration::from_millis(10);

pub const CPU_REGISTERS_OFFSET: usize = 0;
pub const RUNNING_FLAG_OFFSET: usize = CPU_REGISTERS_OFFSET + mem::size_of::<CPURegisters>();
pub const TERMINATE_COMMAND_OFFSET: usize = RUNNING_FLAG_OFFSET + mem::size_of::<bool>();
pub const VM_UPDATED_COUNTER_OFFSET: usize = TERMINATE_COMMAND_OFFSET + mem::size_of::<bool>();
pub const VM_MEM_OFFSET: usize = VM_UPDATED_COUNTER_OFFSET + mem::size_of::<u8>();

pub const DEBUGGER_PATH_ENV: &str = "RUSTYVM_DEBUGGER";
