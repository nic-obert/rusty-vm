import pathlib
from sys import argv
from typing import Callable, Dict, List

import files
from byte_code import ByteCodes
from registers import Registers


register_map = \
{
    'a': Registers.A,
    'b': Registers.B,
    'c': Registers.C,
    'd': Registers.D,
    'e': Registers.E,
    'f': Registers.F,
    'g': Registers.G,
    'h': Registers.H,
    'STACK_POINTER': Registers.STACK_POINTER,
    'PROGRAM_COUNTER': Registers.PROGRAM_COUNTER
}


def add_opetator(operands: List[str]) -> bytes:
    op1 = register_map[operands[0]]
    op2 = register_map[operands[1]]
    return bytes([ByteCodes.ADD, op1, op2])

def subtract_operator(operands: List[str]) -> bytes:
    op1 = register_map[operands[0]]
    op2 = register_map[operands[1]]
    return bytes([ByteCodes.SUBTRACT, op1, op2])

def multiply_operator(operands: List[str]) -> bytes:
    op1 = register_map[operands[0]]
    op2 = register_map[operands[1]]
    return bytes([ByteCodes.MULTIPLY, op1, op2])

def divide_operator(operands: List[str]) -> bytes:
    op1 = register_map[operands[0]]
    op2 = register_map[operands[1]]
    return bytes([ByteCodes.DIVIDE, op1, op2])


operator_handler: Dict[str, Callable[[List[str]], bytes]] = \
{
    'add': add_opetator,
    'subtract': subtract_operator,
    'multiply': multiply_operator,
    'divide': divide_operator
}



def assemble(assembly: List[str]) -> bytes:
    byte_code = bytearray()

    for line in assembly:
        line = line.strip()
        if line.startswith(';') or line == '':
            continue
        
        pieces = [ piece.split(',') for piece in line.split(' ') ]
        operator = pieces[0]
        operands = pieces[1:]
        handler = operator_handler[operator]
        byte_code.extend(handler(operands))

    return bytes(byte_code)


def main() -> None:
    if len(argv) != 2:
        print("Usage: python3 assembler.py <file_path>")
        exit(1)

    assembly = files.load_file(argv[1])
    byte_code = assemble(assembly)
    
    new_file_name = pathlib.Path(argv[1]).stem + '.bc'
    files.save_byte_code(byte_code, new_file_name)


if __name__ == "__main__":
    main()

