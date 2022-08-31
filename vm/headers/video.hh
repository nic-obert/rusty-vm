#pragma once

#include "utils.hh"
#include "memory.hh"


namespace video {

    typedef struct Pixel {
        Byte r;
        Byte g;
        Byte b;

        Pixel(Byte r, Byte g, Byte b)
        : r(r), g(g), b(b)
        { }

        Pixel()
        : r(0), g(0), b(0)
        { }

    } Pixel;

    typedef uint64 VAddress;

};

