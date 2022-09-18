#pragma once

#include "utils.hh"
#include <iostream>


namespace byte_code {

    enum class ByteCodes : Byte {

        // Arithmetic

        ADD,
        SUB,
        MUL,
        DIV,
        MOD,


        INC_REG,
        INC_ADDR_IN_REG,
        INC_ADDR_LITERAL,

        DEC_REG,
        DEC_ADDR_IN_REG,
        DEC_ADDR_LITERAL,

        // No operation

        NO_OPERATION,

        // Memory

        MOVE_INTO_REG_FROM_REG,
        MOVE_INTO_REG_FROM_ADDR_IN_REG,
        MOVE_INTO_REG_FROM_CONST,
        MOVE_INTO_REG_FROM_ADDR_LITERAL,
        MOVE_INTO_ADDR_IN_REG_FROM_REG,
        MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG,
        MOVE_INTO_ADDR_IN_REG_FROM_CONST,
        MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL,
        MOVE_INTO_ADDR_LITERAL_FROM_REG,
        MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG,
        MOVE_INTO_ADDR_LITERAL_FROM_CONST,
        MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL,

        // Stack

        PUSH_FROM_REG,
        PUSH_FROM_ADDR_IN_REG,
        PUSH_FROM_CONST,
        PUSH_FROM_ADDR_LITERAL,

        POP_INTO_REG,
        POP_INTO_ADDR_IN_REG,
        POP_INTO_ADDR_LITERAL,

        // Control flow

        LABEL,

        JUMP,
        JUMP_IF_TRUE_REG,
        JUMP_IF_FALSE_REG,

        // Comparison

        COMPARE_REG_REG,
        COMPARE_REG_CONST,
        COMPARE_CONST_REG,
        COMPARE_CONST_CONST,

        // Interrupts

        PRINT,
        PRINT_CHAR,
        PRINT_STRING,

        INPUT_INT,
        INPUT_STRING,

        EXIT,

        // ENUM_COUNT should always be the last element of the enum
        ENUM_COUNT

    };


    #define BYTE_CODES_COUNT static_cast<Byte>(ByteCodes::ENUM_COUNT)


    constexpr inline bool isJumpInstruction(ByteCodes instruction);


    constexpr inline const char* getInstructionName(ByteCodes instruction);

}


std::ostream& operator<<(std::ostream& os, byte_code::ByteCodes instruction);

