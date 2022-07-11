#pragma once

#include "registers.hh"
#include "utils.hh"
#include "memory.hh"


using namespace registers;
using namespace memory;


namespace processor {

    class Processor {
        private:
            uint64 registers[REGISTERS_COUNT] = {
                0,  // A
                0,  // B 
                0,  // C
                0,  // D
                0,  // EXIT
                0,  // INPUT
                0,  // ERROR
                0,  // PRINT
                0,  // STACK_POINTER
                0,  // PROGRAM_COUNTER
                0,  // ZERO_FLAG
                0,  // SIGN_FLAG
                0   // REMAINDER_FLAG
            };

            Memory memory;


            void clearVolatileRegisters();
            void setArithmeticalFlags(uint64 result, uint64 remainder);
            


        public:
            Processor();
            ~Processor();

            void execute(Byte* byteCode, size_t size);


    };

}

