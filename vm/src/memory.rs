use crate::video::Pixel;


pub type Byte = u8;
pub type Address = usize;
pub type Size = usize;


pub struct Memory {

    stack_size: Size,
    video_size: Size,

    stack: Vec<Byte>,
    video: Vec<Pixel>,

}


impl Memory {

    pub fn new(stack_size: Size, video_size: Size) -> Memory {
        Memory {
            stack_size: stack_size,
            video_size: video_size,
            stack: vec![0; stack_size as usize],
            video: vec![Pixel::new(); video_size as usize],
        }
    }


    pub fn set_byte(&mut self, address: Address, data: Byte) {
        self.stack[address] = data;
    }


    pub fn set_bytes(&mut self, address: Address, data: &[Byte]) {
        for (i, byte) in data.iter().enumerate() {
            self.stack[address + i] = *byte;
        }
    }


    pub fn get_byte(&self, address: Address) -> Byte {
        self.stack[address]
    }


    pub fn get_bytes(&self, address: Address, size: Size) -> &[Byte] {
        &self.stack[address..address + size]
    }


    pub fn get_bytes_mut(&mut self, address: Address, size: Size) -> &mut [Byte] {
        &mut self.stack[address..address + size]
    }


    pub fn set_pixel(&mut self, address: Address, data: Pixel) {
        self.video[address] = data;
    }


    pub fn set_pixels(&mut self, address: Address, data: &[Pixel]) {
        for (i, pixel) in data.iter().enumerate() {
            self.video[address + i] = (*pixel).clone();
        }
    }


    pub fn get_pixel(&self, address: Address) -> &Pixel {
        &self.video[address]
    }


    pub fn get_pixels(&self, address: Address, size: Size) -> &[Pixel] {
        &self.video[address..address + size]
    }


    pub fn get_pixels_mut(&mut self, address: Address, size: Size) -> &mut [Pixel] {
        &mut self.video[address..address + size]
    }

}

