use rusty_vm_lib::vm::Address;


pub type Byte = u8;


/// Virtual memory module for the VM
pub struct Memory {

    memory: Box<[Byte]>,

}


impl Memory {

    pub fn new(max_size: usize) -> Memory {
        Memory {
            memory: vec![0; max_size].into_boxed_slice(),
        }
    }


    /// Get the start address of the stack, which is the end of the memory
    pub fn get_stack_base(&self) -> Address {
        self.memory.len()
    }


    /// Get a reference to the raw memory, unadviced
    pub fn get_raw(&self) -> &[Byte] {
        &self.memory
    }


    pub fn set_bytes(&mut self, address: Address, data: &[Byte]) {
        self.memory[address..address + data.len()].copy_from_slice(data);
    }


    // pub fn set_byte(&mut self, address: Address, data: Byte) {
    //     self.memory[address] = data;
    // }


    /// Copy `size` bytes from `src_address` to `dest_address`.
    /// Implements safe buffred copying for overlapping memory regions.
    pub fn memcpy(&mut self, src_address: Address, dest_address: Address, size: usize) {
        self.memory.copy_within(src_address..src_address + size, dest_address);
    }


    // TODO: probably this is faster than get_bytes
    pub fn read<T>(&self, address: Address) -> T {
        unsafe {
            ((self.memory.as_ptr() as usize + address) as *const T)
            .read_unaligned()
        }
    }


    pub fn get_byte(&self, address: Address) -> Byte {
        self.memory[address]
    }


    pub fn get_bytes(&self, address: Address, size: usize) -> &[Byte] {
        &self.memory[address..address + size]
    }


    pub fn get_bytes_mut(&mut self, address: Address, size: usize) -> &mut [Byte] {
        &mut self.memory[address..address + size]
    }

}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_overlapping_memcpy() {
        let mut memory = Memory::new(0);
        memory.memory = vec![0, 1, 2, 3, 4, 5, 6, 7].into_boxed_slice();
        memory.memcpy(0, 4, 4);
        assert_eq!(memory.memory, vec![0, 1, 2, 3, 0, 1, 2, 3].into_boxed_slice());
    }


    #[test]
    fn test_non_overlapping_memcpy() {
        let mut memory = Memory::new(0);
        memory.memory = vec![0, 1, 2, 3, 4, 5, 6, 7].into_boxed_slice();
        memory.memcpy(0, 4, 3);
        assert_eq!(memory.memory, vec![0, 1, 2, 3, 0, 1, 2, 7].into_boxed_slice());
    }


    #[test]
    fn test_memcpy_to_self() {
        let mut memory = Memory::new(0);
        memory.memory = vec![0, 1, 2, 3, 4, 5, 6, 7].into_boxed_slice();
        memory.memcpy(0, 0, 4);
        assert_eq!(memory.memory, vec![0, 1, 2, 3, 4, 5, 6, 7].into_boxed_slice());
    }


    #[test]
    fn test_memcpy_to_self_overlapping() {
        let mut memory = Memory::new(0);
        memory.memory = vec![0, 1, 2, 3, 4, 5, 6, 7].into_boxed_slice();
        memory.memcpy(0, 2, 4);
        assert_eq!(memory.memory, vec![0, 1, 0, 1, 2, 3, 6, 7].into_boxed_slice());
    }    

}

