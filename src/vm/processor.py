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
        # Load the byte code into memory.
        self.memory.store_bytes(0, byte_code)
        # Set the stack pointer to the first available memory address.
        self.registers[Registers.STACK_POINTER] = len(byte_code)

        self.running = True
        while self.running:
            # Fetch the instruction
            opcode = self.get_from_byte_code(1)

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


    def get_from_byte_code(self, size: int) -> int:
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
        register = self.get_from_byte_code(1)
        self.registers[register] += 1
        
        self.set_arithmetical_flags(self.registers[register])

    
    def handle_inc1_addr_in_reg(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.registers[register]
        value = self.memory.get_data(address, 1) + 1
        self.memory.store_data(self.register, value, 1)

        self.set_arithmetical_flags(value)


    def handle_inc1_addr_literal(self) -> None:
        address = self.get_from_byte_code(8)
        value = self.memory.get_data(address, 1) + 1
        self.memory.store_data(address, value, 1)

        self.set_arithmetical_flags(value)


    def handle_inc2_addr_in_reg(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.registers[register]
        value = self.memory.get_data(address, 2) + 1
        self.memory.store_data(address, value, 2)

        self.set_arithmetical_flags(value)

    
    def handle_inc2_addr_literal(self) -> None:
        address = self.get_from_byte_code(8)
        value = self.memory.get_data(address, 2) + 1
        self.memory.store_data(address, value, 2)

        self.set_arithmetical_flags(value)    

    
    def handle_inc4_addr_in_reg(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.registers[register]
        value = self.memory.get_data(address, 4) + 1
        self.memory.store_data(address, value, 4)

        self.set_arithmetical_flags(value)


    def handle_inc4_addr_literal(self) -> None:
        address = self.get_from_byte_code(8)
        value = self.memory.get_data(address, 4) + 1
        self.memory.store_data(address, value, 4)

        self.set_arithmetical_flags(value)

    
    def handle_inc8_addr_in_reg(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.registers[register]
        value = self.memory.get_data(address, 8) + 1
        self.memory.store_data(address, value, 8)

        self.set_arithmetical_flags(value)

    
    def handle_inc8_addr_literal(self) -> None:
        address = self.get_from_byte_code(8)
        value = self.memory.get_data(address, 8) + 1
        self.memory.store_data(address, value, 8)

        self.set_arithmetical_flags(value)

    
    def handle_dec_reg(self) -> None:
        register = self.get_from_byte_code(1)
        self.registers[register] -= 1

        self.set_arithmetical_flags(self.registers[register])
    

    def handle_dec1_addr_in_reg(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.registers[register]
        value = self.memory.get_data(address, 1) - 1
        self.memory.store_data(address, value, 1)

        self.set_arithmetical_flags(value)

    
    def handle_dec1_addr_literal(self) -> None:
        address = self.get_from_byte_code(8)
        value = self.memory.get_data(address, 1) - 1
        self.memory.store_data(address, value, 1)

        self.set_arithmetical_flags(value)

    
    def handle_dec2_addr_in_reg(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.registers[register]
        value = self.memory.get_data(address, 2) - 1
        self.memory.store_data(address, value, 2)

        self.set_arithmetical_flags(value)

    
    def handle_dec2_addr_literal(self) -> None:
        address = self.get_from_byte_code(8)
        value = self.memory.get_data(address, 2) - 1
        self.memory.store_data(address, value, 2)

        self.set_arithmetical_flags(value)

    
    def handle_dec4_addr_in_reg(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.registers[register]
        value = self.memory.get_data(address, 4) - 1
        self.memory.store_data(address, value, 4)

        self.set_arithmetical_flags(value)


    def handle_dec4_addr_literal(self) -> None:
        address = self.get_from_byte_code(8)
        value = self.memory.get_data(address, 4) - 1
        self.memory.store_data(address, value, 4)

        self.set_arithmetical_flags(value)


    def handle_dec8_addr_in_reg(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.registers[register]
        value = self.memory.get_data(address, 8) - 1
        self.memory.store_data(address, value, 8)

        self.set_arithmetical_flags(value)


    def handle_dec8_addr_literal(self) -> None:
        address = self.get_from_byte_code(8)
        value = self.memory.get_data(address, 8) - 1
        self.memory.store_data(address, value, 8)

        self.set_arithmetical_flags(value)

    
    def handle_no_operation(self) -> None:
        pass


    def handle_load_reg_reg(self) -> None:
        register1 = self.get_from_byte_code(1)
        register2 = self.get_from_byte_code(1)
        self.registers[register1] = self.registers[register2]
    

    def handle_load1_reg_addr_in_reg(self) -> None:
        register1 = self.get_from_byte_code(1)
        register2 = self.get_from_byte_code(1)
        address = self.registers[register2]
        self.registers[register1] = self.memory.get_data(address, 1)


    def handle_load1_reg_const(self) -> None:
        register = self.get_from_byte_code(1)
        value = self.get_from_byte_code(1)
        self.registers[register] = value


    def handle_load1_reg_addr_literal(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.get_from_byte_code(8)
        self.registers[register] = self.memory.get_data(address, 1)
    

    def handle_load2_reg_addr_in_reg(self) -> None:
        register1 = self.get_from_byte_code(1)
        register2 = self.get_from_byte_code(1)
        address = self.registers[register2]
        self.registers[register1] = self.memory.get_data(address, 2)


    def handle_load2_reg_const(self) -> None:
        register = self.get_from_byte_code(1)
        value = self.get_from_byte_code(2)
        self.registers[register] = value


    def handle_load2_reg_addr_literal(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.get_from_byte_code(8)
        self.registers[register] = self.memory.get_data(address, 2)


    def handle_load4_reg_addr_in_reg(self) -> None:
        register1 = self.get_from_byte_code(1)
        register2 = self.get_from_byte_code(1)
        address = self.registers[register2]
        self.registers[register1] = self.memory.get_data(address, 4)


    def handle_load4_reg_const(self) -> None:
        register = self.get_from_byte_code(1)
        value = self.get_from_byte_code(4)
        self.registers[register] = value


    def handle_load4_reg_addr_literal(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.get_from_byte_code(8)
        self.registers[register] = self.memory.get_data(address, 4)


    def handle_load8_reg_addr_in_reg(self) -> None:
        register1 = self.get_from_byte_code(1)
        register2 = self.get_from_byte_code(1)
        address = self.registers[register2]
        self.registers[register1] = self.memory.get_data(address, 8)


    def handle_load8_reg_const(self) -> None:
        register = self.get_from_byte_code(1)
        value = self.get_from_byte_code(8)
        self.registers[register] = value


    def handle_load8_reg_addr_literal(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.get_from_byte_code(8)
        self.registers[register] = self.memory.get_data(address, 8)


    def handle_move_reg_reg(self) -> None:
        register1 = self.get_from_byte_code(1)
        register2 = self.get_from_byte_code(1)
        self.registers[register1] = self.registers[register2]
    

    def handle_move1_reg_addr_in_reg(self) -> None:
        register1 = self.get_from_byte_code(1)
        register2 = self.get_from_byte_code(1)
        address = self.registers[register2]
        self.registers[register1] = self.memory.get_data(address, 1)
    

    def handle_move1_reg_const(self) -> None:
        register = self.get_from_byte_code(1)
        self.registers[register] = self.get_from_byte_code(1)

    
    def handle_move1_reg_addr_literal(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.get_from_byte_code(8)
        self.registers[register] = self.memory.get_data(address, 1)
    

    def handle_move1_addr_in_reg_reg(self) -> None:
        register1 = self.get_from_byte_code(1)
        address = self.registers[register1]
        register2 = self.get_from_byte_code(1)
        self.memory.store_data(address, self.registers[register2], 1)
    

    def handle_move1_addr_in_reg_addr_in_reg(self) -> None:
        register1 = self.get_from_byte_code(1)
        address1 = self.registers[register1]
        register2 = self.get_from_byte_code(1)
        address2 = self.registers[register2]
        self.memory.store_data(address1, self.memory.get_data(address2, 1), 1)
        

    def handle_move1_addr_in_reg_const(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.registers[register]
        value = self.get_from_byte_code(1)
        self.memory.store_data(address, value, 1)
    

    def handle_move1_addr_in_reg_addr_literal(self) -> None:
        register = self.get_from_byte_code(1)
        address1 = self.registers[register]
        address2 = self.get_from_byte_code(8)
        self.memory.store_data(address1, self.memory.get_data(address2, 1), 1)

    
    def handle_move1_addr_literal_reg(self) -> None:
        address = self.get_from_byte_code(8)
        register = self.get_from_byte_code(1)
        self.memory.store_data(address, self.registers[register])
    

    def handle_move1_addr_literal_addr_in_reg(self) -> None:
        address1 = self.get_from_byte_code(8)
        register = self.get_from_byte_code(1)
        address2 = self.registers[register]
        self.memory.store_data(address1, self.memory.get_data(address2, 1), 1)
    

    def handle_move1_addr_literal_const(self) -> None:
        address = self.get_from_byte_code(8)
        value = self.get_from_byte_code(1)
        self.memory.store_data(address, value, 1)
    

    def handle_move1_addr_literal_addr_literal(self) -> None:
        address1 = self.get_from_byte_code(8)
        address2 = self.get_from_byte_code(8)
        self.memory.store_data(address1, self.memory.get_data(address2, 1), 1)


    def handle_move2_reg_addr_in_reg(self) -> None:
        register1 = self.get_from_byte_code(1)
        register2 = self.get_from_byte_code(1)
        address = self.registers[register2]
        self.registers[register1] = self.memory.get_data(address, 2)

    
    def handle_move2_reg_const(self) -> None:
        register = self.get_from_byte_code(1)
        self.registers[register] = self.get_from_byte_code(2)
    

    def handle_move2_reg_addr_literal(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.get_from_byte_code(8)
        self.registers[register] = self.memory.get_data(address, 2)


    def handle_move2_addr_in_reg_reg(self) -> None:
        register1 = self.get_from_byte_code(1)
        address = self.registers[register1]
        register2 = self.get_from_byte_code(1)
        self.memory.store_data(address, self.registers[register2], 2)


    def handle_move2_addr_in_reg_addr_in_reg(self) -> None:
        register1 = self.get_from_byte_code(1)
        address1 = self.registers[register1]
        register2 = self.get_from_byte_code(1)
        address2 = self.registers[register2]
        self.memory.store_data(address1, self.memory.get_data(address2, 2), 2)


    def handle_move2_addr_in_reg_const(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.registers[register]
        value = self.get_from_byte_code(2)
        self.memory.store_data(address, value, 2)


    def handle_move2_addr_in_reg_addr_literal(self) -> None:
        register = self.get_from_byte_code(1)
        address1 = self.registers[register]
        address2 = self.get_from_byte_code(8)
        self.memory.store_data(address1, self.memory.get_data(address2, 2), 2)


    def handle_move2_addr_literal_reg(self) -> None:
        address = self.get_from_byte_code(8)
        register = self.get_from_byte_code(1)
        self.memory.store_data(address, self.registers[register], 2)
    

    def handle_move2_addr_literal_addr_in_reg(self) -> None:
        address1 = self.get_from_byte_code(8)
        register = self.get_from_byte_code(1)
        address2 = self.registers[register]
        self.memory.store_data(address1, self.memory.get_data(address2, 2), 2)


    def handle_move2_addr_literal_const(self) -> None:
        address = self.get_from_byte_code(8)
        value = self.get_from_byte_code(2)
        self.memory.store_data(address, value, 2)


    def handle_move2_addr_literal_addr_literal(self) -> None:
        address1 = self.get_from_byte_code(8)
        address2 = self.get_from_byte_code(8)
        self.memory.store_data(address1, self.memory.get_data(address2, 2), 2)
    

    def handle_move4_reg_addr_in_reg(self) -> None:
        register1 = self.get_from_byte_code(1)
        register2 = self.get_from_byte_code(1)
        address = self.registers[register2]
        self.registers[register1] = self.memory.get_data(address, 4)

    
    def handle_move4_reg_const(self) -> None:
        register = self.get_from_byte_code(1)
        self.registers[register] = self.get_from_byte_code(4)
    

    def handle_move4_reg_addr_literal(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.get_from_byte_code(8)
        self.registers[register] = self.memory.get_data(address, 4)


    def handle_move4_addr_in_reg_reg(self) -> None:
        register1 = self.get_from_byte_code(1)
        address = self.registers[register1]
        register2 = self.get_from_byte_code(1)
        self.memory.store_data(address, self.registers[register2], 4)


    def handle_move4_addr_in_reg_addr_in_reg(self) -> None:
        register1 = self.get_from_byte_code(1)
        address1 = self.registers[register1]
        register2 = self.get_from_byte_code(1)
        address2 = self.registers[register2]
        self.memory.store_data(address1, self.memory.get_data(address2, 4), 4)


    def handle_move4_addr_in_reg_const(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.registers[register]
        value = self.get_from_byte_code(4)
        self.memory.store_data(address, value, 4)


    def handle_move4_addr_in_reg_addr_literal(self) -> None:
        register = self.get_from_byte_code(1)
        address1 = self.registers[register]
        address2 = self.get_from_byte_code(8)
        self.memory.store_data(address1, self.memory.get_data(address2, 4), 4)


    def handle_move4_addr_literal_reg(self) -> None:
        address = self.get_from_byte_code(8)
        register = self.get_from_byte_code(1)
        self.memory.store_data(address, self.registers[register], 4)
    

    def handle_move4_addr_literal_addr_in_reg(self) -> None:
        address1 = self.get_from_byte_code(8)
        register = self.get_from_byte_code(1)
        address2 = self.registers[register]
        self.memory.store_data(address1, self.memory.get_data(address2, 4), 4)


    def handle_move4_addr_literal_const(self) -> None:
        address = self.get_from_byte_code(8)
        value = self.get_from_byte_code(4)
        self.memory.store_data(address, value, 4)


    def handle_move4_addr_literal_addr_literal(self) -> None:
        address1 = self.get_from_byte_code(8)
        address2 = self.get_from_byte_code(8)
        self.memory.store_data(address1, self.memory.get_data(address2, 4), 4)
    

    def handle_move8_reg_addr_in_reg(self) -> None:
        register1 = self.get_from_byte_code(1)
        register2 = self.get_from_byte_code(1)
        address = self.registers[register2]
        self.registers[register1] = self.memory.get_data(address, 8)

    
    def handle_move8_reg_const(self) -> None:
        register = self.get_from_byte_code(1)
        self.registers[register] = self.get_from_byte_code(8)
    

    def handle_move8_reg_addr_literal(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.get_from_byte_code(8)
        self.registers[register] = self.memory.get_data(address, 8)


    def handle_move8_addr_in_reg_reg(self) -> None:
        register1 = self.get_from_byte_code(1)
        address = self.registers[register1]
        register2 = self.get_from_byte_code(1)
        self.memory.store_data(address, self.registers[register2], 8)


    def handle_move8_addr_in_reg_addr_in_reg(self) -> None:
        register1 = self.get_from_byte_code(1)
        address1 = self.registers[register1]
        register2 = self.get_from_byte_code(1)
        address2 = self.registers[register2]
        self.memory.store_data(address1, self.memory.get_data(address2, 8), 8)


    def handle_move8_addr_in_reg_const(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.registers[register]
        value = self.get_from_byte_code(8)
        self.memory.store_data(address, value, 8)


    def handle_move8_addr_in_reg_addr_literal(self) -> None:
        register = self.get_from_byte_code(1)
        address1 = self.registers[register]
        address2 = self.get_from_byte_code(8)
        self.memory.store_data(address1, self.memory.get_data(address2, 8), 8)


    def handle_move8_addr_literal_reg(self) -> None:
        address = self.get_from_byte_code(8)
        register = self.get_from_byte_code(1)
        self.memory.store_data(address, self.registers[register], 8)
    

    def handle_move8_addr_literal_addr_in_reg(self) -> None:
        address1 = self.get_from_byte_code(8)
        register = self.get_from_byte_code(1)
        address2 = self.registers[register]
        self.memory.store_data(address1, self.memory.get_data(address2, 8), 8)


    def handle_move8_addr_literal_const(self) -> None:
        address = self.get_from_byte_code(8)
        value = self.get_from_byte_code(8)
        self.memory.store_data(address, value, 8)


    def handle_move8_addr_literal_addr_literal(self) -> None:
        address1 = self.get_from_byte_code(8)
        address2 = self.get_from_byte_code(8)
        self.memory.store_data(address1, self.memory.get_data(address2, 8), 8)
    

    def handle_store1_addr_in_reg_reg(self) -> None:
        register1 = self.get_from_byte_code(1)
        address = self.registers[register1]
        register2 = self.get_from_byte_code(1)
        self.memory.store_data(address, self.registers[register2], 1)


    def handle_store1_addr_literal_reg(self) -> None:
        address = self.get_from_byte_code(8)
        register = self.get_from_byte_code(1)
        self.memory.store_data(address, self.registers[register], 1)


    def handle_store2_addr_in_reg_reg(self) -> None:
        register1 = self.get_from_byte_code(1)
        address = self.registers[register1]
        register2 = self.get_from_byte_code(1)
        self.memory.store_data(address, self.registers[register2], 2)

    
    def handle_store2_addr_literal_reg(self) -> None:
        address = self.get_from_byte_code(8)
        register = self.get_from_byte_code(1)
        self.memory.store_data(address, self.registers[register], 2)

    
    def handle_store4_addr_in_reg_reg(self) -> None:
        register1 = self.get_from_byte_code(1)
        address = self.registers[register1]
        register2 = self.get_from_byte_code(1)
        self.memory.store_data(address, self.registers[register2], 4)
    

    def handle_store4_addr_literal_reg(self) -> None:
        address = self.get_from_byte_code(8)
        register = self.get_from_byte_code(1)
        self.memory.store_data(address, self.registers[register], 4)


    def handle_store8_addr_in_reg_reg(self) -> None:
        register1 = self.get_from_byte_code(1)
        address = self.registers[register1]
        register2 = self.get_from_byte_code(1)
        self.memory.store_data(address, self.registers[register2], 8)

    
    def handle_store8_addr_literal_reg(self) -> None:
        address = self.get_from_byte_code(8)
        register = self.get_from_byte_code(1)
        self.memory.store_data(address, self.registers[register], 8)

    
    def handle_push_reg(self) -> None:
        register = self.get_from_byte_code(1)
        self.push_stack(self.registers[register], 8)
    

    def handle_push1_addr_in_reg(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.registers[register]
        self.push_stack(self.memory.get_data(address, 1))
    

    def handle_push1_const(self) -> None:
        value = self.get_from_byte_code(1)
        self.push_stack(value, 1)
    

    def handle_push1_addr_literal(self) -> None:
        address = self.get_from_byte_code(8)
        self.push_stack(self.memory.get_data(address, 1))


    def handle_push2_addr_in_reg(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.registers[register]
        self.push_stack(self.memory.get_data(address, 2))


    def handle_push2_const(self) -> None:
        value = self.get_from_byte_code(2)
        self.push_stack(value, 2)


    def handle_push2_addr_literal(self) -> None:
        address = self.get_from_byte_code(8)
        self.push_stack(self.memory.get_data(address, 2))


    def handle_push4_addr_in_reg(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.registers[register]
        self.push_stack(self.memory.get_data(address, 4))


    def handle_push4_const(self) -> None:
        value = self.get_from_byte_code(4)
        self.push_stack(value, 4)


    def handle_push4_addr_literal(self) -> None:
        address = self.get_from_byte_code(8)
        self.push_stack(self.memory.get_data(address, 4))


    def handle_push8_addr_in_reg(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.registers[register]
        self.push_stack(self.memory.get_data(address, 8))


    def handle_push8_const(self) -> None:
        value = self.get_from_byte_code(8)
        self.push_stack(value, 8)


    def handle_push8_addr_literal(self) -> None:
        address = self.get_from_byte_code(8)
        self.push_stack(self.memory.get_data(address, 8))


    def handle_pop_reg(self) -> None:
        register = self.get_from_byte_code(1)
        self.registers[register] = self.pop_stack(8)


    def handle_pop1_addr_in_reg(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.registers[register]
        self.memory.store_data(address, self.pop_stack(1), 1)


    def handle_pop1_addr_literal(self) -> None:
        address = self.get_from_byte_code(8)
        self.memory.store_data(address, self.pop_stack(1), 1)


    def handle_pop2_addr_in_reg(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.registers[register]
        self.memory.store_data(address, self.pop_stack(2), 2)


    def handle_pop2_addr_literal(self) -> None:
        address = self.get_from_byte_code(8)
        self.memory.store_data(address, self.pop_stack(2), 2)


    def handle_pop4_addr_in_reg(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.registers[register]
        self.memory.store_data(address, self.pop_stack(4), 4)


    def handle_pop4_addr_literal(self) -> None:
        address = self.get_from_byte_code(8)
        self.memory.store_data(address, self.pop_stack(4), 4)


    def handle_pop8_addr_in_reg(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.registers[register]
        self.memory.store_data(address, self.pop_stack(8), 8)


    def handle_pop8_addr_literal(self) -> None:
        address = self.get_from_byte_code(8)
        self.memory.store_data(address, self.pop_stack(8), 8)

    
    def handle_jump(self) -> None:
        self.registers[Registers.PROGRAM_COUNTER] = self.get_from_byte_code(8)
    

    def handle_jump_if_true_reg(self) -> None:
        address = self.get_from_byte_code(8)
        register = self.get_from_byte_code(1)
        if self.registers[register] != 0:
            self.registers[Registers.PROGRAM_COUNTER] = address
    

    def handle_jump_if_false_reg(self) -> None:
        address = self.get_from_byte_code(8)
        register = self.get_from_byte_code(1)
        if self.registers[register] == 0:
            self.registers[Registers.PROGRAM_COUNTER] = address


    def handle_compare_reg_reg(self) -> None:
        register1 = self.get_from_byte_code(1)
        register2 = self.get_from_byte_code(1)
        self.set_arithmetical_flags(register1 - register2)


    def handle_compare1_reg_const(self) -> None:
        register = self.get_from_byte_code(1)
        value = self.get_from_byte_code(1)
        self.set_arithmetical_flags(self.registers[register] - value)
    

    def handle_compare1_const_reg(self) -> None:
        value = self.get_from_byte_code(1)
        register = self.get_from_byte_code(1)
        self.set_arithmetical_flags(value - self.registers[register])
    

    def handle_compare1_const_const(self) -> None:
        value1 = self.get_from_byte_code(1)
        value2 = self.get_from_byte_code(1)
        self.set_arithmetical_flags(value1 - value2)


    def handle_compare2_reg_const(self) -> None:
        register = self.get_from_byte_code(1)
        value = self.get_from_byte_code(2)
        self.set_arithmetical_flags(self.registers[register] - value)
    

    def handle_compare2_const_reg(self) -> None:
        value = self.get_from_byte_code(2)
        register = self.get_from_byte_code(1)
        self.set_arithmetical_flags(value - self.registers[register])
    

    def handle_compare2_const_const(self) -> None:
        value1 = self.get_from_byte_code(2)
        value2 = self.get_from_byte_code(2)
        self.set_arithmetical_flags(value1 - value2)
    

    def handle_compare4_reg_const(self) -> None:
        register = self.get_from_byte_code(1)
        value = self.get_from_byte_code(4)
        self.set_arithmetical_flags(self.registers[register] - value)
    

    def handle_compare4_const_reg(self) -> None:
        value = self.get_from_byte_code(4)
        register = self.get_from_byte_code(1)
        self.set_arithmetical_flags(value - self.registers[register])
    

    def handle_compare4_const_const(self) -> None:
        value1 = self.get_from_byte_code(4)
        value2 = self.get_from_byte_code(4)
        self.set_arithmetical_flags(value1 - value2)

    
    def handle_compare8_reg_const(self) -> None:
        register = self.get_from_byte_code(1)
        value = self.get_from_byte_code(8)
        self.set_arithmetical_flags(self.registers[register] - value)
    

    def handle_compare8_const_reg(self) -> None:
        value = self.get_from_byte_code(8)
        register = self.get_from_byte_code(1)
        self.set_arithmetical_flags(value - self.registers[register])
    

    def handle_compare8_const_const(self) -> None:
        value1 = self.get_from_byte_code(8)
        value2 = self.get_from_byte_code(8)
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

        handle_inc1_addr_in_reg,
        handle_inc1_addr_literal,

        handle_inc2_addr_in_reg,
        handle_inc2_addr_literal,

        handle_inc4_addr_in_reg,
        handle_inc4_addr_literal,

        handle_inc8_addr_in_reg,
        handle_inc8_addr_literal,


        handle_dec_reg,

        handle_dec1_addr_in_reg,
        handle_dec1_addr_literal,

        handle_dec2_addr_in_reg,
        handle_dec2_addr_literal,

        handle_dec4_addr_in_reg,
        handle_dec4_addr_literal,

        handle_dec8_addr_in_reg,
        handle_dec8_addr_literal,


        handle_no_operation,


        handle_load_reg_reg,

        handle_load1_reg_addr_in_reg,
        handle_load1_reg_const,
        handle_load1_reg_addr_literal,

        handle_load2_reg_addr_in_reg,
        handle_load2_reg_const,
        handle_load2_reg_addr_literal,

        handle_load4_reg_addr_in_reg,
        handle_load4_reg_const,
        handle_load4_reg_addr_literal,

        handle_load8_reg_addr_in_reg,
        handle_load8_reg_const,
        handle_load8_reg_addr_literal,


        handle_move_reg_reg,

        handle_move1_reg_addr_in_reg,
        handle_move1_reg_const,
        handle_move1_reg_addr_literal,
        handle_move1_addr_in_reg_reg,
        handle_move1_addr_in_reg_addr_in_reg,
        handle_move1_addr_in_reg_const,
        handle_move1_addr_in_reg_addr_literal,
        handle_move1_addr_literal_reg,
        handle_move1_addr_literal_addr_in_reg,
        handle_move1_addr_literal_const,
        handle_move1_addr_literal_addr_literal,

        handle_move2_reg_addr_in_reg,
        handle_move2_reg_const,
        handle_move2_reg_addr_literal,
        handle_move2_addr_in_reg_reg,
        handle_move2_addr_in_reg_addr_in_reg,
        handle_move2_addr_in_reg_const,
        handle_move2_addr_in_reg_addr_literal,
        handle_move2_addr_literal_reg,
        handle_move2_addr_literal_addr_in_reg,
        handle_move2_addr_literal_const,
        handle_move2_addr_literal_addr_literal,

        handle_move4_reg_addr_in_reg,
        handle_move4_reg_const,
        handle_move4_reg_addr_literal,
        handle_move4_addr_in_reg_reg,
        handle_move4_addr_in_reg_addr_in_reg,
        handle_move4_addr_in_reg_const,
        handle_move4_addr_in_reg_addr_literal,
        handle_move4_addr_literal_reg,
        handle_move4_addr_literal_addr_in_reg,
        handle_move4_addr_literal_const,
        handle_move4_addr_literal_addr_literal,

        handle_move8_reg_addr_in_reg,
        handle_move8_reg_const,
        handle_move8_reg_addr_literal,
        handle_move8_addr_in_reg_reg,
        handle_move8_addr_in_reg_addr_in_reg,
        handle_move8_addr_in_reg_const,
        handle_move8_addr_in_reg_addr_literal,
        handle_move8_addr_literal_reg,
        handle_move8_addr_literal_addr_in_reg,
        handle_move8_addr_literal_const,
        handle_move8_addr_literal_addr_literal,


        handle_store1_addr_in_reg_reg,
        handle_store1_addr_literal_reg,

        handle_store2_addr_in_reg_reg,
        handle_store2_addr_literal_reg,

        handle_store4_addr_in_reg_reg,
        handle_store4_addr_literal_reg,

        handle_store8_addr_in_reg_reg,
        handle_store8_addr_literal_reg,


        handle_push_reg,

        handle_push1_addr_in_reg,
        handle_push1_const,
        handle_push1_addr_literal,

        handle_push2_addr_in_reg,
        handle_push2_const,
        handle_push2_addr_literal,

        handle_push4_addr_in_reg,
        handle_push4_const,
        handle_push4_addr_literal,

        handle_push8_addr_in_reg,
        handle_push8_const,
        handle_push8_addr_literal,


        handle_pop_reg,

        handle_pop1_addr_in_reg,
        handle_pop1_addr_literal,

        handle_pop2_addr_in_reg,
        handle_pop2_addr_literal,

        handle_pop4_addr_in_reg,
        handle_pop4_addr_literal,

        handle_pop8_addr_in_reg,
        handle_pop8_addr_literal,


        None, # Labels don't get handled


        handle_jump,

        handle_jump_if_true_reg,

        handle_jump_if_false_reg,


        handle_compare_reg_reg,
        
        handle_compare1_reg_const,
        handle_compare1_const_reg,
        handle_compare1_const_const,

        handle_compare2_reg_const,
        handle_compare2_const_reg,
        handle_compare2_const_const,

        handle_compare4_reg_const,
        handle_compare4_const_reg,
        handle_compare4_const_const,

        handle_compare8_reg_const,
        handle_compare8_const_reg,
        handle_compare8_const_const,


        handle_print,

        handle_print_string,


        handle_input_int,

        handle_input_string,


        handle_exit,


    ]

