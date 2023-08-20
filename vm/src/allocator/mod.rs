pub mod fixed_size_block_allocator;


use rust_vm_lib::vm::{Address, ErrorCodes};


pub trait Allocator {

    /// Allocate a block of memory of the given size and return its address
    /// If the allocation fails, return an error code
    fn allocate(&mut self, size: usize) -> Result<Address, ErrorCodes>;

    /// Free the block of memory at the given address
    /// If the operation was unsuccessful, return an error code
    fn free(&mut self, address: Address) -> Result<(), ErrorCodes>;

}


pub struct BlankAllocator {

}


impl Allocator for BlankAllocator {

    fn allocate(&mut self, _size: usize) -> Result<Address, ErrorCodes> {
        unimplemented!()
    }

    fn free(&mut self, _address: Address) -> Result<(), ErrorCodes> {
        unimplemented!()
    }

}

