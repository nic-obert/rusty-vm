from typing import Callable, Dict, List

from src.shared.byte_code import ByteCodes
from src.shared.registers import Registers


register_map: Dict[str, Registers] = \
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


def get_number(string: str) -> int:
    if string.startswith('0x'):
        return int(string, 16)
    else:
        return int(string)


# Operator handlers

def add_opetator(operands: List[str]) -> bytes:
    # Destination is a register
    try:
        destination = register_map[operands[0]]
        try:
            # Source is a register
            source = register_map[operands[1]]
            instruction = ByteCodes.ADD_REG_REG
        except KeyError:
            # Source is a memory address
            if operands[1].startswith('[') and operands[1].endswith(']'):
                address = operands[1][1:-1]
                # Source is a memory address stored in a register
                try:
                    source = register_map[address]
                    instruction = ByteCodes.ADD_REG_ADDR
                source = get_number(operands[1][1:-1])
                instruction = ByteCodes.ADD_REG_ADDR
            # Source is a literal
            else:
                source = get_number(operands[1])
                instruction = ByteCodes.ADD_REG_CONST
    # Destination is a memory address
    except KeyError:
        if operands[0].startswith('[') and operands[0].endswith(']'):
            destination = get_number(operands[0][1:-1])
            try:
                # Source is a register
                source = register_map[operands[1]]
                instruction = ByteCodes.ADD_ADDR_REG
            except KeyError:
                # Source is a memory address
                if operands[1].startswith('[') and operands[1].endswith(']'):
                    source = get_number(operands[1][1:-1])
                    instruction = ByteCodes.ADD_ADDR_ADDR
                # Source is a literal
                else:
                    source = get_number(operands[1])
                    instruction = ByteCodes.ADD_ADDR_CONST
            
    return bytes([instruction, destination, source])


def subtract_operator(operands: List[str]) -> bytes:
    # Destination is a register
    try:
        destination = register_map[operands[0]]
        try:
            # Source is a register
            source = register_map[operands[1]]
            instruction = ByteCodes.SUB_REG_REG
        except KeyError:
            # Source is a memory address
            if operands[1].startswith('[') and operands[1].endswith(']'):
                source = get_number(operands[1][1:-1])
                instruction = ByteCodes.SUB_REG_ADDR
            # Source is a literal
            else:
                source = get_number(operands[1])
                instruction = ByteCodes.SUB_REG_CONST
    # Destination is a memory address
    except KeyError:
        if operands[0].startswith('[') and operands[0].endswith(']'):
            destination = get_number(operands[0][1:-1])
            try:
                # Source is a register
                source = register_map[operands[1]]
                instruction = ByteCodes.SUB_ADDR_REG
            except KeyError:
                # Source is a memory address
                if operands[1].startswith('[') and operands[1].endswith(']'):
                    source = get_number(operands[1][1:-1])
                    instruction = ByteCodes.SUB_ADDR_ADDR
                # Source is a literal
                else:
                    source = get_number(operands[1])
                    instruction = ByteCodes.SUB_ADDR_CONST
    
    return bytes([instruction, destination, source])


def multiply_operator(operands: List[str]) -> bytes:
    # Destination is a register
    try:
        destination = register_map[operands[0]]
        try:
            # Source is a register
            source = register_map[operands[1]]
            instruction = ByteCodes.MUL_REG_REG
        except KeyError:
            # Source is a memory address
            if operands[1].startswith('[') and operands[1].endswith(']'):
                source = get_number(operands[1][1:-1])
                instruction = ByteCodes.MUL_REG_ADDR
            # Source is a literal
            else:
                source = get_number(operands[1])
                instruction = ByteCodes.MUL_REG_CONST
    # Destination is a memory address
    except KeyError:
        if operands[0].startswith('[') and operands[0].endswith(']'):
            destination = get_number(operands[0][1:-1])
            try:
                # Source is a register
                source = register_map[operands[1]]
                instruction = ByteCodes.MUL_ADDR_REG
            except KeyError:
                # Source is a memory address
                if operands[1].startswith('[') and operands[1].endswith(']'):
                    source = get_number(operands[1][1:-1])
                    instruction = ByteCodes.MUL_ADDR_ADDR
                # Source is a literal
                else:
                    source = get_number(operands[1])
                    instruction = ByteCodes.MUL_ADDR_CONST
    
    return bytes([instruction, destination, source])


def divide_operator(operands: List[str]) -> bytes:
    # Destination is a register
    try:
        destination = register_map[operands[0]]
        try:
            # Source is a register
            source = register_map[operands[1]]
            instruction = ByteCodes.DIV_REG_REG
        except KeyError:
            # Source is a memory address
            if operands[1].startswith('[') and operands[1].endswith(']'):
                source = get_number(operands[1][1:-1])
                instruction = ByteCodes.DIV_REG_ADDR
            # Source is a literal
            else:
                source = get_number(operands[1])
                instruction = ByteCodes.DIV_REG_CONST
    # Destination is a memory address
    except KeyError:
        if operands[0].startswith('[') and operands[0].endswith(']'):
            destination = get_number(operands[0][1:-1])
            try:
                # Source is a register
                source = register_map[operands[1]]
                instruction = ByteCodes.DIV_ADDR_REG
            except KeyError:
                # Source is a memory address
                if operands[1].startswith('[') and operands[1].endswith(']'):
                    source = get_number(operands[1][1:-1])
                    instruction = ByteCodes.DIV_ADDR_ADDR
                # Source is a literal
                else:
                    source = get_number(operands[1])
                    if source == 0:
                        print('Error: Division by zero')
                        exit(1)
                    instruction = ByteCodes.DIV_ADDR_CONST

    return bytes([instruction, destination, source])


def load_operator(operands: List[str]) -> bytes:
    # Destination is always gister
    try:
        destination = register_map[operands[0]]
        try:
            # Source is a register
            source = register_map[operands[1]]
            instruction = ByteCodes.LOAD_REG_REG
        except KeyError:
            # Source is a memory address
            if operands[1].startswith('[') and operands[1].endswith(']'):
                source = get_number(operands[1][1:-1])
                instruction = ByteCodes.LOAD_REG_ADDR
            # Source is a literal
            else:
                source = get_number(operands[1])
                instruction = ByteCodes.LOAD_REG_CONST
    
    except KeyError:
        print(f'Error: Invalid register {operands[0]}')
        exit(1)
    
    return bytes([instruction, destination, source])


def move_operator(operands: List[str]) -> bytes:
    pass
    


operator_handler: Dict[str, Callable[[List[str]], bytes]] = \
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
        line = line.strip()
        if line.startswith(';') or line == '':
            continue
        
        pieces = [ piece.split(',') for piece in line.split(' ') ]
        operator = pieces[0]
        operands = pieces[1:]
        handler = operator_handler[operator]
        byte_code.extend(handler(operands))

    return bytes(byte_code)


