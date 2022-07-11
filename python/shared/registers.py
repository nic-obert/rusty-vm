import enum
from typing import Dict, Tuple, List


@enum.unique
class Registers(enum.IntEnum):

    def _generate_next_value_(name: str, start: int, count: int, last_values: List[int]) -> int:
        return count

    # General purpose registers

    A = enum.auto() # First arithmetical register
    B = enum.auto() # Second arithmetical register

    C = enum.auto()
    D = enum.auto()

    # Special registers

    EXIT = enum.auto() # Exit status register
    INPUT = enum.auto() # External input register
    ERROR = enum.auto() # Error code register
    PRINT = enum.auto() # Print register

    # Stack registers

    STACK_POINTER = enum.auto()

    PROGRAM_COUNTER = enum.auto()

    # Flags

    ZERO_FLAG = enum.auto()
    SIGN_FLAG = enum.auto()
    REMAINDER_FLAG = enum.auto()


register_names: Tuple[str] = \
(
    "a",
    "b",
    "c",
    "d",
    "exit",
    "input",
    "error",
    "print",

    "sp",

    "pc"

    "zf",
    "sf",
    "rf",
)


register_table: Dict[str, Registers] = \
{
    'a': Registers.A,
    'b': Registers.B,
    'c': Registers.C,
    'd': Registers.D,
    'exit': Registers.EXIT,
    'input': Registers.INPUT,
    'error': Registers.ERROR,
    'print': Registers.PRINT,
    'sp': Registers.STACK_POINTER,
    'pc': Registers.PROGRAM_COUNTER,
    'zf': Registers.ZERO_FLAG,
    'sf': Registers.SIGN_FLAG,
    'rf': Registers.REMAINDER_FLAG,
}

