use crate::allocator::Allocator;

use rust_vm_lib::vm::{Address, ErrorCodes};


/// Represents a memory block
struct Block {

    pub size: usize,
    pub address: Address,
    pub children: Option<(Box<Block>, Box<Block>)>,

}


impl Block {

    pub fn new(size: usize, address: Address) -> Block {
        Block {
            size,
            address,
            children: None,
        }
    }

}


/// Keeps track of the free and allocated blocks
struct Btree {

    root: Block,

}


impl Btree {

    pub fn new(size: usize, address: Address) -> Btree {
        Btree {
            root: Block::new(size, address),
        }
    }

}


pub struct BuddyAllocator {

    blocks: Btree,

    heap_start: Address,
    heap_end: Address,

}


impl BuddyAllocator {

    /// Minimum allocation block size in bytes
    const MIN_BLOCK_SIZE: usize = 64;


    pub fn new(heap_start: Address, heap_size: usize) -> BuddyAllocator {
        
        // Assert that the heap size is a power of 2 (should be)
        debug_assert!(heap_size.is_power_of_two());

        // Assert that the heap size is at least the minimum block size
        if heap_size < Self::MIN_BLOCK_SIZE {
            panic!("Heap size ({}) is too small to fit the minimum block size ({})", heap_size, Self::MIN_BLOCK_SIZE);
        }

        BuddyAllocator { 
            blocks: Btree::new(heap_size, heap_start),
            heap_start,
            heap_end: heap_start + heap_size,
        }
    }

}


impl Allocator for BuddyAllocator {

    fn allocate(&mut self, size: usize) -> Result<Address, ErrorCodes> {
        todo!()
    }


    fn free(&mut self, address: Address) -> Result<(), ErrorCodes> {
        todo!()
    }


    fn get_heap_end(&self) -> Address {
        self.heap_end
    }

}

