#![allow(unused)]


pub mod fixed_size_block_allocator;
pub mod buddy_allocator;


use rust_vm_lib::vm::{Address, ErrorCodes};


pub trait Allocator {

    /// Allocate a block of memory of the given size and return its address
    /// If the allocation fails, return an error code
    fn allocate(&mut self, size: usize) -> Result<Address, ErrorCodes>;

    /// Free the block of memory at the given address
    /// If the operation was unsuccessful, return an error code
    fn free(&mut self, address: Address) -> Result<(), ErrorCodes>;

    /// Return the end address of the heap
    fn get_heap_end(&self) -> Address;

}


/// A dummy allocator that does nothing
/// Can be used as a placeholder
pub struct BlankAllocator {

}


impl Allocator for BlankAllocator {

    fn allocate(&mut self, _size: usize) -> Result<Address, ErrorCodes> {
        unimplemented!()
    }

    fn free(&mut self, _address: Address) -> Result<(), ErrorCodes> {
        unimplemented!()
    }

    fn get_heap_end(&self) -> Address {
        unimplemented!()
    }

}

