from typing import Callable, List, Tuple

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
    if number_size(number) > size:
        print(f'Number {number} cannot fit in {size} bytes')
        exit(1)

    value = bytearray(size)
    for i in range(size):
        value[i] = number % 256
        number = number // 256

    return bytes(value)


token_conversion_table: Tuple[Callable[[List[Token]], bytes]] = \
(
    # Arithmetic

    # ByteCodes.ADD
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1)
    )),

    # ByteCodes.SUB
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1)
    )),

    # ByteCodes.MUL
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1)
    )),

    # ByteCodes.DIV
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1)
    )),

    # ByteCodes.MOD
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1)
    )),


    # ByteCodes.INC_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1), 
    )),

    # ByteCodes.INC1_ADDR_IN_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
    )),
    # ByteCodes.INC1_ADDR_LITERAL
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8),
    )),

    # ByteCodes.INC2_ADDR_IN_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
    )),
    # ByteCodes.INC2_ADDR_LITERAL
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8),
    )),

    # ByteCodes.INC4_ADDR_IN_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
    )),
    # ByteCodes.INC4_ADDR_LITERAL
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8),
    )),

    # ByteCodes.INC8_ADDR_IN_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
    )),
    # ByteCodes.INC8_ADDR_LITERAL
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8),
    )),


    # ByteCodes.DEC_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
    )),

    # ByteCodes.DEC1_ADDR_IN_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
    )),
    # ByteCodes.DEC1_ADDR_LITERAL
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8),
    )),

    # ByteCodes.DEC2_ADDR_IN_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
    )),
    # ByteCodes.DEC2_ADDR_LITERAL
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8),
    )),

    # ByteCodes.DEC4_ADDR_IN_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
    )),
    # ByteCodes.DEC4_ADDR_LITERAL
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8),
    )),

    # ByteCodes.DEC8_ADDR_IN_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
    )),
    # ByteCodes.DEC8_ADDR_LITERAL
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8),
    )),


    # No operation
    # ByteCodes.NOP
    lambda operands: bytes(0),

    # Memory

    # ByteCodes.LOAD_REG_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1),
    )),

    # ByteCodes.LOAD1_REG_ADDR_IN_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.LOAD1_REG_CONST
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.LOAD1_REG_ADDR_LITERAL
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 8),
    )),

    # ByteCodes.LOAD2_REG_ADDR_IN_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.LOAD2_REG_CONST
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 2),
    )),
    # ByteCodes.LOAD2_REG_ADDR_LITERAL
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 8),
    )),

    # ByteCodes.LOAD4_REG_ADDR_IN_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.LOAD4_REG_CONST
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 4),
    )),
    # ByteCodes.LOAD4_REG_ADDR_LITERAL
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 8),
    )),

    # ByteCodes.LOAD8_REG_ADDR_IN_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.LOAD8_REG_CONST
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 8),
    )),
    # ByteCodes.LOAD8_REG_ADDR_LITERAL
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 8),
    )),


    # ByteCodes.MOVE_REG_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1),
    )),

    # ByteCodes.MOVE1_REG_ADDR_IN_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.MOVE1_REG_CONST
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.MOVE1_REG_ADDR_LITERAL
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 8),
    )),
    # ByteCodes.MOVE1_ADDR_IN_REG_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.MOVE1_ADDR_IN_REG_ADDR_IN_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.MOVE1_ADDR_IN_REG_CONST
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.MOVE1_ADDR_IN_REG_ADDR_LITERAL
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 8),
    )),
    # ByteCodes.MOVE1_ADDR_LITERAL_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.MOVE1_ADDR_LITERAL_ADDR_IN_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.MOVE1_ADDR_LITERAL_CONST
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.MOVE1_ADDR_LITERAL_ADDR_LITERAL
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8),
        *number_to_bytes(operands[1].value, 8),
    )),

    # ByteCodes.MOVE2_REG_ADDR_IN_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.MOVE2_REG_CONST
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 2),
    )),
    # ByteCodes.MOVE2_REG_ADDR_LITERAL
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 8),
    )),
    # ByteCodes.MOVE2_ADDR_IN_REG_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.MOVE2_ADDR_IN_REG_ADDR_IN_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.MOVE2_ADDR_IN_REG_CONST
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 2),
    )),
    # ByteCodes.MOVE2_ADDR_IN_REG_ADDR_LITERAL
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 8),
    )), 
    # ByteCodes.MOVE2_ADDR_LITERAL_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.MOVE2_ADDR_LITERAL_ADDR_IN_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.MOVE2_ADDR_LITERAL_CONST
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8),
        *number_to_bytes(operands[1].value, 2),
    )),
    # ByteCodes.MOVE2_ADDR_LITERAL_ADDR_LITERAL
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8),
        *number_to_bytes(operands[1].value, 8),
    )),

    # ByteCodes.MOVE4_REG_ADDR_IN_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.MOVE4_REG_CONST
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 4),
    )),
    # ByteCodes.MOVE4_REG_ADDR_LITERAL
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 8),
    )),
    # ByteCodes.MOVE4_ADDR_IN_REG_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.MOVE4_ADDR_IN_REG_ADDR_IN_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.MOVE4_ADDR_IN_REG_CONST
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 4),
    )),
    # ByteCodes.MOVE4_ADDR_IN_REG_ADDR_LITERAL
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 8),
    )),
    # ByteCodes.MOVE4_ADDR_LITERAL_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.MOVE4_ADDR_LITERAL_ADDR_IN_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.MOVE4_ADDR_LITERAL_CONST
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8),
        *number_to_bytes(operands[1].value, 4),
    )),
    # ByteCodes.MOVE4_ADDR_LITERAL_ADDR_LITERAL
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8),
        *number_to_bytes(operands[1].value, 8),
    )),

    # ByteCodes.MOVE8_REG_ADDR_IN_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.MOVE8_REG_CONST
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 8),
    )),
    # ByteCodes.MOVE8_REG_ADDR_LITERAL
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 8),
    )),
    # ByteCodes.MOVE8_ADDR_IN_REG_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.MOVE8_ADDR_IN_REG_ADDR_IN_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.MOVE8_ADDR_IN_REG_CONST
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 8),
    )),
    # ByteCodes.MOVE8_ADDR_IN_REG_ADDR_LITERAL
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 8),
    )),
    # ByteCodes.MOVE8_ADDR_LITERAL_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.MOVE8_ADDR_LITERAL_ADDR_IN_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.MOVE8_ADDR_LITERAL_CONST
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8),
        *number_to_bytes(operands[1].value, 8),
    )),
    # ByteCodes.MOVE8_ADDR_LITERAL_ADDR_LITERAL
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8),
        *number_to_bytes(operands[1].value, 8),
    )),
    

    # ByteCodes.STORE1_ADDR_IN_REG_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.STORE1_ADDR_LITERAL_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8),
        *number_to_bytes(operands[1].value, 1),
    )),

    # ByteCodes.STORE2_ADDR_IN_REG_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.STORE2_ADDR_LITERAL_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8),
        *number_to_bytes(operands[1].value, 1),
    )),

    # ByteCodes.STORE4_ADDR_IN_REG_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.STORE4_ADDR_LITERAL_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8),
        *number_to_bytes(operands[1].value, 1),
    )),

    # ByteCodes.STORE8_ADDR_IN_REG_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.STORE8_ADDR_LITERAL_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8),
        *number_to_bytes(operands[1].value, 1),
    )),

    # Control flow

    # ByteCodes.LABEL
    lambda operands: bytes(0),

    # ByteCodes.JUMP
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8), # Argument is an 8-byte address
    )),

    # ByteCodes.JUMP_IF_TRUE_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8), # Argument is an 8-byte address
        *number_to_bytes(operands[1].value, 1)
    )),

    # ByteCodes.JUMP_IF_FALSE_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8), # Argument is an 8-byte address
        *number_to_bytes(operands[1].value, 1)
    )),

    # Comparison

    # ByteCodes.COMPARE_REG_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1)
    )),

    # ByteCodes.COMPARE1_REG_CONST
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1)
    )),

    # ByteCodes.COMPARE1_CONST_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1)
    )),

    # ByteCodes.COMPARE1_CONST_CONST
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1)
    )),

    # ByteCodes.COMPARE2_REG_CONST
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 2)
    )),

    # ByteCodes.COMPARE2_CONST_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 2),
        *number_to_bytes(operands[1].value, 1)
    )),

    # ByteCodes.COMPARE2_CONST_CONST
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 2),
        *number_to_bytes(operands[1].value, 2)
    )),

    # ByteCodes.COMPARE4_REG_CONST
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 4)
    )),

    # ByteCodes.COMPARE4_CONST_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 4),
        *number_to_bytes(operands[1].value, 1)
    )),

    # ByteCodes.COMPARE4_CONST_CONST
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 4),
        *number_to_bytes(operands[1].value, 4)
    )),

    # ByteCodes.COMPARE8_REG_CONST
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 8)
    )),

    # ByteCodes.COMPARE8_CONST_REG
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8),
        *number_to_bytes(operands[1].value, 1)
    )),

    # ByteCodes.COMPARE8_CONST_CONST
    lambda operands: bytes((
        *number_to_bytes(operands[0].value, 8),
        *number_to_bytes(operands[1].value, 8)
    )),

    # Interrupts

    # ByteCodes.PRINT
    lambda operands: bytes(0),

    # ByteCodes.PRINT_STRING
    lambda operands: bytes(0),

    # ByteCodes.EXIT
    lambda operands: bytes(0),
    
    
)

