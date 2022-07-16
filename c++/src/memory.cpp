#include "memory.hh"


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
    for (size_t i = 0; i < size; i++) {
        this->stack[address + i] = data[i];
    }
}


Byte Memory::getByte(Address address) const {
    return this->stack[address];
}


const Byte* Memory::getBytes(Address address, size_t size) const {
    Byte* data = new Byte[size];
    for (size_t i = 0; i < size; i++) {
        data[i] = this->stack[address + i];
    }
    return data;
}


Byte* Memory::getBytesMutable(Address address) {
    return this->stack + address;
}

