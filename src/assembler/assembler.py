from typing import Callable, Dict, List

from src.shared.byte_code import ByteCodes
from src.shared.registers import Registers
from tokenizer import tokenize_operands, Token, TokenType


def assemble(assembly: List[str]) -> bytes:
    byte_code = bytearray()

    for line in assembly:
        
        operator, operands = line.split(' ', 1)

        operands = tokenize_operands(operands)    
        
        handler = operator_handler[operator]
        byte_code.extend(handler(operands))

    return bytes(byte_code)

