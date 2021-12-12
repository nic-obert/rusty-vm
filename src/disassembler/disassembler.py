from typing import List

from disassembly_table import disassembly_table
from operator_disassembly import operator_disassembly_table


def disassemble(byte_code: bytes) -> List[str]:
    assembly: List[str] = []

    index = 0
    while index < len(byte_code):

        operator = byte_code[index]
        index += 1

        name, args, sizes, is_sized = disassembly_table[operator]
        if is_sized:
            handled_size = byte_code[index]
            index += 1
            name += str(handled_size)
        
        string = f'   \t{operator}:{hex(operator)}   \t\t{name}'
        for arg, size in zip(args, sizes):
            operand_string = operator_disassembly_table[arg](byte_code[index : index + size])
            index += size
            string += f" {operand_string}"

        assembly.append(string)

    return assembly
        

