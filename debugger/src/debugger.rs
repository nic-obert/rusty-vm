use std::mem;
use std::io;
use std::fs;
use std::path::PathBuf;
use std::thread;

use rusty_vm_lib::registers::CPURegisters;
use rusty_vm_lib::debugger::{CPU_REGISTERS_OFFSET, DEBUGGER_UPDATE_WAIT_SLEEP, RUNNING_FLAG_OFFSET, TERMINATE_COMMAND_OFFSET, VM_MEM_OFFSET, VM_UPDATED_COUNTER_OFFSET};

use shared_memory::{Shmem, ShmemConf, ShmemError};


pub struct Debugger {

    shmem: Shmem,

    running_flag: *mut bool,
    terminate_command: *mut bool,
    vm_updated_counter: *mut u8,
    cpu_registers: *mut CPURegisters,
    vm_memory: *mut u8

}

impl Debugger {

    fn read_update_counter(&self) -> u8 {
        unsafe {
            self.vm_updated_counter.read_volatile()
        }
    }


    fn wait_for_vm(&self, old_counter: u8) {
        while old_counter == self.read_update_counter() {
            thread::sleep(DEBUGGER_UPDATE_WAIT_SLEEP);
        }
    }

    fn is_running(&self) -> bool {
        unsafe {
            self.running_flag.read_volatile()
        }
    }


    pub fn close(&self) {
        unsafe {
            self.terminate_command.write(true);
        }
    }


    /// The returned slice is not guaranteed to be consistent when the VM is running
    pub fn read_vm_memory(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.vm_memory,
                self.vm_memory_size()
            )
        }
    }


    pub fn stop_vm(&self) {

        // Don't stop the VM if it's not running
        if !self.is_running() {
            return;
        }

        let old_counter = self.read_update_counter();
        // Tell the VM to stop
        unsafe {
            self.running_flag.write_volatile(false);
        }
        // Wait for the VM to stop
        self.wait_for_vm(old_counter);
    }


    pub fn dump_core<F>(&self, get_dump_path: F) -> Option<io::Result<()>>
        where F: FnOnce() -> Option<PathBuf>
    {
        /*
            Core dump structure:
            - Registers
            - Memory
        */

        self.stop_vm();

        let file_path = get_dump_path()?;

        let mut buf: Vec<u8> = Vec::with_capacity(mem::size_of::<CPURegisters>() + self.vm_memory_size());

        let cpu_registers = unsafe {
            self.cpu_registers.read_volatile()
        };

        buf.extend_from_slice(cpu_registers.as_bytes());
        buf.extend_from_slice(self.read_vm_memory());

        Some(
            fs::write(file_path, buf)
        )
    }

    pub fn vm_memory_size(&self) -> usize {
        self.shmem.len() - VM_MEM_OFFSET
    }


    pub fn resume_vm(&self) {

        // Don't resume the VM if it's already running
        if self.is_running() {
            return;
        }

        let old_counter = self.read_update_counter();
        // Tell the VM to resume
        unsafe {
            self.running_flag.write_volatile(true);
        }
        // Wait until the VM is ready
        self.wait_for_vm(old_counter);
    }


    pub fn try_attach(shmem_id: String) -> Result<Self, ShmemError> {
        let shmem = ShmemConf::new()
            .os_id(shmem_id)
            .open()?;

        let running_flag = unsafe { shmem.as_ptr().byte_add(RUNNING_FLAG_OFFSET).cast::<bool>() };
        let terminate_command = unsafe { shmem.as_ptr().byte_add(TERMINATE_COMMAND_OFFSET).cast::<bool>() };
        let vm_updated_counter = unsafe { shmem.as_ptr().byte_add(VM_UPDATED_COUNTER_OFFSET) };
        let cpu_registers = unsafe { shmem.as_ptr().byte_add(CPU_REGISTERS_OFFSET).cast::<CPURegisters>() };
        let vm_memory = unsafe { shmem.as_ptr().byte_add(VM_MEM_OFFSET) };

        Ok(Self {
            shmem,
            running_flag,
            terminate_command,
            vm_updated_counter,
            cpu_registers,
            vm_memory
        })
    }

}
