mod cli_parser;

use clap::Parser;
use cli_parser::CliParser;
use shared_memory::ShmemConf;

use std::mem;
use std::time::Duration;

use rusty_vm_lib::registers::CPURegisters;
use rusty_vm_lib::debugger::{RUNNING_FLAG_OFFSET, TERMINATE_COMMAND_OFFSET, VM_MEM_OFFSET, VM_UPDATED_COUNTER_OFFSET, CPU_REGISTERS_OFFSET};




fn main() {

    let args = CliParser::parse();

    let shmem = ShmemConf::new()
        .os_id(args.shmem_id)
        .open()
        .unwrap();

    let running_flag = unsafe { shmem.as_ptr().byte_add(RUNNING_FLAG_OFFSET).cast::<bool>() };
    let terminate_command = unsafe { shmem.as_ptr().byte_add(TERMINATE_COMMAND_OFFSET).cast::<bool>() };
    let vm_updated_counter = unsafe { shmem.as_ptr().byte_add(VM_UPDATED_COUNTER_OFFSET) };
    let cpu_registers = unsafe { shmem.as_ptr().byte_add(CPU_REGISTERS_OFFSET).cast::<CPURegisters>() };


    unsafe {
        running_flag.write_volatile(true);
    }

    println!("{:?}", unsafe {cpu_registers.read_volatile()});

    std::thread::sleep(Duration::from_secs(2));

    println!("{:?}", unsafe {cpu_registers.read_volatile()});


}
