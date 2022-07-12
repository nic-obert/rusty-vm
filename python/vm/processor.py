from __future__ import annotations
from typing import Callable, List, Union

from memory import Memory
from errors import Errors
from shared.byte_code import byte_code_names
from shared.registers import Registers


class Processor:

    def __init__(self, memory_size: int) -> None:
        self.memory = Memory(memory_size)
        self.running = False

        self.registers: List[Union[int, bool]] = \
        [
            0,      # A
            0,      # B
            0,      # C
            0,      # D
            0,      # Exit
            0,      # Input
            0,      # Error
            0,      # Print
            0,      # Stack Pointer
            0,      # Program Counter
            False,  # Zero Flag
            False,  # Sign Flag
            0,      # Remainder Flag
        ]
    

    def execute(self, byte_code: bytes, verbose: bool = False) -> None:
        """
        Execute the byte code.
        """
        # Load the byte code into memory
        self.push_stack_bytes(byte_code)

        self.running = True
        while self.running:
            # Fetch the instruction
            opcode = self.next_byte_code(1)

            if verbose:
                print(f'Instruction: {byte_code_names[opcode]}')

            # Execute the instruction
            self.instruction_handlers_table[opcode](self)

            # Clear after executing the instruction
            self.clear_volatile_registers()

        # Exit the program with the status code stored in E register.
        exit(self.registers[Registers.EXIT])  


    # Useful functions

    def clear_volatile_registers(self) -> None:
        self.registers[Registers.ERROR] = 0


    def set_arithmetical_flags(self, result: int, remainder: int = 0) -> None:
        self.registers[Registers.ZERO_FLAG] = result == 0
        self.registers[Registers.SIGN_FLAG] = result < 0
        self.registers[Registers.REMAINDER_FLAG] = remainder


    def next_byte_code(self, size: int) -> int:
        data = self.memory.get_data(self.registers[Registers.PROGRAM_COUNTER], size)
        self.registers[Registers.PROGRAM_COUNTER] += size
        return data

    
    def push_stack(self, value: int, size: int) -> None:
        self.memory.store_data(self.registers[Registers.STACK_POINTER], value, size)
        self.registers[Registers.STACK_POINTER] += size


    def push_stack_bytes(self, data: bytes) -> None:
        self.memory.store_bytes(self.registers[Registers.STACK_POINTER], data)
        self.registers[Registers.STACK_POINTER] += len(data)


    def pop_stack(self, size: int) -> int:
        self.registers[Registers.STACK_POINTER] -= size
        value = self.memory.get_data(self.registers[Registers.STACK_POINTER], size)
        return value


    # Instruction handlers


    def handle_add(self) -> None:
        self.registers[Registers.A] += self.registers[Registers.B]

        self.set_arithmetical_flags(self.registers[Registers.A])


    def handle_sub(self) -> None:
        self.registers[Registers.A] -= self.registers[Registers.B]

        self.set_arithmetical_flags(self.registers[Registers.A])


    def handle_mul(self) -> None:
        self.registers[Registers.A] *= self.registers[Registers.B]

        self.set_arithmetical_flags(self.registers[Registers.A])


    def handle_div(self) -> None:
        remainder = self.registers[Registers.A] % self.registers[Registers.B]
        self.registers[Registers.A] //= self.registers[Registers.B]

        self.set_arithmetical_flags(self.registers[Registers.A], remainder)


    def handle_mod(self) -> None:
        self.registers[Registers.A] %= self.registers[Registers.B]

        self.set_arithmetical_flags(self.registers[Registers.A])


    def handle_inc_reg(self) -> None:
        register = self.next_byte_code(1)
        self.registers[register] += 1
        
        self.set_arithmetical_flags(self.registers[register])

    
    def handle_inc_addr_in_reg(self) -> None:
        size = self.next_byte_code(1)
        register = self.next_byte_code(1)
        address = self.registers[register]
        value = self.memory.get_data(address, size) + 1
        self.memory.store_data(self.register, value, size)

        self.set_arithmetical_flags(value)


    def handle_inc_addr_literal(self) -> None:
        size = self.next_byte_code(1)
        address = self.next_byte_code(8)
        value = self.memory.get_data(address, size) + 1
        self.memory.store_data(address, value, size)

        self.set_arithmetical_flags(value)

    
    def handle_dec_reg(self) -> None:
        register = self.next_byte_code(1)
        self.registers[register] -= 1

        self.set_arithmetical_flags(self.registers[register])
    

    def handle_dec_addr_in_reg(self) -> None:
        size = self.next_byte_code(1)
        register = self.next_byte_code(1)
        address = self.registers[register]
        value = self.memory.get_data(address, size) - 1
        self.memory.store_data(address, value, size)

        self.set_arithmetical_flags(value)

    
    def handle_dec_addr_literal(self) -> None:
        size = self.next_byte_code(1)
        address = self.next_byte_code(8)
        value = self.memory.get_data(address, size) - 1
        self.memory.store_data(address, value, size)

        self.set_arithmetical_flags(value)

    
    def handle_no_operation(self) -> None:
        pass
    

    def handle_move_reg_reg(self) -> None:
        register1 = self.next_byte_code(1)
        register2 = self.next_byte_code(1)
        self.registers[register1] = self.registers[register2]
    

    def handle_move_reg_addr_in_reg(self) -> None:
        size = self.next_byte_code(1)
        register1 = self.next_byte_code(1)
        register2 = self.next_byte_code(1)
        address = self.registers[register2]
        self.registers[register1] = self.memory.get_data(address, size)
    

    def handle_move_reg_const(self) -> None:
        size = self.next_byte_code(1)
        register = self.next_byte_code(1)
        self.registers[register] = self.next_byte_code(size)

    
    def handle_move_reg_addr_literal(self) -> None:
        size = self.next_byte_code(1)
        register = self.next_byte_code(1)
        address = self.next_byte_code(8)
        self.registers[register] = self.memory.get_data(address, size)
    

    def handle_move_addr_in_reg_reg(self) -> None:
        size = self.next_byte_code(1)
        register1 = self.next_byte_code(1)
        address = self.registers[register1]
        register2 = self.next_byte_code(1)
        self.memory.store_data(address, self.registers[register2], size)
    

    def handle_move_addr_in_reg_addr_in_reg(self) -> None:
        size = self.next_byte_code(1)
        register1 = self.next_byte_code(1)
        address1 = self.registers[register1]
        register2 = self.next_byte_code(1)
        address2 = self.registers[register2]
        self.memory.store_data(address1, self.memory.get_data(address2, size), size)
        

    def handle_move_addr_in_reg_const(self) -> None:
        size = self.next_byte_code(1)
        register = self.next_byte_code(1)
        address = self.registers[register]
        value = self.next_byte_code(size)
        self.memory.store_data(address, value, size)
    

    def handle_move_addr_in_reg_addr_literal(self) -> None:
        size = self.next_byte_code(1)
        register = self.next_byte_code(1)
        address1 = self.registers[register]
        address2 = self.next_byte_code(8)
        self.memory.store_data(address1, self.memory.get_data(address2, size), size)

    
    def handle_move_addr_literal_reg(self) -> None:
        size = self.next_byte_code(1)
        address = self.next_byte_code(8)
        register = self.next_byte_code(1)
        self.memory.store_data(address, self.registers[register], size)
    

    def handle_move_addr_literal_addr_in_reg(self) -> None:
        size = self.next_byte_code(1)
        address1 = self.next_byte_code(8)
        register = self.next_byte_code(1)
        address2 = self.registers[register]
        self.memory.store_data(address1, self.memory.get_data(address2, size), size)
    

    def handle_move_addr_literal_const(self) -> None:
        size = self.next_byte_code(1)
        address = self.next_byte_code(8)
        value = self.next_byte_code(size)
        self.memory.store_data(address, value, size)
    

    def handle_move_addr_literal_addr_literal(self) -> None:
        size = self.next_byte_code(1)
        address1 = self.next_byte_code(8)
        address2 = self.next_byte_code(8)
        self.memory.store_data(address1, self.memory.get_data(address2, size), size)

    
    def handle_push_reg(self) -> None:
        register = self.next_byte_code(1)
        self.push_stack(self.registers[register], 8)
    

    def handle_push_addr_in_reg(self) -> None:
        size = self.next_byte_code(1)
        register = self.next_byte_code(1)
        address = self.registers[register]
        self.push_stack(self.memory.get_data(address, size), size)
    

    def handle_push_const(self) -> None:
        size = self.next_byte_code(1)
        value = self.next_byte_code(size)
        self.push_stack(value, size)
    

    def handle_push_addr_literal(self) -> None:
        size = self.next_byte_code(1)
        address = self.next_byte_code(8)
        self.push_stack(self.memory.get_data(address, size), size)


    def handle_pop_reg(self) -> None:
        register = self.next_byte_code(1)
        self.registers[register] = self.pop_stack(8)


    def handle_pop_addr_in_reg(self) -> None:
        size = self.next_byte_code(1)
        register = self.next_byte_code(1)
        address = self.registers[register]
        self.memory.store_data(address, self.pop_stack(size), size)


    def handle_pop_addr_literal(self) -> None:
        size = self.next_byte_code(1)
        address = self.next_byte_code(8)
        self.memory.store_data(address, self.pop_stack(size), size)

    
    def handle_jump(self) -> None:
        self.registers[Registers.PROGRAM_COUNTER] = self.next_byte_code(8)
    

    def handle_jump_if_true_reg(self) -> None:
        address = self.next_byte_code(8)
        register = self.next_byte_code(1)
        if self.registers[register] != 0:
            self.registers[Registers.PROGRAM_COUNTER] = address
    

    def handle_jump_if_false_reg(self) -> None:
        address = self.next_byte_code(8)
        register = self.next_byte_code(1)
        if self.registers[register] == 0:
            self.registers[Registers.PROGRAM_COUNTER] = address


    def handle_compare_reg_reg(self) -> None:
        register1 = self.next_byte_code(1)
        register2 = self.next_byte_code(1)
        self.set_arithmetical_flags(register1 - register2)


    def handle_compare_reg_const(self) -> None:
        size = self.next_byte_code(1)
        register = self.next_byte_code(1)
        value = self.next_byte_code(size)
        self.set_arithmetical_flags(self.registers[register] - value)
    

    def handle_compare_const_reg(self) -> None:
        size = self.next_byte_code(1)
        value = self.next_byte_code(size)
        register = self.next_byte_code(1)
        self.set_arithmetical_flags(value - self.registers[register])
    

    def handle_compare_const_const(self) -> None:
        size = self.next_byte_code(1)
        value1 = self.next_byte_code(size)
        value2 = self.next_byte_code(size)
        self.set_arithmetical_flags(value1 - value2)

    
    def handle_print(self) -> None:
        print(int(self.registers[Registers.PRINT]), end='')

    
    def handle_print_string(self) -> None:
        address = self.registers[Registers.PRINT]
        buffer = ''
        byte = 1
        while byte != 0:
            byte = self.memory.load_data(address, 1)
            address += 1
            buffer += byte.decode('utf-8')
        
        print(buffer, end='')


    def handle_input_int(self) -> None:
        try:
            self.registers[Registers.INPUT] = int(input())
        except ValueError:
            self.registers[Registers.ERROR] = Errors.INVALID_INPUT
        except EOFError:
            self.registers[Registers.ERROR] = Errors.END_OF_FILE
        except:
            self.registers[Registers.ERROR] = Errors.GENERIC_ERROR
            

    def handle_input_string(self) -> None:
        try:
            data = input().encode('utf-8')
        except EOFError:
            self.registers[Registers.ERROR] = Errors.END_OF_FILE
            return
        except UnicodeEncodeError:
            self.registers[Registers.ERROR] = Errors.INVALID_INPUT
            return
        except:
            self.registers[Registers.ERROR] = Errors.GENERIC_ERROR
            return

        self.push_stack_bytes(data)
        self.registers[Registers.INPUT] = len(data)


    def handle_exit(self) -> None:
        self.running = False

    
    # End of instruction handlers

    instruction_handlers_table: List[Callable[[Processor], None]] = \
    [
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


        None, # Labels don't get handled


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

        handle_exit,


    ]

