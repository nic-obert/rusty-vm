use rust_vm_lib::vm::{Address, ErrorCodes};

use crate::allocator::{Allocator, BlankAllocator};
use crate::allocator::buddy_allocator::BuddyAllocator;
use crate::error;


pub type Byte = u8;


/// Virtual memory module for the VM
pub struct Memory {

    memory: Vec<Byte>,
    allocator: Box<dyn Allocator>,
    /// Hard limit on the amount of memory that can be allocated.
    /// If None, there is no upper limit.
    max_size: Option<usize>,

}


impl Memory {

    pub fn new(max_size: Option<usize>) -> Memory {
        Memory {
            memory: vec![0; max_size.unwrap_or(0)],
            allocator: Box::new(BlankAllocator{}),
            max_size,
        }
    }


    /// Get the start address of the stack, which is the end of the memory
    pub fn get_stack_start(&self) -> Address {
        self.memory.len()
    }


    pub fn get_heap_end(&self) -> Address {
        self.allocator.get_heap_end()
    }


    /// Initialize the memory layout
    /// This function should be called before any other memory function
    /// Allocate a memory chunk large enough to hold the program, stack, and heap
    pub fn init_layout(&mut self, static_program_end: Address) {

        // TODO: use different allocators depending on the available memory
        // For example, if there are no limits on memory size, there's no need to pack the stack and heap together
        
        if let Some(max_size) = self.max_size {

            if static_program_end > max_size {
                error::error(format!("Static program section ({}) is larger than the maximum memory size ({})", static_program_end, max_size).as_str());
            }

            // The heap is located after the static program section
            let heap_start = static_program_end;

            let total_available_memory = max_size - heap_start;

            // As a rule of thumb, allocate roughly 1/4 of the available memory for the stack          
            // The heap size must be a power of 2 in order to use the buddy allocator

            let base_heap_size = total_available_memory / 4 * 3;
            
            // Calculate the nearest floor power of 2
            let highest_exp = base_heap_size.ilog2();

            if highest_exp == 0 {
                error::error(format!("Not enough memory to allocate the stack and heap: {} bytes", total_available_memory).as_str());
            }

            let heap_size = 2usize.pow(highest_exp);

            // Leave the rest of the memory for the stack

            // If the max size is defined, the memory was already allocated
            // We just need to initialize the allocator

            self.allocator = Box::new(BuddyAllocator::new(heap_start, heap_size));

        } else {
            unimplemented!("Memory layout initialization without a max size is not implemented, yet")
        }

    }


    /// Allocate the requested amount of memory
    /// Return the address of the allocated memory
    /// Return an error if the allocation failed
    pub fn allocate(&mut self, size: usize) -> Result<usize, ErrorCodes> {
        self.allocator.allocate(size)
    }


    /// Free the memory at the given address
    /// Return an error if the operation failed
    pub fn free(&mut self, address: Address) -> Result<(), ErrorCodes> {
        self.allocator.free(address)
    }


    /// Get a reference to the raw memory, unadviced
    pub fn get_raw(&self) -> &[Byte] {
        &self.memory
    }


    pub fn set_bytes(&mut self, address: Address, data: &[Byte]) {
        self.memory[address..address + data.len()].copy_from_slice(data);
    }


    /// Copy `size` bytes from `src_address` to `dest_address`.
    /// Implements safe buffred copying for overlapping memory regions.
    pub fn memcpy(&mut self, src_address: Address, dest_address: Address, size: usize) {
        self.memory.copy_within(src_address..src_address + size, dest_address);
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
        let mut memory = Memory::new(None);
        memory.memory = vec![0, 1, 2, 3, 4, 5, 6, 7];
        memory.memcpy(0, 4, 4);
        assert_eq!(memory.memory, vec![0, 1, 2, 3, 0, 1, 2, 3]);
    }


    #[test]
    fn test_non_overlapping_memcpy() {
        let mut memory = Memory::new(None);
        memory.memory = vec![0, 1, 2, 3, 4, 5, 6, 7];
        memory.memcpy(0, 4, 3);
        assert_eq!(memory.memory, vec![0, 1, 2, 3, 0, 1, 2, 7]);
    }


    #[test]
    fn test_memcpy_to_self() {
        let mut memory = Memory::new(None);
        memory.memory = vec![0, 1, 2, 3, 4, 5, 6, 7];
        memory.memcpy(0, 0, 4);
        assert_eq!(memory.memory, vec![0, 1, 2, 3, 4, 5, 6, 7]);
    }


    #[test]
    fn test_memcpy_to_self_overlapping() {
        let mut memory = Memory::new(None);
        memory.memory = vec![0, 1, 2, 3, 4, 5, 6, 7];
        memory.memcpy(0, 2, 4);
        assert_eq!(memory.memory, vec![0, 1, 0, 1, 2, 3, 6, 7]);
    }    

}

