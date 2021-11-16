from typing import List
from memory import Memory

from shared.registers import Registers


class Processor:

    def __init__(self, memory_size: int) -> None:
        self.memory = Memory(memory_size)

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
            self.SIGN_FLAG
        ]
    

    def execute(self, byte_code: bytes) -> None:
        """
        Execute the byte code.
        """
        # Load the byte code into memory.
        self.memory.store_data(0, byte_code, len(byte_code))


    def set_flags(self, value: int) -> None:
        self.ZERO_FLAG = value == 0
        self.SIGN_FLAG = value < 0
    

    def get_from_byte_code(self, size: int) -> int:
        data = self.memory.get_data(self.PROGRAM_COUNTER, size)
        self.PROGRAM_COUNTER += size
        return data


    # Instruction handlers


    def handle_add(self) -> None:
        self.A += self.B

        self.set_flags(self.A)


    def handle_sub(self) -> None:
        self.A -= self.B

        self.set_flags(self.A)


    def handle_mul(self) -> None:
        self.A *= self.B

        self.set_flags(self.A)


    def handle_div(self) -> None:
        self.A //= self.B

        self.set_flags(self.A)


    def handle_mod(self) -> None:
        self.A %= self.B

        self.set_flags(self.A)


    def handle_inc_reg(self) -> None:
        register = self.get_from_byte_code(1)
        self.registers[register] += 1
        
        self.set_flags(self.registers[register])

    
    def handle_inc1_addr_in_reg(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.registers[register]
        value = self.memory.get_data(address, 1) + 1
        self.memory.store_data(self.register, value, 1)

        self.set_flags(self.A)


    def handle_inc1_addr_literal(self) -> None:
        address = self.get_from_byte_code(8)
        value = self.memory.get_data(address, 1) + 1
        self.memory.store_data(address, value, 1)

        self.set_flags(value)


    def handle_inc2_addr_in_reg(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.registers[register]
        value = self.memory.get_data(address, 2) + 1
        self.memory.store_data(address, value, 2)

        self.set_flags(value)

    
    def handle_inc2_addr_literal(self) -> None:
        address = self.get_from_byte_code(8)
        value = self.memory.get_data(address, 2) + 1
        self.memory.store_data(address, value, 2)

        self.set_flags(value)    

    
    def handle_inc4_addr_in_reg(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.registers[register]
        value = self.memory.get_data(address, 4) + 1
        self.memory.store_data(address, value, 4)

        self.set_flags(value)


    def handle_inc4_addr_literal(self) -> None:
        address = self.get_from_byte_code(8)
        value = self.memory.get_data(address, 4) + 1
        self.memory.store_data(address, value, 4)

        self.set_flags(value)

    
    def handle_inc8_addr_in_reg(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.registers[register]
        value = self.memory.get_data(address, 8) + 1
        self.memory.store_data(address, value, 8)

        self.set_flags(value)

    
    def handle_inc8_addr_literal(self) -> None:
        address = self.get_from_byte_code(8)
        value = self.memory.get_data(address, 8) + 1
        self.memory.store_data(address, value, 8)

        self.set_flags(value)

    
    def handle_dec_reg(self) -> None:
        register = self.get_from_byte_code(1)
        self.registers[register] -= 1

        self.set_flags(self.registers[register])
    

    def handle_dec1_addr_in_reg(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.registers[register]
        value = self.memory.get_data(address, 1) - 1
        self.memory.store_data(address, value, 1)

        self.set_flags(value)

    
    def handle_dec1_addr_literal(self) -> None:
        address = self.get_from_byte_code(8)
        value = self.memory.get_data(address, 1) - 1
        self.memory.store_data(address, value, 1)

        self.set_flags(value)

    
    def handle_dec2_addr_in_reg(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.registers[register]
        value = self.memory.get_data(address, 2) - 1
        self.memory.store_data(address, value, 2)

        self.set_flags(value)

    
    def handle_dec2_addr_literal(self) -> None:
        address = self.get_from_byte_code(8)
        value = self.memory.get_data(address, 2) - 1
        self.memory.store_data(address, value, 2)

        self.set_flags(value)

    
    def handle_dec4_addr_in_reg(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.registers[register]
        value = self.memory.get_data(address, 4) - 1
        self.memory.store_data(address, value, 4)

        self.set_flags(value)


    def handle_dec4_addr_literal(self) -> None:
        address = self.get_from_byte_code(8)
        value = self.memory.get_data(address, 4) - 1
        self.memory.store_data(address, value, 4)

        self.set_flags(value)


    def handle_dec8_addr_in_reg(self) -> None:
        register = self.get_from_byte_code(1)
        address = self.registers[register]
        value = self.memory.get_data(address, 8) - 1
        self.memory.store_data(address, value, 8)

        self.set_flags(value)


    def handle_dec8_addr_literal(self) -> None:
        address = self.get_from_byte_code(8)
        value = self.memory.get_data(address, 8) - 1
        self.memory.store_data(address, value, 8)

        self.set_flags(value)

    
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




