import enum


@enum.unique
class ByteCodes(enum.IntEnum):

    # Arithmetic

    ADD = enum.auto()
    SUB = enum.auto()
    MUL = enum.auto()
    DIV = enum.auto()
    MOD = enum.auto()

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

    # Control flow



