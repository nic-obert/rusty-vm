use std::cmp::min;

use rust_vm_lib::vm::Address;

use crate::allocator::{Allocator, BlankAllocator};
use crate::allocator::fixed_size_block_allocator::FixedSizeBlockAllocator;


pub type Byte = u8;


/// Virtual memory module for the VM
pub struct Memory {

    memory: Vec<Byte>,
    allocator: Box<dyn Allocator>,
    max_size: usize,

}


impl Memory {

    pub fn new(max_size: Option<usize>) -> Memory {
        Memory {
            memory: vec![],
            allocator: Box::new(BlankAllocator{}),
            max_size: max_size.unwrap_or(usize::MAX),
        }
    }


    /// Initialize the memory layout
    /// This function should be called before any other memory function
    /// Allocate a memory chunk large enough to hold the program, stack, and heap
    pub fn init_layout(&mut self, static_program_end: Address) {

        // The heap is located after the static program section
        let heap_start = static_program_end;
        
        // Give the stack 1/4 of the available memory
        // Give the heap 3/4 of the available memory
        let stack_size = min(static_program_end * 4, self.max_size / 4);
        let heap_size = stack_size * 3;

        // Resize the memory to fit the program, stack, and heap
        self.memory.resize(stack_size + heap_size, 0);

        // Initialize the allocator with the new program size info
        self.allocator = Box::new(FixedSizeBlockAllocator::new(heap_start, heap_size));
    }


    pub fn allocate(&mut self, size: usize) -> Option<Address> {
        self.allocator.allocate(size)
    }


    pub fn free(&mut self, address: Address) -> bool {
        self.allocator.free(address)
    }


    pub fn get_raw(&self) -> &[Byte] {
        &self.memory
    }


    pub fn set_bytes(&mut self, address: Address, data: &[Byte]) {
        self.memory[address..address + data.len()].copy_from_slice(data);
    }


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

