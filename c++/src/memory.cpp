#include "memory.hh"
#include <string.h>


using namespace memory;


Memory::Memory(size_t stackSize, size_t videoSize) {
    this->stackSize = stackSize;
    this->videoSize = videoSize;
    this->stack = new Byte[this->stackSize];
    this->video = new Byte[this->videoSize];
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

