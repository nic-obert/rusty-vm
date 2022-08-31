#include "memory.hh"
#include <string.h>


using namespace memory;


Memory::Memory(size_t stackSize, size_t videoSize) {
    this->stackSize = stackSize;
    this->videoSize = videoSize;
    this->stack = new Byte[this->stackSize];
    this->video = new video::Pixel[this->videoSize];
}


Memory::~Memory()
{
    delete[] this->stack;
    delete[] this->video;
}


void Memory::setByte(Address address, Byte data) {
    this->stack[address] = data;
}


void Memory::setBytes(Address address, const Byte* data, size_t size) {
    memcpy(this->stack + address, data, size);
}


Byte Memory::getByte(Address address) const {
    return this->stack[address];
}


const Byte* Memory::getBytes(Address address, size_t size) const {
    Byte* data = new Byte[size];
    memcpy(data, this->stack + address, size);
    return data;
}


Byte* Memory::getBytesMutable(Address address) {
    return this->stack + address;
}


void Memory::setPixel(video::VAddress address, const video::Pixel& pixel) {
    this->video[address] = video::Pixel(pixel.r, pixel.g, pixel.b);
}


void Memory::setPixels(video::VAddress address, const video::Pixel* pixels, size_t count) {
    memcpy(this->video + address, pixels, count * sizeof(video::Pixel));   
}


video::Pixel Memory::getPixel(video::VAddress address) const {
    return this->video[address];
}


const video::Pixel* Memory::getPixels(video::VAddress address, size_t count) const {
    video::Pixel* data = new video::Pixel[count];
    memcpy(data, this->video + address, count * sizeof(video::Pixel));
    return data;
}


video::Pixel* Memory::getPixelsMutable(video::VAddress address) {
    return (video::Pixel*)(this->video + address);
}

