from typing import List

from src.shared.byte_code import ByteCodes
from tokenizer import tokenize_operands
from arguments_table import arguments_table


def assemble(assembly: List[str]) -> bytes:
    byte_code = bytearray()

    for line in assembly:
        
        operator, operands = line.split(' ', 1)

        operands = tokenize_operands(operands)    
        
        options = arguments_table[operator]

        for operand in operands:
            options = options[operand.type]

        options: ByteCodes

        instruction = bytes([options, operands[0].value, operands[1].value])

        byte_code.extend(instruction)

    return bytes(byte_code)

