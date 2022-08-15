#pragma once

#include "utils.hh"
#include "memory.hh"


namespace video {

    typedef struct Pixel {
        Byte r;
        Byte g;
        Byte b;

        Pixel(Byte r, Byte g, Byte b) {
            this->r = r;
            this->g = g;
            this->b = b;
        }

        Pixel() {
            this->r = 0;
            this->g = 0;
            this->b = 0;
        }

        Pixel(const Pixel& pixel) {
            this->r = pixel.r;
            this->g = pixel.g;
            this->b = pixel.b;
        }

    } Pixel;

    typedef uint64 VAddress;

};

