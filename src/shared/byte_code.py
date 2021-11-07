import enum


@enum.unique
class ByteCodes(enum.auto):

    # Arithmetic

    ADD_REG_REG = enum.auto()
    ADD_ADDR_REG = enum.auto()
    ADD_REG_ADDR = enum.auto()
    ADD_REG_CONST = enum.auto()
    ADD_ADDR_CONST = enum.auto()
    ADD_ADDR_ADDR = enum.auto()

    SUB_REG_REG = enum.auto()
    SUB_ADDR_REG = enum.auto()
    SUB_REG_ADDR = enum.auto()
    SUB_REG_CONST = enum.auto()
    SUB_ADDR_CONST = enum.auto()

    MUL_REG_REG = enum.auto()
    MUL_ADDR_REG = enum.auto()
    MUL_REG_ADDR = enum.auto()
    MUL_REG_CONST = enum.auto()
    MUL_ADDR_CONST = enum.auto()

    DIV_REG_REG = enum.auto()
    DIV_ADDR_REG = enum.auto()
    DIV_REG_ADDR = enum.auto()
    DIV_REG_CONST = enum.auto()
    DIV_ADDR_CONST = enum.auto()

    # No operation

    NO_OPERATION = enum.auto()

    # Registers

    LOAD_REG_REG = enum.auto()
    LOAD_REG_ADDR = enum.auto()
    LOAD_REG_CONST = enum.auto()

    MOVE_REG_REG = enum.auto()
    MOVE_REG_CONST = enum.auto()
    MOVE_ADDR_REG = enum.auto()
    MOVE_REG_ADDR = enum.auto()
    MOVE_ADDR_ADDR = enum.auto()
    MOVE_ADDR_CONST = enum.auto()

    # Control flow



