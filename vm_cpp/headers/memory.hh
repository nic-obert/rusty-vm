#pragma once

#include "utils.hh"
#include "video.hh"


namespace memory {

    typedef uint64 Address;


    class Memory {
        private:
            size_t stackSize;
            size_t videoSize;
            Byte* stack = nullptr;
            video::Pixel* video = nullptr;

        public:
            Memory(size_t stackSize, size_t videoSize);
            Memory() = delete;
            ~Memory();

            void setByte(Address address, Byte data);
            void setBytes(Address address, const Byte* data, size_t size);

            Byte getByte(Address address) const;
            const Byte* getBytes(Address addrss, size_t size) const;
            Byte* getBytesMutable(Address address);

            void setPixel(video::VAddress address, const video::Pixel& pixel);
            void setPixels(video::VAddress address, const video::Pixel* pixels, size_t count);

            video::Pixel getPixel(video::VAddress address) const;
            const video::Pixel* getPixels(video::VAddress address, size_t count) const;
            video::Pixel* getPixelsMutable(video::VAddress address);
    };

}

