from typing import List

from disassembly_table import disassembly_table
from operator_disassembly import operator_disassembly_table


def disassemble(byte_code: bytes) -> List[str]:
    assembly: List[str] = []

    print(f'Total bytes: {len(byte_code)}')

    print('LINE\tOPERATOR\t\tASSEMBLY\tSIZE, INDEX')

    index = 0
    line_number = 0
    while index < len(byte_code):

        start_index = index
        operator = byte_code[index]
        index += 1
        
        name, args, sizes, sized_indexes = disassembly_table[operator]
        
        # Check if the operator has sized operands
        if sized_indexes:
            # Get the size of the operands
            handled_size = byte_code[index]
            index += 1
            name += str(handled_size)

            # Update the operand sizes
            new_sizes = list(sizes)
            for arg_index in sized_indexes:
                new_sizes[arg_index] = handled_size
            sizes = tuple(new_sizes)

        string = f'{line_number}:   \t{operator}:{hex(operator)}   \t\t{name}'

        # Disassemble the operands
        for arg, size in zip(args, sizes):
            operand_string = operator_disassembly_table[arg](byte_code[index : index + size])
            index += size
            string += f' {operand_string}'

        string += f'     \t({index - start_index}, {start_index})'
        assembly.append(string)

    return assembly
        
