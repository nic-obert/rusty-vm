use std::ptr::{drop_in_place, NonNull};

use rusty_vm_lib::vm::Address;


pub type Byte = u8;


/// Virtual memory module for the VM
pub struct Memory {

    memory: NonNull<[Byte]>,
    shared: bool

}

impl Drop for Memory {
    fn drop(&mut self) {
        if !self.shared {
            unsafe {
                drop_in_place(self.memory.as_ptr());
            }
        }
    }
}

impl Memory {

    pub fn new(max_size: usize) -> Memory {
        Memory {
            memory: NonNull::from_mut(Box::leak(vec![0; max_size].into_boxed_slice())),
            shared: false
        }
    }


    /// Get the start address of the stack, which is the end of the memory
    pub fn get_stack_base(&self) -> Address {
        self.size_bytes()
    }


    /// Get a reference to the raw memory, unadviced
    pub const fn get_raw(&self) -> &[Byte] {
        unsafe {
            self.memory.as_ref()
        }
    }


    pub fn set_bytes(&mut self, address: Address, data: &[Byte]) {
        unsafe {
            self.memory.as_mut()[address..address + data.len()].copy_from_slice(data);
        }
    }


    pub fn set_byte(&mut self, address: Address, data: Byte) {
        unsafe {
            self.memory.as_mut()[address] = data;
        }
    }


    /// Copy `size` bytes from `src_address` to `dest_address`.
    /// Implements safe buffred copying for overlapping memory regions.
    pub fn memcpy(&mut self, src_address: Address, dest_address: Address, size: usize) {
        unsafe {
            self.memory.as_mut().copy_within(src_address..src_address + size, dest_address);
        }
    }


    pub fn read<T>(&self, address: Address) -> T {
        unsafe {
            ((self.memory.as_ptr() as *const Byte as usize + address) as *const T)
            .read_unaligned()
        }
    }


    pub const fn get_byte(&self, address: Address) -> Byte {
        unsafe {
            self.memory.as_ref()[address]
        }
    }


    pub fn get_bytes(&self, address: Address, size: usize) -> &[Byte] {
        unsafe {
            &self.memory.as_ref()[address..address + size]
        }
    }


    pub fn get_bytes_mut(&mut self, address: Address, size: usize) -> &mut [Byte] {
        unsafe {
            &mut self.memory.as_mut()[address..address + size]
        }
    }


    pub fn size_bytes(&self) -> usize {
        self.memory.len()
    }


    /// Replace the current backing buffer with the provided one.
    /// Copy the current memory buffer into the new buffer.
    /// Correct deallocation of the new buffer is the caller's responsibility.
    pub unsafe fn set_shared_buffer(&mut self, mut new_buffer: NonNull<[Byte]>) {
        assert_eq!(new_buffer.len(), self.memory.len());
        unsafe {
            new_buffer.as_mut().copy_from_slice(self.memory.as_ref());
        }
        self.memory = new_buffer;
        self.shared = true;
    }

}


#[cfg(test)]
mod tests {

    use super::*;

    /// WARNING: leaks the allocated buffer
    macro_rules! leaky_mem {
        () => (
            NonNull::from_mut(Box::leak(vec![].into_boxed_slice()))
        );
        ($elem:expr; $n:expr) => (
            NonNull::from_mut(Box::leak(vec![$elem; $n].into_boxed_slice()))
        );
        ($($x:expr),+ $(,)?) => (
            NonNull::from_mut(Box::leak(vec![$($x),+].into_boxed_slice()))
        );
    }

    fn assert_mem_eq(a: NonNull<[Byte]>, b: NonNull<[Byte]>) {
        unsafe {
            if a.as_ref() != b.as_ref() {
                panic!("Memory buffers are different:\nLeft: {:?}\nRight: {:?}\n", a.as_ref(), b.as_ref());
            }
        }
    }


    #[test]
    fn test_overlapping_memcpy() {
        let mut memory = Memory::new(0);
        memory.memory = leaky_mem![0, 1, 2, 3, 4, 5, 6, 7];
        memory.memcpy(0, 4, 4);
        assert_mem_eq(memory.memory, leaky_mem![0, 1, 2, 3, 0, 1, 2, 3]);
    }


    #[test]
    fn test_non_overlapping_memcpy() {
        let mut memory = Memory::new(0);
        memory.memory = leaky_mem![0, 1, 2, 3, 4, 5, 6, 7];
        memory.memcpy(0, 4, 3);
        assert_mem_eq(memory.memory, leaky_mem![0, 1, 2, 3, 0, 1, 2, 7]);
    }


    #[test]
    fn test_memcpy_to_self() {
        let mut memory = Memory::new(0);
        memory.memory = leaky_mem![0, 1, 2, 3, 4, 5, 6, 7];
        memory.memcpy(0, 0, 4);
        assert_mem_eq(memory.memory, leaky_mem![0, 1, 2, 3, 4, 5, 6, 7]);
    }


    #[test]
    fn test_memcpy_to_self_overlapping() {
        let mut memory = Memory::new(0);
        memory.memory = leaky_mem![0, 1, 2, 3, 4, 5, 6, 7];
        memory.memcpy(0, 2, 4);
        assert_mem_eq(memory.memory, leaky_mem![0, 1, 0, 1, 2, 3, 6, 7]);
    }

}
