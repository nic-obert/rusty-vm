import enum

@enum.unique
class Registers(enum.IntEnum):

    # General purpose registers
    A = 0
    B = 1
    C = 2
    D = 3
    E = 4
    F = 5
    G = 6
    H = 7

    # Stack pointer
    STACK_POINTER = 8

    # Program counter
    PROGRAM_COUNTER = 9

