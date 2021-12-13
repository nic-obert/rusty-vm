from typing import Dict, Tuple, Union

from shared.byte_code import ByteCodes


arguments_table : Dict[
    str,
    Union[
        Tuple[ByteCodes, int],
        Tuple[Tuple[ByteCodes, int]],
        Tuple[Tuple[Tuple[ByteCodes, int]]]
    ]
] = \
{
    # Arithmetic

    'add': (ByteCodes.ADD, 0), # No arguments

    'sub': (ByteCodes.SUB, 0), # No arguments

    'mul': (ByteCodes.MUL, 0), # No arguments

    'div': (ByteCodes.DIV, 0), # No arguments

    'mod': (ByteCodes.MOD, 0), # No arguments

    'inc': (
        (ByteCodes.INC_REG, 0), # Register
    ),
    'inc1': (
        None, # Register
        (ByteCodes.INC_ADDR_IN_REG, 1), # Address in register
        None, # Constant
        (ByteCodes.INC_ADDR_LITERAL, 1), # Address literal
    ),
    'inc2': (
        None, # Register
        (ByteCodes.INC_ADDR_IN_REG, 2), # Address in register
        None, # Constant
        (ByteCodes.INC_ADDR_LITERAL, 2), # Address literal
    ),
    'inc4': (
        None, # Register
        (ByteCodes.INC_ADDR_IN_REG, 4), # Address in register
        None, # Constant
        (ByteCodes.INC_ADDR_LITERAL, 4), # Address literal
    ),
    'inc8': (
        None, # Register
        (ByteCodes.INC_ADDR_IN_REG, 8), # Address in register
        None, # Constant
        (ByteCodes.INC_ADDR_LITERAL, 8), # Address literal
    ),

    'dec': (
        (ByteCodes.DEC_REG, 0), # Register
    ),
    'dec1': (
        None, # Register
        (ByteCodes.DEC_ADDR_IN_REG, 1), # Address in register
        None, # Constant
        (ByteCodes.DEC_ADDR_LITERAL, 1), # Address literal
    ),
    'dec2': (
        None, # Register
        (ByteCodes.DEC_ADDR_IN_REG, 2), # Address in register
        None, # Constant
        (ByteCodes.DEC_ADDR_LITERAL, 2), # Address literal
    ),
    'dec4': (
        None, # Register
        (ByteCodes.DEC_ADDR_IN_REG, 4), # Address in register
        None, # Constant
        (ByteCodes.DEC_ADDR_LITERAL, 4), # Address literal
    ),
    'dec8': (
        None, # Register
        (ByteCodes.DEC_ADDR_IN_REG, 8), # Address in register
        None, # Constant
        (ByteCodes.DEC_ADDR_LITERAL, 8), # Address literal
    ),

    # No operation

    'nop': (ByteCodes.NO_OPERATION, 0), # No arguments

    # Memory

    'mov': (
        # Register
        (
            (ByteCodes.MOVE_REG_REG, 0), # Register
        ),
    ),
    'mov1': (
        # Register
        (
            None, # Register
            (ByteCodes.MOVE_REG_ADDR_IN_REG, 1), # Address in register
            (ByteCodes.MOVE_REG_CONST, 1), # Constant
            (ByteCodes.MOVE_REG_ADDR_LITERAL, 1), # Address literal
        ),
        # Address in register
        (
            (ByteCodes.MOVE_ADDR_IN_REG_REG, 1), # Register
            (ByteCodes.MOVE_ADDR_IN_REG_ADDR_IN_REG, 1), # Address in register
            (ByteCodes.MOVE_ADDR_IN_REG_CONST, 1), # Constant
            (ByteCodes.MOVE_ADDR_IN_REG_ADDR_LITERAL, 1), # Address literal
        ),
        None, # Constant
        # Address literal
        (
            (ByteCodes.MOVE_ADDR_LITERAL_REG, 1), # Register
            (ByteCodes.MOVE_ADDR_LITERAL_ADDR_IN_REG, 1), # Address in register
            (ByteCodes.MOVE_ADDR_LITERAL_CONST, 1), # Constant
            (ByteCodes.MOVE_ADDR_LITERAL_ADDR_LITERAL, 1), # Address literal
        ),
    ),
    'mov2': (
        # Register
        (
            None, # Register
            (ByteCodes.MOVE_REG_ADDR_IN_REG, 2), # Address in register
            (ByteCodes.MOVE_REG_CONST, 2), # Constant
            (ByteCodes.MOVE_REG_ADDR_LITERAL, 2), # Address literal
        ),
        # Address in register
        (
            (ByteCodes.MOVE_ADDR_IN_REG_REG, 2), # Register
            (ByteCodes.MOVE_ADDR_IN_REG_ADDR_IN_REG, 2), # Address in register
            (ByteCodes.MOVE_ADDR_IN_REG_CONST, 2), # Constant
            (ByteCodes.MOVE_ADDR_IN_REG_ADDR_LITERAL, 2), # Address literal
        ),
        None, # Constant
        # Address literal
        (
            (ByteCodes.MOVE_ADDR_LITERAL_REG, 2), # Register
            (ByteCodes.MOVE_ADDR_LITERAL_ADDR_IN_REG, 2), # Address in register
            (ByteCodes.MOVE_ADDR_LITERAL_CONST, 2), # Constant
            (ByteCodes.MOVE_ADDR_LITERAL_ADDR_LITERAL, 2), # Address literal
        ),
    ),
    'mov4': (
        # Register
        (
            None, # Register
            (ByteCodes.MOVE_REG_ADDR_IN_REG, 4), # Address in register
            (ByteCodes.MOVE_REG_CONST, 4), # Constant
            (ByteCodes.MOVE_REG_ADDR_LITERAL, 4), # Address literal
        ),
        # Address in register
        (
            (ByteCodes.MOVE_ADDR_IN_REG_REG, 4), # Register
            (ByteCodes.MOVE_ADDR_IN_REG_ADDR_IN_REG, 4), # Address in register
            (ByteCodes.MOVE_ADDR_IN_REG_CONST, 4), # Constant
            (ByteCodes.MOVE_ADDR_IN_REG_ADDR_LITERAL, 4), # Address literal
        ),
        None, # Constant
        # Address literal
        (
            (ByteCodes.MOVE_ADDR_LITERAL_REG, 4), # Register
            (ByteCodes.MOVE_ADDR_LITERAL_ADDR_IN_REG, 4), # Address in register
            (ByteCodes.MOVE_ADDR_LITERAL_CONST, 4), # Constant
            (ByteCodes.MOVE_ADDR_LITERAL_ADDR_LITERAL, 4), # Address literal
        ),
    ),
    'mov8': (
        # Register
        (
            None, # Register
            (ByteCodes.MOVE_REG_ADDR_IN_REG, 8), # Address in register
            (ByteCodes.MOVE_REG_CONST, 8), # Constant
            (ByteCodes.MOVE_REG_ADDR_LITERAL, 8), # Address literal
        ),
        # Address in register
        (
            (ByteCodes.MOVE_ADDR_IN_REG_REG, 8), # Register
            (ByteCodes.MOVE_ADDR_IN_REG_ADDR_IN_REG, 8), # Address in register
            (ByteCodes.MOVE_ADDR_IN_REG_CONST, 8), # Constant
            (ByteCodes.MOVE_ADDR_IN_REG_ADDR_LITERAL, 8), # Address literal
        ),
        None, # Constant
        # Address literal
        (
            (ByteCodes.MOVE_ADDR_LITERAL_REG, 8), # Register
            (ByteCodes.MOVE_ADDR_LITERAL_ADDR_IN_REG, 8), # Address in register
            (ByteCodes.MOVE_ADDR_LITERAL_CONST, 8), # Constant
            (ByteCodes.MOVE_ADDR_LITERAL_ADDR_LITERAL, 8), # Address literal
        ),
    ),

    'push': (
        (ByteCodes.PUSH_REG, 0), # Register
    ),
    'push1': (
        None, # Register
        (ByteCodes.PUSH_ADDR_IN_REG, 1), # Address in register
        (ByteCodes.PUSH_CONST, 1), # Constant
        (ByteCodes.PUSH_ADDR_LITERAL, 1), # Address literal
    ),
    'push2': (
        None, # Register
        (ByteCodes.PUSH_ADDR_IN_REG, 2), # Address in register
        (ByteCodes.PUSH_CONST, 2), # Constant
        (ByteCodes.PUSH_ADDR_LITERAL, 2), # Address literal
    ),
    'push4': (
        None, # Register
        (ByteCodes.PUSH_ADDR_IN_REG, 4), # Address in register
        (ByteCodes.PUSH_CONST, 4), # Constant
        (ByteCodes.PUSH_ADDR_LITERAL, 4), # Address literal
    ),
    'push8': (
        None, # Register
        (ByteCodes.PUSH_ADDR_IN_REG, 8), # Address in register
        (ByteCodes.PUSH_CONST, 8), # Constant
        (ByteCodes.PUSH_ADDR_LITERAL, 8), # Address literal
    ),

    'pop': (
        (ByteCodes.POP_REG, 0), # Register
    ),
    'pop1': (
        None, # Register
        (ByteCodes.POP_ADDR_IN_REG, 1), # Address in register
        None, # Constant
        (ByteCodes.POP_ADDR_LITERAL, 1), # Address literal
    ),
    'pop2': (
        None, # Register
        (ByteCodes.POP_ADDR_IN_REG, 2), # Address in register
        None, # Constant
        (ByteCodes.POP_ADDR_LITERAL, 2), # Address literal
    ),
    'pop4': (
        None, # Register
        (ByteCodes.POP_ADDR_IN_REG, 4), # Address in register
        None, # Constant
        (ByteCodes.POP_ADDR_LITERAL, 4), # Address literal
    ),
    'pop8': (
        None, # Register
        (ByteCodes.POP_ADDR_IN_REG, 8), # Address in register
        None, # Constant
        (ByteCodes.POP_ADDR_LITERAL, 8), # Address literal
    ),

    # Control flow

    '@': (
        None, None, None, None, # Register, Address in register, Constant, Address literal
        (ByteCodes.LABEL, 0), # Label
    ),

    'jmp': (
        None, None, None, None, # Register, Address in register, Constant, Address literal
        (ByteCodes.JUMP, 0), # Label
    ),
    'cjmp': (
        None, None, None, None, # Register, Address in register, Constant, Address literal
        # Label
        (
            (ByteCodes.JUMP_IF_TRUE_REG, 0), # Register
        ),
    ),
    'njmp': (
        None, None, None, None, # Register, Address in register, Constant, Address literal
        # Label
        (
            (ByteCodes.JUMP_IF_FALSE_REG, 0), # Register
        ),
    ),

    # Comparison

    'cmp': (
        # Register
        (
            (ByteCodes.COMPARE_REG_REG, 0), # Register
        ),
    ),
    'cmp1': (
        # Register
        (
            None, # Register
            None, # Address in register
            (ByteCodes.COMPARE_REG_CONST, 1), # Constant
        ),
        None, # Address in register
        # Constant
        (
            (ByteCodes.COMPARE_CONST_REG, 1), # Register
            None, # Address in register
            (ByteCodes.COMPARE_CONST_CONST, 1), # Constant
        ),
    ),
    'cmp2': (
        # Register
        (
            None, # Register
            None, # Address in register
            (ByteCodes.COMPARE_REG_CONST, 2), # Constant
        ),
        None, # Address in register
        # Constant
        (
            (ByteCodes.COMPARE_CONST_REG, 2), # Register
            None, # Address in register
            (ByteCodes.COMPARE_CONST_CONST, 2), # Constant
        ),
    ),
    'cmp4': (
        # Register
        (
            None, # Register
            None, # Address in register
            (ByteCodes.COMPARE_REG_CONST, 4), # Constant
        ),
        None, # Address in register
        # Constant
        (
            (ByteCodes.COMPARE_CONST_REG, 4), # Register
            None, # Address in register
            (ByteCodes.COMPARE_CONST_CONST, 4),  # Constant
        ),
    ),
    'cmp8': (
        # Register
        (
            None, # Register
            None, # Address in register
            (ByteCodes.COMPARE_REG_CONST, 8), # Constant
        ),
        None, # Address in register
        # Constant
        (
            (ByteCodes.COMPARE_CONST_REG, 8), # Register
            None, # Address in register
            (ByteCodes.COMPARE_CONST_CONST, 8), # Constant
        ),
    ),

    # Interrupts

    'prt': (ByteCodes.PRINT, 0), # No arguments
    'prtstr': (ByteCodes.PRINT_STRING, 0), # No arguments

    'inint': (ByteCodes.INPUT_INT, 0), # No arguments
    'instr': (ByteCodes.INPUT_STRING, 0), # No arguments
    
    'exit': (ByteCodes.EXIT, 0), # No arguments

}

