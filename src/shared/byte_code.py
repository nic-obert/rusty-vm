import enum


@enum.unique
class ByteCodes(enum.IntEnum):

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

    JUMP = enum.auto()
    JUMP_IF_TRUE_REG = enum.auto()
    JUMP_IF_TRUE_ADDR = enum.auto()
    JUMP_IF_FALSE_REG = enum.auto()
    JUMP_IF_FALSE_ADDR = enum.auto()

