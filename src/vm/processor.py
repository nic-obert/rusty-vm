from __future__ import annotations
from typing import Callable, List

from memory import Memory


class Processor:

    def __init__(self, memory_size: int) -> None:
        self.memory = Memory(memory_size)
        self.running = False

        self.A = 0
        self.B = 0
        self.C = 0
        self.D = 0
        self.E = 0
        self.F = 0
        self.G = 0
        self.H = 0
        self.STACK_POINTER = 0
        self.PROGRAM_COUNTER = 0
        self.ZERO_FLAG = False
        self.SIGN_FLAG = False
        self.REMAINDER_FLAG = 0

        self.registers: List[int] = \
        [
            self.A,
            self.B,
            self.C,
            self.D,
            self.E,
            self.F,
            self.G,
            self.H,
            self.STACK_POINTER,
            self.PROGRAM_COUNTER,
            self.ZERO_FLAG,
            self.SIGN_FLAG,
            self.REMAINDER_FLAG,
        ]
    

    def execute(self, byte_code: bytes) -> None:
        """
        Execute the byte code.
        """
        # Load the byte code into memory.
        self.memory.store_bytes(0, byte_code)

        self.running = True
        while self.running:
            opcode = self.get_from_byte_code(1)
            self.instruction_handlers_table[opcode](self)

        exit(self.E)
    

    # Useful functions

    def set_arithmetical_flags(self, result: int, remainder: int = 0) -> None:
        self.ZERO_FLAG = result == 0
        self.SIGN_FLAG = result < 0
        self.REMAINDER_FLAG = remainder
    

    def get_from_byte_code(self, size: int) -> int:
        data = self.memory.get_data(self.PROGRAM_COUNTER, size)
        self.PROGRAM_COUNTER += size
        return data


    # Instruction handlers


    def handle_add(self) -> None:
        self.A += self.B

        self.set_arithmetical_flags(self.A)


    def handle_sub(self) -> None:
        self.A -= self.B

        self.set_arithmetical_flags(self.A)


    def handle_mul(self) -> None:
        self.A *= self.B

        self.set_arithmetical_flags(self.A)


    def handle_div(self) -> None:
        remainder = self.A % self.B
        self.A //= self.B

        self.set_arithmetical_flags(self.A, remainder)


    def handle_mod(self) -> None:
        self.A %= self.B

        self.set_arithmetical_flags(self.A)


    def handle_inc_reg(self) -> None:
        register = self.get_from_byte_code(1)
        self.registers[register] += 1
        
        self.set_arithmetical_flags(self.registers[register])

    
    def handle_inc1_addr_in_reg(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.registers[register]
        value = self.memory.get_data(address, 1) + 1
        self.memory.store_data(self.register, value, 1)

        self.set_arithmetical_flags(self.A)


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

    
    def handle_jump(self) -> None:
        self.PROGRAM_COUNTER = self.get_from_byte_code(8)
    

    def handle_jump_if_true_reg(self) -> None:
        address = self.get_from_byte_code(8)
        register = self.get_from_byte_code(1)
        if self.registers[register] != 0:
            self.PROGRAM_COUNTER = address
    

    def handle_jump_if_false_reg(self) -> None:
        address = self.get_from_byte_code(8)
        register = self.get_from_byte_code(1)
        if self.registers[register] == 0:
            self.PROGRAM_COUNTER = address


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
        print(int(self.H))

    
    def handle_print_string(self) -> None:
        address = self.H
        buffer = ''
        byte = 1
        while byte != 0:
            byte = self.memory.load_data(address, 1)
            address += 1
            buffer += byte.decode('utf-8')
        
        print(buffer, end='')


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


        None, # Label don't get handled


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


        handle_exit,


    ]


