from typing import Callable, List, Tuple

from tokenizer import Token


def number_size(number: int) -> int:
    """
    Returns the number of bytes needed to represent the number.
    """
    if number == 0:
        return 1

    size = 0
    while number != 0:
        number = number // 256
        size += 1

    return size


def number_to_bytes(number: int, size: int) -> bytes:
    """
    Returns the bytes representation of the number.
    """
    if number_size(number) > size:
        print(f'Number {number} cannot fit in {size} bytes')
        exit(1)

    value = bytearray(size)
    for i in range(size):
        value[i] = number % 256
        number = number // 256

    return bytes(value)


def sized_operator_bytes_handled(operator: str) -> int:
    """
    Returns the number of bytes a sized operator handles.
    The output size should always be representable in a single byte.
    """
    return int(operator[-1])



"""
The following functions are used to convert the operand tokens to bytes.
"""
instruction_conversion_table: Tuple[Callable[[str, List[Token]], bytes]] = \
(
    # Arithmetic

    # ByteCodes.ADD
    lambda operator, operands: bytes(0),

    # ByteCodes.SUB
    lambda operator, operands: bytes(0),

    # ByteCodes.MUL
    lambda operator, operands: bytes(0),

    # ByteCodes.DIV
    lambda operator, operands: bytes(0),

    # ByteCodes.MOD
    lambda operator, operands: bytes(0),


    # ByteCodes.INC_REG
    lambda operator, operands: bytes((
        *number_to_bytes(operands[0].value, 1), 
    )),

    # ByteCodes.INC_ADDR_IN_REG
    lambda operator, operands: bytes((
        sized_operator_bytes_handled(operator),
        *number_to_bytes(operands[0].value, 1),
    )),
    # ByteCodes.INC_ADDR_LITERAL
    lambda operator, operands: bytes((
        sized_operator_bytes_handled(operator),
        *number_to_bytes(operands[0].value, 8),
    )),

    # ByteCodes.DEC_REG
    lambda operator, operands: bytes((
        *number_to_bytes(operands[0].value, 1),
    )),

    # ByteCodes.DEC_ADDR_IN_REG
    lambda operator, operands: bytes((
        sized_operator_bytes_handled(operator),
        *number_to_bytes(operands[0].value, 1),
    )),
    # ByteCodes.DEC_ADDR_LITERAL
    lambda operator, operands: bytes((
        sized_operator_bytes_handled(operator),
        *number_to_bytes(operands[0].value, 8),
    )),

    # No operation
    # ByteCodes.NOP
    lambda operator, operands: bytes(0),

    # Memory

    # ByteCodes.MOVE_REG_REG
    lambda operator, operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1),
    )),

    # ByteCodes.MOVE_REG_ADDR_IN_REG
    lambda operator, operands: bytes((
        sized_operator_bytes_handled(operator),
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.MOVE_REG_CONST
    lambda operator, operands: bytes((
        sized_operator_bytes_handled(operator),
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.MOVE_REG_ADDR_LITERAL
    lambda operator, operands: bytes((
        sized_operator_bytes_handled(operator),
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 8),
    )),
    # ByteCodes.MOVE_ADDR_IN_REG_REG
    lambda operator, operands: bytes((
        sized_operator_bytes_handled(operator),
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.MOVE_ADDR_IN_REG_ADDR_IN_REG
    lambda operator, operands: bytes((
        sized_operator_bytes_handled(operator),
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.MOVE_ADDR_IN_REG_CONST
    lambda operator, operands: bytes((
        sized_operator_bytes_handled(operator),
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.MOVE_ADDR_IN_REG_ADDR_LITERAL
    lambda operator, operands: bytes((
        sized_operator_bytes_handled(operator),
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 8),
    )),
    # ByteCodes.MOVE_ADDR_LITERAL_REG
    lambda operator, operands: bytes((
        sized_operator_bytes_handled(operator),
        *number_to_bytes(operands[0].value, 8),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.MOVE_ADDR_LITERAL_ADDR_IN_REG
    lambda operator, operands: bytes((
        sized_operator_bytes_handled(operator),
        *number_to_bytes(operands[0].value, 8),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.MOVE_ADDR_LITERAL_CONST
    lambda operator, operands: bytes((
        sized_operator_bytes_handled(operator),
        *number_to_bytes(operands[0].value, 8),
        *number_to_bytes(operands[1].value, 1),
    )),
    # ByteCodes.MOVE_ADDR_LITERAL_ADDR_LITERAL
    lambda operator, operands: bytes((
        sized_operator_bytes_handled(operator),
        *number_to_bytes(operands[0].value, 8),
        *number_to_bytes(operands[1].value, 8),
    )),

    # Stack

    # ByteCodes.PUSH_REG
    lambda operator, operands: bytes((
        *number_to_bytes(operands[0].value, 1),
    )),

    # ByteCodes.PUSH_ADDR_IN_REG
    lambda operator, operands: bytes((
        sized_operator_bytes_handled(operator),
        *number_to_bytes(operands[0].value, 1),
    )),
    # ByteCodes.PUSH_CONST
    lambda operator, operands: bytes((
        sized_operator_bytes_handled(operator),
        *number_to_bytes(operands[0].value, 1),
    )),
    # ByteCodes.PUSH_ADDR_LITERAL
    lambda operator, operands: bytes((
        sized_operator_bytes_handled(operator),
        *number_to_bytes(operands[0].value, 8),
    )),

    # ByteCodes.POP_REG
    lambda operator, operands: bytes((
        *number_to_bytes(operands[0].value, 1),
    )),

    # ByteCodes.POP_ADDR_IN_REG
    lambda operator, operands: bytes((
        sized_operator_bytes_handled(operator),
        *number_to_bytes(operands[0].value, 1),
    )),
    # ByteCodes.POP_ADDR_LITERAL
    lambda operator, operands: bytes((
        sized_operator_bytes_handled(operator),
        *number_to_bytes(operands[0].value, 8),
    )),

    # Control flow

    # ByteCodes.LABEL
    lambda operator, operands: bytes(0),

    # ByteCodes.JUMP
    lambda operator, operands: bytes((
        *number_to_bytes(operands[0].value, 8), # Argument is an 8-byte address
    )),

    # ByteCodes.JUMP_IF_TRUE_REG
    lambda operator, operands: bytes((
        *number_to_bytes(operands[0].value, 8), # Argument is an 8-byte address
        *number_to_bytes(operands[1].value, 1)
    )),

    # ByteCodes.JUMP_IF_FALSE_REG
    lambda operator, operands: bytes((
        *number_to_bytes(operands[0].value, 8), # Argument is an 8-byte address
        *number_to_bytes(operands[1].value, 1)
    )),

    # Comparison

    # ByteCodes.COMPARE_REG_REG
    lambda operator, operands: bytes((
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1)
    )),

    # ByteCodes.COMPARE_REG_CONST
    lambda operator, operands: bytes((
        sized_operator_bytes_handled(operator),
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1)
    )),

    # ByteCodes.COMPARE_CONST_REG
    lambda operator, operands: bytes((
        sized_operator_bytes_handled(operator),
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1)
    )),

    # ByteCodes.COMPARE_CONST_CONST
    lambda operator, operands: bytes((
        sized_operator_bytes_handled(operator),
        *number_to_bytes(operands[0].value, 1),
        *number_to_bytes(operands[1].value, 1)
    )),

    # Interrupts

    # ByteCodes.PRINT
    lambda operator, operands: bytes(0),

    # ByteCodes.PRINT_STRING
    lambda operator, operands: bytes(0),

    # ByteCodes.INPUT_INT
    lambda operator, operands: bytes(0),

    # ByteCodes.INPUT_STRING
    lambda operator, operands: bytes(0),

    # ByteCodes.EXIT
    lambda operator, operands: bytes(0),
    
    
)

