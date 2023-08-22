use crate::allocator::Allocator;

use rust_vm_lib::vm::{Address, ErrorCodes};


/// Minimum allocation block size in bytes
const MIN_BLOCK_SIZE: usize = 64;


/// Represents a memory block
struct Block {

    pub size: usize,
    pub address: Address,
    pub free: bool,
    pub children: Option<(Box<Block>, Box<Block>)>,

}


impl Block {

    pub fn new(size: usize, address: Address) -> Block {
        Block {
            size,
            address,
            free: true,
            children: None,
        }
    }


    pub fn can_split(&self) -> bool {
        // Assume blocks are always a power of 2
        self.size > MIN_BLOCK_SIZE
    }


    pub fn split(&mut self) {
        debug_assert!(self.can_split());
        debug_assert!(self.children.is_none());
        debug_assert!(self.free);

        let half_size = self.size / 2;

        self.children = Some((
            Box::new(Block::new(half_size, self.address)),
            Box::new(Block::new(half_size, self.address + half_size)),
        ));
    }


    /// Propagate the memory allocation to the children, assuming the block has children
    fn propagate_children_allocation(&mut self, size: usize) -> Option<Address> {
        debug_assert!(self.children.is_some());

        let children = self.children.as_mut().unwrap();
            
        if let Some(address) = children.0.propagate_allocate(size) {
            Some(address)
        } else {
            children.1.propagate_allocate(size)
        }
    }


    /// Propagate the allocation down the tree
    pub fn propagate_allocate(&mut self, size: usize) -> Option<Address> {

        if !self.free {
            return None;
        }

        if self.children.is_some() {
            return self.propagate_children_allocation(size);
        }

        match self.size.cmp(&size) {

            // The block is too small, so return None
            std::cmp::Ordering::Less => None,

            // The block is exactly the right size, so allocate it whole
            std::cmp::Ordering::Equal => {
                self.free = false;
                Some(self.address)
            },

            // The block is larget than needed, check if it can be split
            std::cmp::Ordering::Greater => {

                if self.can_split() && self.size / 2 >= size {
                    // The block can be split further, so split it and propagate the allocation
                    self.split();
                    self.propagate_children_allocation(size)

                } else {
                    // The block cannot be split further, so return the entire block
                    self.free = false;
                    Some(self.address)
                }
            },
        }
    }


    /// Propagate the free operation down the tree
    /// Coalesce adjacent free blocks if possible
    pub fn propagate_free(&mut self, address: Address) -> Result<(), ErrorCodes> {
        
        if let Some(children) = self.children.as_mut() {

            if address >= children.1.address {
                // The address is in the second child
                children.1.propagate_free(address)?;
            } else {
                // The address is in the first child
                children.0.propagate_free(address)?;
            }

            // Check if the children can be coalesced
            if children.0.free && children.1.free && children.0.children.is_none() && children.1.children.is_none() {
                self.children = None;
            }

            return Ok(());
        } 

        // The block has no children, so it must be the block we are looking for

        debug_assert_eq!(self.address, address);

        if self.free {
            return Err(ErrorCodes::DoubleFree);
        }

        self.free = true;

        Ok(())
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


    pub fn allocate(&mut self, size: usize) -> Option<Address> {
        self.root.propagate_allocate(size)
    }


    pub fn free(&mut self, address: Address) -> Result<(), ErrorCodes> {
        self.root.propagate_free(address)
    }

}


pub struct BuddyAllocator {

    blocks: Btree,

    heap_start: Address,
    heap_end: Address,

}


impl BuddyAllocator {

    pub fn new(heap_start: Address, heap_size: usize) -> BuddyAllocator {
        
        // Assert that the heap size is a power of 2 (should be)
        debug_assert!(heap_size.is_power_of_two());

        // Assert that the heap size is at least the minimum block size
        if heap_size < MIN_BLOCK_SIZE {
            panic!("Heap size ({}) is too small to fit the minimum block size ({})", heap_size, MIN_BLOCK_SIZE);
        }

        BuddyAllocator { 
            blocks: Btree::new(heap_size, heap_start),
            heap_start,
            heap_end: heap_start + heap_size,
        }
    }


    #[inline(always)]
    fn heap_size(&self) -> usize {
        self.heap_end - self.heap_start
    }

}


impl Allocator for BuddyAllocator {

    fn allocate(&mut self, size: usize) -> Result<Address, ErrorCodes> {
        
        // Don't allow  empty allocations and allocations larger than the heap size
        if size > self.heap_size() || size == 0 {
            return Err(ErrorCodes::AllocationTooLarge);
        }

        self.blocks.allocate(size).ok_or(ErrorCodes::HeapOverflow)
    }


    fn free(&mut self, address: Address) -> Result<(), ErrorCodes> {

        // Check if the address is in the heap
        if address < self.heap_start || address >= self.heap_end {
            return Err(ErrorCodes::OutOfBounds);
        }

        // Check if the address is block aligned
        if (address - self.heap_start) % MIN_BLOCK_SIZE != 0 {
            return Err(ErrorCodes::UnalignedAddress);
        }

        self.blocks.free(address)
    }


    fn get_heap_end(&self) -> Address {
        self.heap_end
    }

}


#[cfg(test)]
mod tests {

    use super::*;


    #[test]
    fn test_allocate_zero() {
        let mut allocator = BuddyAllocator::new(0, 1024);

        assert!(matches!(allocator.allocate(0), Err(ErrorCodes::AllocationTooLarge)));
    }


    #[test]
    fn test_allocate_too_large() {
        let mut allocator = BuddyAllocator::new(0, 1024);

        assert!(matches!(allocator.allocate(1025), Err(ErrorCodes::AllocationTooLarge)));
    }


    #[test]
    fn test_allocate() {
        let mut allocator = BuddyAllocator::new(0, 1024);

        let address = allocator.allocate(64).unwrap();

        assert_eq!(address, 0);
    }


    #[test]
    fn test_free() {
        let mut allocator = BuddyAllocator::new(0, 1024);

        let address = allocator.allocate(64).unwrap();

        allocator.free(address).unwrap();
    }


    #[test]
    fn test_free_out_of_bounds() {
        let mut allocator = BuddyAllocator::new(0, 1024);

        let result = allocator.free(1024);

        assert!(matches!(result, Err(ErrorCodes::OutOfBounds)));
    }


    #[test]
    fn test_free_unaligned() {
        let mut allocator = BuddyAllocator::new(0, 1024);

        let result = allocator.free(1);

        assert!(matches!(result, Err(ErrorCodes::UnalignedAddress)));
    }


    #[test]
    fn test_free_double_free() {
        let mut allocator = BuddyAllocator::new(0, 1024);

        let address = allocator.allocate(64).unwrap();

        allocator.free(address).unwrap();

        let result = allocator.free(address);

        assert!(matches!(result, Err(ErrorCodes::DoubleFree)));
    }


    #[test]
    fn test_allocate_many() {
        let mut allocator = BuddyAllocator::new(0, 1024);

        let mut addresses = Vec::new();

        for i in 0..16 {
            let address = allocator.allocate(64).unwrap();
            assert_eq!(address, i * 64);
            addresses.push(address);
        }

        for address in addresses {
            allocator.free(address).unwrap();
        }
    }


    #[test]
    fn test_allocate_too_many() {
        let mut allocator = BuddyAllocator::new(0, 1024);

        let mut addresses = Vec::new();

        for _ in 0..16 {
            let address = allocator.allocate(64).unwrap();
            addresses.push(address);
        }

        let result = allocator.allocate(64);

        assert!(matches!(result, Err(ErrorCodes::HeapOverflow)));
    }


    #[test]
    fn test_free_reverse() {
        let mut allocator = BuddyAllocator::new(0, 1024);

        let mut addresses = Vec::new();

        for _ in 0..16 {
            let address = allocator.allocate(64).unwrap();
            addresses.push(address);
        }

        for address in addresses.into_iter().rev() {
            allocator.free(address).unwrap();
        }
    }


    #[test]
    fn test_free_coalesce() {
        let mut allocator = BuddyAllocator::new(0, 1024);

        let mut addresses = Vec::new();

        for _ in 0..16 {
            let address = allocator.allocate(64).unwrap();
            addresses.push(address);
        }

        for address in addresses {
            allocator.free(address).unwrap();
        }

        let address = allocator.allocate(1024).unwrap();

        assert_eq!(address, 0);
    }


    #[test]
    fn test_allocate_random_sizes_small() {
        use rand::Rng;

        let mut allocator = BuddyAllocator::new(0, 1024);

        let mut addresses = Vec::new();

        let mut rng = rand::thread_rng();

        for _ in 0..16 {
            let size = rng.gen_range(1..64);
            let address = allocator.allocate(size).unwrap();
            addresses.push(address);
        }

        for address in addresses {
            allocator.free(address).unwrap();
        }
    }


    #[test]
    fn test_allocate_random_sizes_large() {
        use rand::Rng;

        let mut allocator = BuddyAllocator::new(0, 1024 * 16);

        let mut addresses = Vec::new();

        let mut rng = rand::thread_rng();

        for _ in 0..16 {
            let size = rng.gen_range(1..1024);
            let address = allocator.allocate(size).unwrap();
            addresses.push(address);
        }

        for address in addresses {
            allocator.free(address).unwrap();
        }
    }

}

