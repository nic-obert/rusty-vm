#include "memory.hh"
#include <string.h>


using namespace memory;


Memory::Memory(size_t size)
{
    this->size = size;
    this->stack = new Byte[size];
}


Memory::~Memory()
{
    delete[] this->stack;
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

