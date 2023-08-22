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

        // Check for heap overflow
        if size >= self.heap_end - self.heap_start {
            return Err(ErrorCodes::HeapOverflow);
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


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_allocate() {
        let mut allocator = FixedSizeBlockAllocator::new(0, 1024);

        let address = allocator.allocate(64).unwrap();

        assert_eq!(address, 0);
    }


    #[test]
    fn test_allocate_too_large() {
        let mut allocator = FixedSizeBlockAllocator::new(0, 1024);

        let result = allocator.allocate(FixedSizeBlockAllocator::BLOCK_SIZE + 1);

        assert!(matches!(result, Err(ErrorCodes::AllocationTooLarge)));
    }


    #[test]
    fn test_allocate_zero() {
        let mut allocator = FixedSizeBlockAllocator::new(0, 1024);

        let result = allocator.allocate(0);

        assert!(matches!(result, Err(ErrorCodes::AllocationTooLarge)));
    }


    #[test]
    fn test_allocate_heap_overflow() {
        let mut allocator = FixedSizeBlockAllocator::new(0, 64);

        let result = allocator.allocate(64);

        assert!(matches!(result, Err(ErrorCodes::HeapOverflow)));
    }


    #[test]
    fn test_free() {
        let mut allocator = FixedSizeBlockAllocator::new(0, 1024);

        let address = allocator.allocate(64).unwrap();

        allocator.free(address).unwrap();
    }


    #[test]
    fn test_free_out_of_bounds() {
        let mut allocator = FixedSizeBlockAllocator::new(0, 1024);

        let result = allocator.free(1024);

        assert!(matches!(result, Err(ErrorCodes::OutOfBounds)));
    }


    #[test]
    fn test_free_unaligned() {
        let mut allocator = FixedSizeBlockAllocator::new(0, 1024);

        let result = allocator.free(1);

        assert!(matches!(result, Err(ErrorCodes::UnalignedAddress)));
    }


    #[test]
    fn test_free_double_free() {
        let mut allocator = FixedSizeBlockAllocator::new(0, 1024);

        let address = allocator.allocate(64).unwrap();

        allocator.free(address).unwrap();

        let result = allocator.free(address);

        assert!(matches!(result, Err(ErrorCodes::DoubleFree)));
    }


    #[test]
    fn test_many_allocations() {
        let heap_size = 1024;
        let mut allocator = FixedSizeBlockAllocator::new(0, heap_size);

        let mut addresses = Vec::new();

        for i in 0..heap_size / FixedSizeBlockAllocator::BLOCK_SIZE {
            let address = allocator.allocate(34).unwrap();
            assert_eq!(address, i * FixedSizeBlockAllocator::BLOCK_SIZE);
            addresses.push(address);
        }

        for address in addresses {
            allocator.free(address).unwrap();
        }
    }


    #[test]
    fn test_too_many_allocations() {
        let heap_size = 1024;
        let mut allocator = FixedSizeBlockAllocator::new(0, heap_size);

        for _ in 0..heap_size / FixedSizeBlockAllocator::BLOCK_SIZE {
            allocator.allocate(64).unwrap();
        }

        let result = allocator.allocate(64);

        assert!(matches!(result, Err(ErrorCodes::HeapOverflow)));
    }

}

