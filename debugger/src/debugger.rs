use std::collections::HashMap;
use std::mem;
use std::io;
use std::fs;
use std::path::PathBuf;
use std::thread;

use rusty_vm_lib::registers::{CPURegisters, Registers};
use rusty_vm_lib::debugger::{CPU_REGISTERS_OFFSET, DEBUGGER_UPDATE_WAIT_SLEEP, RUNNING_FLAG_OFFSET, TERMINATE_COMMAND_OFFSET, VM_MEM_OFFSET, VM_UPDATED_COUNTER_OFFSET};
use rusty_vm_lib::byte_code::ByteCodes;

use shared_memory::{Shmem, ShmemConf, ShmemError};
use slint::SharedString;


#[derive(Clone)]
struct Breakpoint {
    pub name: Option<SharedString>,
    pub location: usize,
    pub replaced_value: u8,
    pub persistent: bool
}

#[derive(Default)]
struct BreakpointTable {
    breakpoints: HashMap<usize, Breakpoint>,
}

impl BreakpointTable {

    pub fn get(&self, location: usize) -> Option<Breakpoint> {
        self.breakpoints.get(&location).cloned()
    }

    pub fn insert(&mut self, location: usize, bp: Breakpoint) {
        if bp.persistent {
            // Persistent breakpoints always get inserted.
            // If a persistent breakpoint is already present in the same location, the new breakpoint will overwrite the old one.
            self.breakpoints.insert(location, bp);
        } else {
            // Temporary breakpoints are only inserted if no other breakpoint is present in the same location.
            let _ = self.breakpoints.try_insert(location, bp);
        }
    }

    pub fn remove_if_temporary(&mut self, location: usize) {
        let bp = self.breakpoints.get(&location).expect("Breakpoint should exist");
        if !bp.persistent {
            self.breakpoints.remove(&location).unwrap();
        }
    }

}


pub struct Debugger {

    shmem: Shmem,

    running_flag: *mut bool,
    terminate_command: *mut bool,
    vm_updated_counter: *mut u8,
    cpu_registers: *mut CPURegisters,
    vm_memory: *mut u8,

    last_persistent_breakpoint: Option<usize>,
    breakpoint_table: BreakpointTable,
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


    pub fn step_in(&mut self) {

        if self.is_running() {
            return;
        }

        if let Some(last_bp) = self.last_persistent_breakpoint.take() {
            // If the previous instruction had a persistent breakpoint, restore it
            self.write_vm_memory(last_bp, ByteCodes::BREAKPOINT as u8);
        }

        // Set a breakpoint on the next instruction

        // Get current instruction
        // The VM just executed a breapoint instruction and incremented the pc by 1 byte
        // We thus need to get the actual instruction at pc-1
        let mut current_registers = unsafe { self.cpu_registers.read_volatile() };
        let replaced_instruction_pc = current_registers.pc() - 1;
        // Now we need to get the replaced operator from some breakpoint table and interpret the instruction to get the next pc
        let current_breakpoint = self.breakpoint_table.get(replaced_instruction_pc).expect("Breakpoint should be registered");
        let replaced_operator: ByteCodes = ByteCodes::from(current_breakpoint.replaced_value);
        let next_pc = calculate_next_pc(self.read_vm_memory(), replaced_instruction_pc, replaced_operator);
        // Set a breakpoint on the next instruction, if present
        if let Some(next_pc) = next_pc {
            // Register the new breakpoint in the table
            let next_bp = Breakpoint {
                name: None,
                location: next_pc,
                replaced_value: self.read_vm_memory()[next_pc],
                persistent: false
            };
            self.breakpoint_table.insert(next_pc, next_bp);
            self.write_vm_memory(next_pc, ByteCodes::BREAKPOINT as u8);
        }

        // Decrement the pc to execute the current instruction that was replaced by the breakpoint
        current_registers.set(Registers::PROGRAM_COUNTER, replaced_instruction_pc as u64);
        unsafe {
            self.cpu_registers.write_volatile(current_registers);
        }
        // Ensure the current pc has a breakpoint
        assert_eq!(self.read_vm_memory()[replaced_instruction_pc], ByteCodes::BREAKPOINT as u8, "The replaced instruction pc should have a breakpoint set");

        // Restore the operator overwritten by the just-executed breakpoint
        self.write_vm_memory(replaced_instruction_pc, replaced_operator as u8);
        // If the current breakpoint is temporary, remove it from the table
        self.breakpoint_table.remove_if_temporary(replaced_instruction_pc);

        if current_breakpoint.persistent {
            self.last_persistent_breakpoint = Some(replaced_instruction_pc);
        }

        // Continue execution. The VM will stop at the next instruction because we set a breakpoint.

        self.resume_vm();
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


    fn write_vm_memory(&self, address: usize, value: u8) {
        unsafe {
            self.vm_memory.byte_add(address).write_volatile(value);
        }
    }


    pub fn read_registers(&self) -> CPURegisters {
        assert!(!self.is_running(), "Should not read registers while the VM is running");
        unsafe {
            self.cpu_registers.read_volatile()
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
            vm_memory,
            last_persistent_breakpoint: None,
            breakpoint_table: BreakpointTable::default(),
        })
    }

}


fn calculate_next_pc(vm_mem: &[u8], operator_pc: usize, operator: ByteCodes) -> Option<usize> {
    match operator {
        ByteCodes::EXIT => None,
        ByteCodes::JUMP => {
            todo!("interpret all the jump instructions to calculate the next pc")
        }
        _ => Some(operator_pc + 1)
    }
}
