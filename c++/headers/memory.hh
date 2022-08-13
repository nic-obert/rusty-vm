#pragma once

#include "utils.hh"


namespace memory {

    typedef uint64 Address;


    class Memory {
        private:
            size_t stackSize;
            size_t videoSize;
            Byte* stack = nullptr;
            Byte* video = nullptr;

        public:
            Memory(size_t stackSize, size_t videoSize);
            Memory() = delete;
            ~Memory();

            void setByte(Address address, Byte data);
            void setBytes(Address address, const Byte* data, size_t size);

            Byte getByte(Address address) const;
            const Byte* getBytes(Address addrss, size_t size) const;
            Byte* getBytesMutable(Address address);

    };

}

