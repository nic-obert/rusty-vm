#pragma once

#include "utils.hh"


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

        MOVE_REG_REG,
        MOVE_REG_ADDR_IN_REG,
        MOVE_REG_CONST,
        MOVE_REG_ADDR_LITERAL,
        MOVE_ADDR_IN_REG_REG,
        MOVE_ADDR_IN_REG_ADDR_IN_REG,
        MOVE_ADDR_IN_REG_CONST,
        MOVE_ADDR_IN_REG_ADDR_LITERAL,
        MOVE_ADDR_LITERAL_REG,
        MOVE_ADDR_LITERAL_ADDR_IN_REG,
        MOVE_ADDR_LITERAL_CONST,
        MOVE_ADDR_LITERAL_ADDR_LITERAL,

        // Stack

        PUSH_REG,
        PUSH_ADDR_IN_REG,
        PUSH_CONST,
        PUSH_ADDR_LITERAL,

        POP_REG,
        POP_ADDR_IN_REG,
        POP_ADDR_LITERAL,

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
        PRINT_STRING,

        INPUT_INT,
        INPUT_STRING,

        EXIT,

        ENUM_COUNT

    };


    #define BYTE_CODES_COUNT static_cast<Byte>(ByteCodes::ENUM_COUNT)


    bool isJumpInstruction(ByteCodes instruction);


    const char* getInstructionName(ByteCodes instruction);

}

