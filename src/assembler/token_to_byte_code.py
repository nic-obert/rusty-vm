from typing import Callable, List
from shared.byte_code import ByteCodes
from tokenizer import Token


def number_size(number: int) -> int:
    if number == 0:
        return 1

    size = 0
    while number != 0:
        number = number // 256
        size += 1

    return size


def number_to_bytes(number: int, size: int) -> bytes:
    if number / 256 > size:
        print(f'Number {number} cannot fit in {size} bytes')
        exit(1)

    value = bytearray(size)
    for i in range(size):
        value[i] = number % 256
        number = number // 256

    return bytes(value)


token_conversion_table: List[Callable[[List[Token]], bytes]] = \
[
    # Arithmetic

    lambda operands: bytes([
        ByteCodes.ADD,
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1)
    ]),
    lambda operands: bytes([
        ByteCodes.SUB,
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1)
    ]),
    lambda operands: bytes([
        ByteCodes.MUL,
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1)
    ]),
    lambda operands: bytes([
        ByteCodes.DIV,
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1)
    ]),
    lambda operands: bytes([
        ByteCodes.MOD,
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1)
    ]),

    lambda operands: bytes([
        ByteCodes.INC_REG,
        *number_to_bytes(operands[0].value, 1)
    ]),
    lambda operands: bytes([
        ByteCodes.INC1_ADDR,
        *number_to_bytes(operands[0].value, 1)
    ]),
    lambda operands: bytes([
        ByteCodes.INC2_ADDR,
        *number_to_bytes(operands[0].value, 2)
    ]),
    lambda operands: bytes([
        ByteCodes.INC4_ADDR,
        *number_to_bytes(operands[0].value, 4)
    ]),
    lambda operands: bytes([
        ByteCodes.INC8_ADDR,
        *number_to_bytes(operands[0].value, 8)
    ]),

    lambda operands: bytes([
        ByteCodes.DEC_REG,
        *number_to_bytes(operands[0].value, 1)
    ]),
    lambda operands: bytes([
        ByteCodes.DEC1_ADDR,
        *number_to_bytes(operands[0].value, 1)
    ]),
    lambda operands: bytes([
        ByteCodes.DEC2_ADDR,
        *number_to_bytes(operands[0].value, 2)
    ]),
    lambda operands: bytes([
        ByteCodes.DEC4_ADDR,
        *number_to_bytes(operands[0].value, 4)
    ]),
    lambda operands: bytes([
        ByteCodes.DEC8_ADDR,
        *number_to_bytes(operands[0].value, 8)
    ]),

    # No operation

    lambda operands: bytes([]),

    # Memory

    lambda operands: bytes([
        ByteCodes.LOAD1_REG_REG,
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1)
    ]),
    lambda operands: bytes([
        ByteCodes.LOAD1_REG_ADDR,
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 8),
    ]),
    lambda operands: bytes([
        ByteCodes.LOAD1_REG_CONST,
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, number_size(operands[1].value))
    ]),
    
    lambda operands: bytes([
        ByteCodes.LOAD2_REG_REG,
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1)
    ]),
    lambda operands: bytes([
        ByteCodes.LOAD2_REG_ADDR,
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 8),
    ]),
    lambda operands: bytes([
        ByteCodes.LOAD2_REG_CONST,
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, number_size(operands[1].value))
    ]),

    lambda operands: bytes([
        ByteCodes.LOAD4_REG_REG,
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1)
    ]),
    lambda operands: bytes([
        ByteCodes.LOAD4_REG_ADDR,
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 8),
    ]),
    lambda operands: bytes([
        ByteCodes.LOAD4_REG_CONST,
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, number_size(operands[1].value))
    ]),

    lambda operands: bytes([
        ByteCodes.LOAD8_REG_REG,
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1)
    ]),
    lambda operands: bytes([
        ByteCodes.LOAD8_REG_ADDR,
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 8),
    ]),
    lambda operands: bytes([
        ByteCodes.LOAD8_REG_CONST,
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, number_size(operands[1].value))
    ]),

    
,
]

