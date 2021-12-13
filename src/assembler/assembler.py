from typing import Dict, List

from shared.byte_code import ByteCodes, is_jump_instruction
from tokenizer import tokenize_operands
from arguments_table import arguments_table
from token_to_byte_code import instruction_conversion_table


def assemble(assembly: List[str], verbose: bool=False) -> bytes:
    byte_code = bytearray()

    # Map of labels to their byte code location
    label_map: Dict[str, int] = {}

    line_number = 0
    for line in assembly:
        line_number += 1

        line = line.strip()
        if line == '' or line.startswith(';'):
            continue
        
        # List containing either a single operator or an operator and its arguments
        raw_tokens = line.split(' ', 1)
        operator = raw_tokens[0]

        # List of all the possible byte code instructions associated with the operator
        possible_instructions = arguments_table.get(operator)

        if possible_instructions is None:
            print(f'Unknown instruction: "{operator}" in line {line_number} "{line}"')
            exit(1)

        if verbose:
            print(f'{line_number}: {line}')
            print(f'    {operator}')

        # Branch for operators with operands (n.b. this is the length of the raw_tokens list, not the number of operands)
        if len(raw_tokens) == 2:
            operands = tokenize_operands(raw_tokens[1]) 

            # Filter out all the possible byte code instructions associated with the operator
            for operand in operands:
                try:
                    possible_instructions = possible_instructions[operand.type]
                except IndexError:
                    print(f'Unknown operand "{operand}" for instruction "{operator}" in line {line_number} "{line}"')
                    exit(1)

            # By now possible_instructions is just a Tuple of ByteCodes and an integer because it has been filtered
            instruction_code, handled_size = possible_instructions
            # Just a type hint for clarity
            instruction_code: ByteCodes
            handled_size: int

            if verbose:
                print(f'    {instruction_code}, {handled_size}')

            # If the operator is a label, store its byte code location
            if instruction_code == ByteCodes.LABEL:
                label_map[operands[0].value] = len(byte_code)
                continue

            # Substitute the label with the byte code location
            if is_jump_instruction(instruction_code):
                operands[0].value = label_map[operands[0].value]

            operand_converter = instruction_conversion_table[instruction_code]

            # Differentiate between operators that require a size specification and those that don't
            if handled_size != 0:
                operand_bytes = operand_converter(operands, handled_size)
            else:
                operand_bytes = operand_converter(operands)
        
        # Branch for operators without operands
        else:
            operand_bytes = bytes(0)
            # In this branch possible_instructions is just a Tuple of ByteCodes and an integer
            instruction_code = possible_instructions[0]
        
        instruction = bytes([instruction_code, *operand_bytes])

        if verbose:
            print(f'    {instruction}')

        byte_code.extend(instruction)
        
    return bytes(byte_code)

