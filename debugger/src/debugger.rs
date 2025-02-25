
use rusty_vm_lib::registers::CPURegisters;
use rusty_vm_lib::debugger::{CPU_REGISTERS_OFFSET, RUNNING_FLAG_OFFSET, TERMINATE_COMMAND_OFFSET, VM_MEM_OFFSET, VM_UPDATED_COUNTER_OFFSET};

use shared_memory::{Shmem, ShmemConf, ShmemError};


pub struct Debugger {

    shmem: Shmem,

    running_flag: *mut bool,
    terminate_command: *mut bool,
    vm_updated_counter: *mut u8,
    cpu_registers: *mut CPURegisters,
    // vm_memory: ()

}

impl Debugger {

    pub fn vm_memory_size(&self) -> usize {
        self.shmem.len() - VM_MEM_OFFSET
    }


    pub fn resume_vm(&self) {
        unsafe {
            self.running_flag.write_volatile(true);
        }
    }


    pub fn try_attach(shmem_id: String) -> Result<Self, ShmemError> {
        let shmem = ShmemConf::new()
            .os_id(shmem_id)
            .open()?;

        let running_flag = unsafe { shmem.as_ptr().byte_add(RUNNING_FLAG_OFFSET).cast::<bool>() };
        let terminate_command = unsafe { shmem.as_ptr().byte_add(TERMINATE_COMMAND_OFFSET).cast::<bool>() };
        let vm_updated_counter = unsafe { shmem.as_ptr().byte_add(VM_UPDATED_COUNTER_OFFSET) };
        let cpu_registers = unsafe { shmem.as_ptr().byte_add(CPU_REGISTERS_OFFSET).cast::<CPURegisters>() };
        // let vm_memory = unsafe {
        //     let vm_mem_ptr = shmem.as_ptr().byte_add(VM_MEM_OFFSET);
        //     let vm_mem_size = shmem.len() - VM_MEM_OFFSET;
        //     slice::from_mut_ptr_range(range)
        // };

        Ok(Self {
            shmem,
            running_flag,
            terminate_command,
            vm_updated_counter,
            cpu_registers,
            // vm_memory: ()
        })
    }

}
