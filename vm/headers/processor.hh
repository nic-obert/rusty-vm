#pragma once

#include "registers.hh"
#include "utils.hh"
#include "memory.hh"
#include "byte_code.hh"
#include "errors.hh"


using namespace registers;
using namespace memory;
using namespace byte_code;


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
            bool running = false;

            // Useful methods

            void clearVolatileRegisters();
            void setArithmeticalFlags(int64 result, uint64 remainder);

            Byte nextByteCode();
            const Byte* nextByteCode(Byte size);
            Address addressFromByteCode();

            void pushStack(uint64 value);
            void pushStackBytes(const Byte* bytes, size_t size);
            const Byte* popStackBytes(size_t size);

            constexpr inline uint64* getRegister(Registers reg);

            // Increments an unsigned integer and updates the arithmetical flags
            void incrementUnsigned(Byte* bytes, Byte size);
            // Decrements an unsigned integer and updates the arithmetical flags
            void decrementUnsigned(Byte* bytes, Byte size);

            void moveBytesIntoRegister(const Byte* bytes, Byte size, Registers reg);
            void moveRegisterIntoAddress(const Registers reg, const Address address, const Byte size);

            void run();
            void runVerbose();

            // Instruction handlers

            inline void handle_add();
            inline void handle_sub();
            inline void handle_mul();
            inline void handle_div();
            inline void handle_mod();

            inline void handle_inc_reg();
            inline void handle_inc_addr_in_reg();
            inline void handle_inc_addr_literal();

            inline void handle_dec_reg();
            inline void handle_dec_addr_in_reg();
            inline void handle_dec_addr_literal();

            inline void handle_no_operation();

            inline void handle_move_into_reg_from_reg();
            inline void handle_move_into_reg_from_addr_in_reg();
            inline void handle_move_into_reg_from_const();
            inline void handle_move_into_reg_from_addr_literal();
            inline void handle_move_into_addr_in_reg_from_reg();
            inline void handle_move_into_addr_in_reg_from_addr_in_reg();
            inline void handle_move_into_addr_in_reg_from_const();
            inline void handle_move_into_addr_in_reg_from_addr_literal();
            inline void handle_move_into_addr_literal_from_reg();
            inline void handle_move_into_addr_literal_from_addr_in_reg();
            inline void handle_move_into_addr_literal_from_const();
            inline void handle_move_into_addr_literal_from_addr_literal();

            inline void handle_push_from_reg();
            inline void handle_push_from_addr_in_reg();
            inline void handle_push_from_const();
            inline void handle_push_from_addr_literal();

            inline void handle_pop_into_reg();
            inline void handle_pop_into_addr_in_reg();
            inline void handle_pop_into_addr_literal();

            // Labels don't get handled

            inline void handle_jump();
            inline void handle_jump_if_true_reg();
            inline void handle_jump_if_false_reg();

            inline void handle_compare_reg_reg(); 
            inline void handle_compare_reg_const();
            inline void handle_compare_const_reg();
            inline void handle_compare_const_const();

            inline void handle_print();
            inline void handle_print_string();

            inline void handle_input_int();
            inline void handle_input_string();

            inline void handle_exit();

            inline void handle_instruction(Byte instruction);

        public:

            Processor(size_t stackSize, size_t videoSize);
            Processor() = delete;
            ~Processor();

            error::ErrorCodes execute(Byte* byteCode, size_t size, bool verbose = false);

    };

}

