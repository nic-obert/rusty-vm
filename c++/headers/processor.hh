#pragma once

#include "registers.hh"
#include "utils.hh"
#include "memory.hh"
#include "byte_code.hh"


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

            const Byte* nextByteCode(Byte size);

            void pushStack(uint64 value);
            void pushStackBytes(Byte* bytes, size_t size);
            Byte* popStackBytes(size_t size);

            constexpr uint64* getRegister(Registers reg) {
                return &registers[static_cast<Byte>(reg)];
            }

            // Instruction handlers

            void handle_add();
            void handle_sub();
            void handle_mul();
            void handle_div();
            void handle_mod();

            void handle_inc_reg();
            void handle_inc_addr_in_reg();
            void handle_inc_addr_literal();

            void handle_dec_reg();
            void handle_dec_addr_in_reg();
            void handle_dec_addr_literal();

            void handle_no_operation();

            void handle_move_reg_reg();
            void handle_move_reg_addr_in_reg();
            void handle_move_reg_const();
            void handle_move_reg_addr_literal();
            void handle_move_addr_in_reg_reg();
            void handle_move_addr_in_reg_addr_in_reg();
            void handle_move_addr_in_reg_const();
            void handle_move_addr_in_reg_addr_literal();
            void handle_move_addr_literal_reg();
            void handle_move_addr_literal_addr_in_reg();
            void handle_move_addr_literal_const();
            void handle_move_addr_literal_addr_literal();

            void handle_push_reg();
            void handle_push_addr_in_reg();
            void handle_push_const();
            void handle_push_addr_literal();

            void handle_pop_reg();
            void handle_pop_addr_in_reg();
            void handle_pop_addr_literal();

            // Labels don't get handled

            void handle_jump();
            void handle_jump_if_true_reg();
            void handle_jump_if_false_reg();

            void handle_compare_reg_reg(); 
            void handle_compare_reg_const();
            void handle_compare_const_reg();
            void handle_compare_const_const();

            void handle_print();
            void handle_print_string();

            void handle_input_int();
            void handle_input_string();

            void handle_exit();


            typedef void (processor::Processor::*InstructionHandler)();

            // Use constexpr to initialize the lookup table at compile time
            static constexpr InstructionHandler const INSTRUCTION_HANDLERS[BYTE_CODES_COUNT] = {

                handle_add,
                handle_sub,
                handle_mul,
                handle_div,
                handle_mod,

                handle_inc_reg,
                handle_inc_addr_in_reg,
                handle_inc_addr_literal,

                handle_dec_reg,
                handle_dec_addr_in_reg,
                handle_dec_addr_literal,

                handle_no_operation,

                handle_move_reg_reg,
                handle_move_reg_addr_in_reg,
                handle_move_reg_const,
                handle_move_reg_addr_literal,
                handle_move_addr_in_reg_reg,
                handle_move_addr_in_reg_addr_in_reg,
                handle_move_addr_in_reg_const,
                handle_move_addr_in_reg_addr_literal,
                handle_move_addr_literal_reg,
                handle_move_addr_literal_addr_in_reg,
                handle_move_addr_literal_const,
                handle_move_addr_literal_addr_literal,

                handle_push_reg,
                handle_push_addr_in_reg,
                handle_push_const,
                handle_push_addr_literal,

                handle_pop_reg,
                handle_pop_addr_in_reg,
                handle_pop_addr_literal,

                nullptr, // Labels don't get handled

                handle_jump,
                handle_jump_if_true_reg,
                handle_jump_if_false_reg,

                handle_compare_reg_reg, 
                handle_compare_reg_const,
                handle_compare_const_reg,
                handle_compare_const_const,

                handle_print,
                handle_print_string,

                handle_input_int,
                handle_input_string,

                handle_exit
            };


        public:

            Processor(size_t memorySize);
            Processor() = delete;
            ~Processor();

            void execute(Byte* byteCode, size_t size);

    };

}

