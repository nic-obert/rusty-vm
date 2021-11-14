from typing import Dict, List

from shared.byte_code import ByteCodes, is_jump_instruction
from tokenizer import tokenize_operands
from arguments_table import arguments_table
from token_to_byte_code import token_conversion_table


def assemble(assembly: List[str]) -> bytes:
    byte_code = bytearray()

    # Map of labels to their byte code location
    label_map: Dict[str, int] = {}

    for line in assembly:

        line = line.strip()
        if line == '' or line.startswith(';'):
            continue
        
        # List containing either a single operator or an operator and its arguments
        raw_tokens = line.split(' ', 1)

        # List of all the possible byte code instructions associated with the operator
        possible_instructions = arguments_table.get(raw_tokens[0])

        if possible_instructions is None:
            print(f'Unknown instruction: "{raw_tokens[0]}" in line "{line}"')
            exit(1)

        # Branch for operators with operands
        if len(raw_tokens) == 2:
            operands = tokenize_operands(raw_tokens[1]) 

            # Filter out all the possible byte code instructions associated with the operator
            for operand in operands:
                possible_instructions = possible_instructions[operand.type]

            # By now possible_instructions is just a ByteCodes instance because it has been filtered
            possible_instructions: ByteCodes

            # If the operator is a label, store its byte code location
            if possible_instructions == ByteCodes.LABEL:
                label_map[operands[0].value] = len(byte_code)
                continue

            # Substitute the label with the byte code location
            if is_jump_instruction(possible_instructions):
                operands[0].value = label_map[operands[0].value]

            operand_converter = token_conversion_table[possible_instructions]
            operand_bytes = operand_converter(operands)
        
        # Branch for operators without operands
        else:
            operand_bytes = bytes(0)
                
        byte_code.extend(bytes([possible_instructions, *operand_bytes]))
        
    return bytes(byte_code)

