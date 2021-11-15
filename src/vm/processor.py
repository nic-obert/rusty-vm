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
        pass


    # Instruction handlers

    def handle_add(self) -> None:
        self.A += self.B

        self.ZERO_FLAG = self.A == 0
        self.SIGN_FLAG = self.A < 0

        self.STACK_POINTER += 1

    def handle_sub(self) -> None:
        self.A -= self.B

        self.ZERO_FLAG = self.A == 0
        self.SIGN_FLAG = self.A < 0

        self.STACK_POINTER += 1

    def handle_mul(self) -> None:
        self.A *= self.B

        self.ZERO_FLAG = self.A == 0
        self.SIGN_FLAG = self.A < 0

        self.STACK_POINTER += 1
    
    def handle_div(self) -> None:
        self.A //= self.B

        self.ZERO_FLAG = self.A == 0
        self.SIGN_FLAG = self.A < 0

        self.STACK_POINTER += 1

    def handle_mod(self) -> None:
        self.A %= self.B

        self.ZERO_FLAG = self.A == 0
        self.SIGN_FLAG = self.A < 0

        self.STACK_POINTER += 1

    def handle_inc_reg(self) -> None:
        self.STACK_POINTER += 1
        register = self.memory.get_data(self.STACK_POINTER, 1)
        self.STACK_POINTER += 1
        self.registers[register] += 1
    
    def handle_inc1_addr_in_reg(self) -> None:
        self.STACK_POINTER += 1
        register = self.memory.get_data(self.STACK_POINTER, 1)
        address = self.registers[register]
        self.STACK_POINTER += 1
        value = self.memory.get_data(address, 1) + 1
        self.memory.store_data(self.register, value, 1)

