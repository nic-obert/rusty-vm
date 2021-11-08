from typing import Callable, Dict, List

from src.shared.byte_code import ByteCodes
from src.shared.registers import Registers
from tokenizer import tokenize_operands, Token, TokenType


# Operator handlers

def add_opetator(operands: List[Token]) -> bytes:
    instruction: ByteCodes
    if operands[0].type == TokenType.REGISTER:
        if operands[1].type == TokenType.REGISTER:
            instruction = ByteCodes.ADD_REG_REG
        elif operands[1].type == TokenType.:
            


def subtract_operator(operands: List[Token]) -> bytes:
    pass



def multiply_operator(operands: List[Token]) -> bytes:
    pass



def divide_operator(operands: List[Token]) -> bytes:
    pass



def load_operator(operands: List[Token]) -> bytes:
    pass



def move_operator(operands: List[Token]) -> bytes:
    pass
    


operator_handler: Dict[str, Callable[[List[Token]], bytes]] = \
{
    'add': add_opetator,
    'sub': subtract_operator,
    'mul': multiply_operator,
    'div': divide_operator,
    'nop': lambda _: bytes([ByteCodes.NOP]),
    'ld': load_operator,
}


def assemble(assembly: List[str]) -> bytes:
    byte_code = bytearray()

    for line in assembly:
        
        operator, operands = line.split(' ', 1)

        operands = tokenize_operands(operands)    
        
        handler = operator_handler[operator]
        byte_code.extend(handler(operands))

    return bytes(byte_code)

