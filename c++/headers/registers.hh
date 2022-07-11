#pragma once

#include "utils.hh"


namespace registers {

    enum class Registers : Byte {

        A,
        B,

        C,
        D,
        
        EXIT,
        INPUT,
        ERROR,
        PRINT,

        STACK_POINTER,
        PROGRAM_COUNTER,

        ZERO_FLAG,
        SIGN_FLAG,
        REMAINDER_FLAG,

        ENUM_COUNT

    };


    #define REGISTERS_COUNT static_cast<Byte>(Registers::ENUM_COUNT)


    const char* getRegisterName(Registers reg);


    Registers getRegisterByName(const char* name);

}

