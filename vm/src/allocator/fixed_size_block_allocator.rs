use crate::allocator::Allocator;

use rust_vm_lib::vm::{Address, ErrorCodes};


pub struct FixedSizeBlockAllocator {

    /// Keeps track of which blocks are taken
    blocks: Vec<bool>,
    heap_start: Address,
    heap_end: Address,

}


impl FixedSizeBlockAllocator {

    const BLOCK_SIZE: usize = 64;


    pub fn new(heap_start: Address, heap_size: usize) -> FixedSizeBlockAllocator {
        let block_count = heap_size / Self::BLOCK_SIZE;

        FixedSizeBlockAllocator {
            blocks: vec![false; block_count],
            heap_start,
            heap_end: heap_start + heap_size,
        }
    }

}


impl Allocator for FixedSizeBlockAllocator {


    fn allocate(&mut self, size: usize) -> Result<Address, ErrorCodes> {
        
        // Don't allow allocations bigger than one block
        if size > Self::BLOCK_SIZE || size == 0 {
            return Err(ErrorCodes::AllocationTooLarge);
        }

        // Find free block
        let address = self.blocks.iter_mut().enumerate().find(
            |(_index, &mut taken)| !taken
        ).map(
            |(_index, taken)| {
                *taken = true;
                self.heap_start + _index * Self::BLOCK_SIZE
            }
        );

        // If no free block was found, return an error
        address.ok_or(ErrorCodes::HeapOverflow)
    }


    fn free(&mut self, address: Address) -> Result<(), ErrorCodes> {
        
        // Check if the address is in the heap
        if address < self.heap_start || address >= self.heap_end {
            return Err(ErrorCodes::OutOfBounds);
        }

        // Check if the address is block aligned
        if (address - self.heap_start) % Self::BLOCK_SIZE != 0 {
            return Err(ErrorCodes::UnalignedAddress);
        }

        let block_index = (address - self.heap_start) / Self::BLOCK_SIZE;

        // Check if the block is already free
        if !self.blocks[block_index] {
            return Err(ErrorCodes::DoubleFree);
        }

        self.blocks[block_index] = false;

        Ok(())
    }

}

