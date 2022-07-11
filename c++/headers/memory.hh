#pragma once

#include "utils.hh"


namespace memory {

    typedef unsigned int Address;


    class Memory {
        private:
            size_t size;
            Byte* stack = nullptr;

        public:
            Memory(size_t size);
            ~Memory();

            void setByte(Address address, Byte data);
            void setBytes(Address address, Byte* data, size_t size);

            Byte getByte(Address address) const;
            Byte* getBytes(Address addrss, size_t size) const;

    };

}

