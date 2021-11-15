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

    "INC1_ADDR_IN_REG",
    "INC1_ADDR_LITERAL",

    "INC2_ADDR_IN_REG",
    "INC2_ADDR_LITERAL",

    "INC4_ADDR_IN_REG",
    "INC4_ADDR_LITERAL",

    "INC8_ADDR_IN_REG",
    "INC8_ADDR_LITERAL",


    "DEC_REG",

    "DEC1_ADDR_IN_REG",
    "DEC1_ADDR_LITERAL",

    "DEC2_ADDR_IN_REG",
    "DEC2_ADDR_LITERAL",

    "DEC4_ADDR_IN_REG",
    "DEC4_ADDR_LITERAL",

    "DEC8_ADDR_IN_REG",
    "DEC8_ADDR_LITERAL",


    "NO_OPERATION",


    "LOAD1_REG_REG",
    "LOAD1_REG_ADDR_IN_REG",
    "LOAD1_REG_CONST",
    "LOAD1_REG_ADDR_LITERAL",

    "LOAD2_REG_REG",
    "LOAD2_REG_ADDR_IN_REG",
    "LOAD2_REG_CONST",
    "LOAD2_REG_ADDR_LITERAL",

    "LOAD4_REG_REG",
    "LOAD4_REG_ADDR_IN_REG",
    "LOAD4_REG_CONST",
    "LOAD4_REG_ADDR_LITERAL",

    "LOAD8_REG_REG",
    "LOAD8_REG_ADDR_IN_REG",
    "LOAD8_REG_CONST",
    "LOAD8_REG_ADDR_LITERAL",


    "MOVE1_REG_REG",
    "MOVE1_REG_ADDR_IN_REG",
    "MOVE1_REG_CONST",
    "MOVE1_REG_ADDR_LITERAL",
    "MOVE1_ADDR_IN_REG_REG",
    "MOVE1_ADDR_IN_REG_ADDR_IN_REG",
    "MOVE1_ADDR_IN_REG_CONST",
    "MOVE1_ADDR_IN_REG_ADDR_LITERAL",
    "MOVE1_ADDR_LITERAL_REG",
    "MOVE1_ADDR_LITERAL_ADDR_IN_REG",   
    "MOVE1_ADDR_LITERAL_CONST",
    "MOVE1_ADDR_LITERAL_ADDR_LITERAL",

    "MOVE2_REG_REG",
    "MOVE2_REG_ADDR_IN_REG",
    "MOVE2_REG_CONST",
    "MOVE2_REG_ADDR_LITERAL",
    "MOVE2_ADDR_IN_REG_REG",
    "MOVE2_ADDR_IN_REG_ADDR_IN_REG",
    "MOVE2_ADDR_IN_REG_CONST",
    "MOVE2_ADDR_IN_REG_ADDR_LITERAL",
    "MOVE2_ADDR_LITERAL_REG",
    "MOVE2_ADDR_LITERAL_ADDR_IN_REG",
    "MOVE2_ADDR_LITERAL_CONST",
    "MOVE2_ADDR_LITERAL_ADDR_LITERAL",

    "MOVE4_REG_REG",
    "MOVE4_REG_ADDR_IN_REG",
    "MOVE4_REG_CONST",
    "MOVE4_REG_ADDR_LITERAL",
    "MOVE4_ADDR_IN_REG_REG",
    "MOVE4_ADDR_IN_REG_ADDR_IN_REG",
    "MOVE4_ADDR_IN_REG_CONST",
    "MOVE4_ADDR_IN_REG_ADDR_LITERAL",
    "MOVE4_ADDR_LITERAL_REG",
    "MOVE4_ADDR_LITERAL_ADDR_IN_REG",
    "MOVE4_ADDR_LITERAL_CONST",
    "MOVE4_ADDR_LITERAL_ADDR_LITERAL",

    "MOVE8_REG_REG",
    "MOVE8_REG_ADDR_IN_REG",
    "MOVE8_REG_CONST",
    "MOVE8_REG_ADDR_LITERAL",
    "MOVE8_ADDR_IN_REG_REG",
    "MOVE8_ADDR_IN_REG_ADDR_IN_REG",
    "MOVE8_ADDR_IN_REG_CONST",
    "MOVE8_ADDR_IN_REG_ADDR_LITERAL",
    "MOVE8_ADDR_LITERAL_REG",
    "MOVE8_ADDR_LITERAL_ADDR_IN_REG",
    "MOVE8_ADDR_LITERAL_CONST",
    "MOVE8_ADDR_LITERAL_ADDR_LITERAL",
    

    "STORE1_REG_REG",
    "STORE1_ADDR_IN_REG_REG",
    "STORE1_ADDR_LITERAL_REG",

    "STORE2_REG_REG",
    "STORE2_ADDR_IN_REG_REG",
    "STORE2_ADDR_LITERAL_REG",

    "STORE4_REG_REG",
    "STORE4_ADDR_IN_REG_REG",
    "STORE4_ADDR_LITERAL_REG",

    "STORE8_REG_REG",
    "STORE8_ADDR_IN_REG_REG",
    "STORE8_ADDR_LITERAL_REG",


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

    INC1_ADDR_IN_REG = enum.auto()
    INC1_ADDR_LITERAL = enum.auto()

    INC2_ADDR_IN_REG = enum.auto()
    INC2_ADDR_LITERAL = enum.auto()

    INC4_ADDR_IN_REG = enum.auto()
    INC4_ADDR_LITERAL = enum.auto()

    INC8_ADDR_IN_REG = enum.auto()
    INC8_ADDR_LITERAL = enum.auto()


    DEC_REG = enum.auto()

    DEC1_ADDR_IN_REG = enum.auto()
    DEC1_ADDR_LITERAL = enum.auto()

    DEC2_ADDR_IN_REG = enum.auto()
    DEC2_ADDR_LITERAL = enum.auto()

    DEC4_ADDR_IN_REG = enum.auto()
    DEC4_ADDR_LITERAL = enum.auto()

    DEC8_ADDR_IN_REG = enum.auto()
    DEC8_ADDR_LITERAL = enum.auto()

    # No operation

    NO_OPERATION = enum.auto()

    # Memory

    LOAD1_REG_REG = enum.auto()
    LOAD1_REG_ADDR_IN_REG = enum.auto()
    LOAD1_REG_CONST = enum.auto()
    LOAD1_REG_ADDR_LITERAL = enum.auto()

    LOAD2_REG_REG = enum.auto()
    LOAD2_REG_ADDR_IN_REG = enum.auto()
    LOAD2_REG_CONST = enum.auto()
    LOAD2_REG_ADDR_LITERAL = enum.auto()

    LOAD4_REG_REG = enum.auto()
    LOAD4_REG_ADDR_IN_REG = enum.auto()
    LOAD4_REG_CONST = enum.auto()
    LOAD4_REG_ADDR_LITERAL = enum.auto()

    LOAD8_REG_REG = enum.auto()
    LOAD8_REG_ADDR_IN_REG = enum.auto()
    LOAD8_REG_CONST = enum.auto()
    LOAD8_REG_ADDR_LITERAL = enum.auto()


    MOVE1_REG_REG = enum.auto()
    MOVE1_REG_ADDR_IN_REG = enum.auto()
    MOVE1_REG_CONST = enum.auto()
    MOVE1_REG_ADDR_LITERAL = enum.auto()
    MOVE1_ADDR_IN_REG_REG = enum.auto()
    MOVE1_ADDR_IN_REG_ADDR_IN_REG = enum.auto()
    MOVE1_ADDR_IN_REG_CONST = enum.auto()
    MOVE1_ADDR_IN_REG_ADDR_LITERAL = enum.auto()
    MOVE1_ADDR_LITERAL_REG = enum.auto()
    MOVE1_ADDR_LITERAL_ADDR_IN_REG = enum.auto()
    MOVE1_ADDR_LITERAL_CONST = enum.auto()
    MOVE1_ADDR_LITERAL_ADDR_LITERAL = enum.auto()

    MOVE2_REG_REG = enum.auto()
    MOVE2_REG_ADDR_IN_REG = enum.auto()
    MOVE2_REG_CONST = enum.auto()
    MOVE2_REG_ADDR_LITERAL = enum.auto()
    MOVE2_ADDR_IN_REG_REG = enum.auto()
    MOVE2_ADDR_IN_REG_ADDR_IN_REG = enum.auto()
    MOVE2_ADDR_IN_REG_CONST = enum.auto()
    MOVE2_ADDR_IN_REG_ADDR_LITERAL = enum.auto()
    MOVE2_ADDR_LITERAL_REG = enum.auto()
    MOVE2_ADDR_LITERAL_ADDR_IN_REG = enum.auto()
    MOVE2_ADDR_LITERAL_CONST = enum.auto()
    MOVE2_ADDR_LITERAL_ADDR_LITERAL = enum.auto()

    MOVE4_REG_REG = enum.auto()
    MOVE4_REG_ADDR_IN_REG = enum.auto()
    MOVE4_REG_CONST = enum.auto()
    MOVE4_REG_ADDR_LITERAL = enum.auto()
    MOVE4_ADDR_IN_REG_REG = enum.auto()
    MOVE4_ADDR_IN_REG_ADDR_IN_REG = enum.auto()
    MOVE4_ADDR_IN_REG_CONST = enum.auto()
    MOVE4_ADDR_IN_REG_ADDR_LITERAL = enum.auto()
    MOVE4_ADDR_LITERAL_REG = enum.auto()
    MOVE4_ADDR_LITERAL_ADDR_IN_REG = enum.auto()
    MOVE4_ADDR_LITERAL_CONST = enum.auto()
    MOVE4_ADDR_LITERAL_ADDR_LITERAL = enum.auto()

    MOVE8_REG_REG = enum.auto()
    MOVE8_REG_ADDR_IN_REG = enum.auto()
    MOVE8_REG_CONST = enum.auto()
    MOVE8_REG_ADDR_LITERAL = enum.auto()
    MOVE8_ADDR_IN_REG_REG = enum.auto()
    MOVE8_ADDR_IN_REG_ADDR_IN_REG = enum.auto()
    MOVE8_ADDR_IN_REG_CONST = enum.auto()
    MOVE8_ADDR_IN_REG_ADDR_LITERAL = enum.auto()
    MOVE8_ADDR_LITERAL_REG = enum.auto()
    MOVE8_ADDR_LITERAL_ADDR_IN_REG = enum.auto()
    MOVE8_ADDR_LITERAL_CONST = enum.auto()
    MOVE8_ADDR_LITERAL_ADDR_LITERAL = enum.auto()


    STORE1_REG_REG = enum.auto()
    STORE1_ADDR_IN_REG_REG = enum.auto()
    STORE1_ADDR_LITERAL_REG = enum.auto()

    STORE2_REG_REG = enum.auto()
    STORE2_ADDR_IN_REG_REG = enum.auto()
    STORE2_ADDR_LITERAL_REG = enum.auto()

    STORE4_REG_REG = enum.auto()
    STORE4_ADDR_IN_REG_REG = enum.auto()
    STORE4_ADDR_LITERAL_REG = enum.auto()

    STORE8_REG_REG = enum.auto()
    STORE8_ADDR_IN_REG_REG = enum.auto()
    STORE8_ADDR_LITERAL_REG = enum.auto()

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

