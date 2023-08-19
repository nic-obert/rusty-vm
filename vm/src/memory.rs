use rust_vm_lib::vm::Address;
//use crate::video::Pixel;


pub type Byte = u8;


/// Virtual memory module for the VM
pub struct Memory {

    memory: Vec<Byte>,
    heap_start: Address,
    heap_end: Address,

}


impl Memory {

    pub fn new(size: usize) -> Memory {
        Memory {
            memory: vec![0; size],
            heap_start: 0,
            heap_end: 0,
        }
    }


    pub fn set_heap(&mut self, start_address: Address) {
        self.heap_start = start_address;
        self.heap_end = start_address;
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

