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
    "INC_ADDR_IN_REG",
    "INC_ADDR_LITERAL",

    "DEC_REG",
    "DEC_ADDR_IN_REG",
    "DEC_ADDR_LITERAL",


    "NO_OPERATION",


    "MOVE_REG_REG",
    "MOVE_REG_ADDR_IN_REG",
    "MOVE_REG_CONST",
    "MOVE_REG_ADDR_LITERAL",
    "MOVE_ADDR_IN_REG_REG",
    "MOVE_ADDR_IN_REG_ADDR_IN_REG",
    "MOVE_ADDR_IN_REG_CONST",
    "MOVE_ADDR_IN_REG_ADDR_LITERAL",
    "MOVE_ADDR_LITERAL_REG",
    "MOVE_ADDR_LITERAL_ADDR_IN_REG",   
    "MOVE_ADDR_LITERAL_CONST",
    "MOVE_ADDR_LITERAL_ADDR_LITERAL",


    "PUSH_REG",
    "PUSH_ADDR_IN_REG",
    "PUSH_CONST",
    "PUSH_ADDR_LITERAL",

    "POP_REG",
    "POP_ADDR_IN_REG",
    "POP_ADDR_LITERAL",


    "LABEL",

    "JUMP",
    "JUMP_IF_TRUE_REG",
    "JUMP_IF_FALSE_REG",


    "COMPARE_REG_REG",
    "COMPARE_REG_CONST",
    "COMPARE_CONST_REG",
    "COMPARE_CONST_CONST",


    "PRINT",
    "PRINT_STRING",

    "INPUT_INT",
    "INPUT_STRING"

    "EXIT",

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
    INC_ADDR_IN_REG = enum.auto()
    INC_ADDR_LITERAL = enum.auto()

    DEC_REG = enum.auto()
    DEC_ADDR_IN_REG = enum.auto()
    DEC_ADDR_LITERAL = enum.auto()

    # No operation

    NO_OPERATION = enum.auto()

    # Memory

    MOVE_REG_REG = enum.auto()
    MOVE_REG_ADDR_IN_REG = enum.auto()
    MOVE_REG_CONST = enum.auto()
    MOVE_REG_ADDR_LITERAL = enum.auto()
    MOVE_ADDR_IN_REG_REG = enum.auto()
    MOVE_ADDR_IN_REG_ADDR_IN_REG = enum.auto()
    MOVE_ADDR_IN_REG_CONST = enum.auto()
    MOVE_ADDR_IN_REG_ADDR_LITERAL = enum.auto()
    MOVE_ADDR_LITERAL_REG = enum.auto()
    MOVE_ADDR_LITERAL_ADDR_IN_REG = enum.auto()
    MOVE_ADDR_LITERAL_CONST = enum.auto()
    MOVE_ADDR_LITERAL_ADDR_LITERAL = enum.auto()

    # Stack

    PUSH_REG = enum.auto()
    PUSH_ADDR_IN_REG = enum.auto()
    PUSH_CONST = enum.auto()
    PUSH_ADDR_LITERAL = enum.auto()

    POP_REG = enum.auto()
    POP_ADDR_IN_REG = enum.auto()
    POP_ADDR_LITERAL = enum.auto()

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

    # Interrupts

    PRINT = enum.auto()
    PRINT_STRING = enum.auto()

    INPUT_INT = enum.auto()
    INPUT_STRING = enum.auto()

    EXIT = enum.auto()


    def __str__(self) -> str:
        return byte_code_names[self.value]

    def __repr__(self) -> str:
        return str(self)


def is_jump_instruction(instruction: ByteCodes) -> bool:
    return ByteCodes.JUMP <= instruction <= ByteCodes.JUMP_IF_FALSE_REG

