pub mod fixed_size_block_allocator;


use rust_vm_lib::vm::Address;


pub trait Allocator {

    /// Allocate a block of memory of the given size and return its address
    /// If the allocation fails, return None
    fn allocate(&mut self, size: usize) -> Option<Address>;

    /// Free the block of memory at the given address
    /// Return whether the operation was successful
    fn free(&mut self, address: Address) -> bool;

}


pub struct BlankAllocator {

}


impl Allocator for BlankAllocator {

    fn allocate(&mut self, _size: usize) -> Option<Address> {
        unimplemented!()
    }

    fn free(&mut self, _address: Address) -> bool {
        unimplemented!()
    }

}

