import enum
from typing import List, Tuple


byte_code_names: Tuple[str] = \
(
    "ADD",
    "SUB",
    "MUL",
    "DIV",
    "MOD",

    "INC_REG",
    "INC1_ADDR",
    "INC2_ADDR",
    "INC4_ADDR",
    "INC8_ADDR",

    "DEC_REG",
    "DEC1_ADDR",
    "DEC2_ADDR",
    "DEC4_ADDR",
    "DEC8_ADDR",

    "NO_OPERATION",

    "LOAD1_REG_REG",
    "LOAD1_REG_ADDR",
    "LOAD1_REG_CONST",

    "LOAD2_REG_REG",
    "LOAD2_REG_ADDR",
    "LOAD2_REG_CONST",

    "LOAD4_REG_REG",
    "LOAD4_REG_ADDR",
    "LOAD4_REG_CONST",

    "LOAD8_REG_REG",
    "LOAD8_REG_ADDR",
    "LOAD8_REG_CONST",

    "MOVE1_REG_REG",
    "MOVE1_REG_ADDR",
    "MOVE1_REG_CONST",
    "MOVE1_ADDR_REG",
    "MOVE1_ADDR_ADDR",
    "MOVE1_ADDR_CONST",

    "MOVE2_REG_REG",
    "MOVE2_REG_ADDR",
    "MOVE2_REG_CONST",
    "MOVE2_ADDR_REG",
    "MOVE2_ADDR_ADDR",
    "MOVE2_ADDR_CONST",

    "MOVE4_REG_REG",
    "MOVE4_REG_ADDR",
    "MOVE4_REG_CONST",
    "MOVE4_ADDR_REG",
    "MOVE4_ADDR_ADDR",
    "MOVE4_ADDR_CONST",

    "MOVE8_REG_REG",
    "MOVE8_REG_ADDR",
    "MOVE8_REG_CONST",
    "MOVE8_ADDR_REG",
    "MOVE8_ADDR_ADDR",
    "MOVE8_ADDR_CONST",

    "STORE1_REG_REG",
    "STORE1_ADDR_REG",
    "STORE1_CONST_REG",

    "STORE2_REG_REG",
    "STORE2_ADDR_REG",
    "STORE2_CONST_REG",

    "STORE4_REG_REG",
    "STORE4_ADDR_REG",
    "STORE4_CONST_REG",

    "STORE8_REG_REG",
    "STORE8_ADDR_REG",
    "STORE8_CONST_REG",

    "LABEL",
    "JUMP",
    "JUMP_IF_TRUE_REG",
    "JUMP_IF_FALSE_REG",

    "COMPARE_REG_REG",
    "COMPARE_REG_CONST",
    "COMPARE_CONST_REG",
    "COMPARE_CONST_CONST",

)


@enum.unique
class ByteCodes(enum.IntEnum):

    def _generate_next_value_(name: str, start: int, count: int, last_values: List[int]) -> int:
        return count

    # Arithmetic

    ADD = enum.auto()
    SUB = enum.auto()
    MUL = enum.auto()
    DIV = enum.auto()
    MOD = enum.auto()

    INC_REG = enum.auto()
    INC1_ADDR = enum.auto()
    INC2_ADDR = enum.auto()
    INC4_ADDR = enum.auto()
    INC8_ADDR = enum.auto()

    DEC_REG = enum.auto()
    DEC1_ADDR = enum.auto()
    DEC2_ADDR = enum.auto()
    DEC4_ADDR = enum.auto()
    DEC8_ADDR = enum.auto()

    # No operation

    NO_OPERATION = enum.auto()

    # Memory

    LOAD1_REG_REG = enum.auto()
    LOAD1_REG_ADDR = enum.auto()
    LOAD1_REG_CONST = enum.auto()

    LOAD2_REG_REG = enum.auto()
    LOAD2_REG_ADDR = enum.auto()
    LOAD2_REG_CONST = enum.auto()

    LOAD4_REG_REG = enum.auto()
    LOAD4_REG_ADDR = enum.auto()
    LOAD4_REG_CONST = enum.auto()

    LOAD8_REG_REG = enum.auto()
    LOAD8_REG_ADDR = enum.auto()
    LOAD8_REG_CONST = enum.auto()


    MOVE1_REG_REG = enum.auto()
    MOVE1_REG_ADDR = enum.auto()
    MOVE1_REG_CONST = enum.auto()
    MOVE1_ADDR_REG = enum.auto()
    MOVE1_ADDR_ADDR = enum.auto()
    MOVE1_ADDR_CONST = enum.auto()

    MOVE2_REG_REG = enum.auto()
    MOVE2_REG_ADDR = enum.auto()
    MOVE2_REG_CONST = enum.auto()
    MOVE2_ADDR_REG = enum.auto()
    MOVE2_ADDR_ADDR = enum.auto()
    MOVE2_ADDR_CONST = enum.auto()

    MOVE4_REG_REG = enum.auto()
    MOVE4_REG_ADDR = enum.auto()
    MOVE4_REG_CONST = enum.auto()
    MOVE4_ADDR_REG = enum.auto()
    MOVE4_ADDR_ADDR = enum.auto()
    MOVE4_ADDR_CONST = enum.auto()

    MOVE8_REG_REG = enum.auto()
    MOVE8_REG_ADDR = enum.auto()
    MOVE8_REG_CONST = enum.auto()
    MOVE8_ADDR_REG = enum.auto()
    MOVE8_ADDR_ADDR = enum.auto()
    MOVE8_ADDR_CONST = enum.auto()


    STORE1_REG_REG = enum.auto()
    STORE1_ADDR_REG = enum.auto()
    STORE1_CONST_REG = enum.auto()

    STORE2_REG_REG = enum.auto()
    STORE2_ADDR_REG = enum.auto()
    STORE2_CONST_REG = enum.auto()

    STORE4_REG_REG = enum.auto()
    STORE4_ADDR_REG = enum.auto()
    STORE4_CONST_REG = enum.auto()

    STORE8_REG_REG = enum.auto()
    STORE8_ADDR_REG = enum.auto()
    STORE8_CONST_REG = enum.auto()

    # Control flow

    LABEL = enum.auto()

    JUMP = enum.auto()
    JUMP_IF_TRUE_REG = enum.auto()
    JUMP_IF_FALSE_REG = enum.auto()

    # Comparison

    COMPARE_REG_REG = enum.auto()
    COMPARE_REG_CONST = enum.auto()
    COMPARE_CONST_REG = enum.auto()
    COMPARE_CONST_CONST = enum.auto()


    def __str__(self) -> str:
        return byte_code_names[self.value]

    def __repr__(self) -> str:
        return str(self)


def is_jump_instruction(instruction: ByteCodes) -> bool:
    return ByteCodes.JUMP <= instruction <= ByteCodes.JUMP_IF_FALSE_REG

