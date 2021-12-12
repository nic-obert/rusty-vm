from typing import Dict, Tuple, Union

from shared.byte_code import ByteCodes


arguments_table : Dict[str, Tuple[Union[Tuple[Union[Tuple, ByteCodes]], ByteCodes]]] = \
{
    # Arithmetic

    'add': ByteCodes.ADD, # No arguments

    'sub': ByteCodes.SUB, # No arguments

    'mul': ByteCodes.MUL, # No arguments

    'div': ByteCodes.DIV, # No arguments

    'mod': ByteCodes.MOD, # No arguments

    'inc': (
        ByteCodes.INC_REG, # Register
    ),
    'inc1': (
        None, # Register
        ByteCodes.INC_ADDR_IN_REG, # Address in register
        None, # Constant
        ByteCodes.INC_ADDR_LITERAL, # Address literal
    ),
    'inc2': (
        None, # Register
        ByteCodes.INC_ADDR_IN_REG, # Address in register
        None, # Constant
        ByteCodes.INC_ADDR_LITERAL, # Address literal
    ),
    'inc4': (
        None, # Register
        ByteCodes.INC_ADDR_IN_REG, # Address in register
        None, # Constant
        ByteCodes.INC_ADDR_LITERAL, # Address literal
    ),
    'inc8': (
        None, # Register
        ByteCodes.INC_ADDR_IN_REG, # Address in register
        None, # Constant
        ByteCodes.INC_ADDR_LITERAL, # Address literal
    ),

    'dec': (
        ByteCodes.DEC_REG, # Register
    ),
    'dec1': (
        None, # Register
        ByteCodes.DEC_ADDR_IN_REG, # Address in register
        None, # Constant
        ByteCodes.DEC_ADDR_LITERAL, # Address literal
    ),
    'dec2': (
        None, # Register
        ByteCodes.DEC_ADDR_IN_REG, # Address in register
        None, # Constant
        ByteCodes.DEC_ADDR_LITERAL, # Address literal
    ),
    'dec4': (
        None, # Register
        ByteCodes.DEC_ADDR_IN_REG, # Address in register
        None, # Constant
        ByteCodes.DEC_ADDR_LITERAL, # Address literal
    ),
    'dec8': (
        None, # Register
        ByteCodes.DEC_ADDR_IN_REG, # Address in register
        None, # Constant
        ByteCodes.DEC_ADDR_LITERAL, # Address literal
    ),

    # No operation

    'nop': ByteCodes.NO_OPERATION, # No arguments

    # Memory

    'mov': (
        # Register
        (
            ByteCodes.MOVE_REG_REG, # Register
        ),
    ),
    'mov1': (
        # Register
        (
            None, # Register
            ByteCodes.MOVE_REG_ADDR_IN_REG, # Address in register
            ByteCodes.MOVE_REG_CONST, # Constant
            ByteCodes.MOVE_REG_ADDR_LITERAL, # Address literal
        ),
        # Address in register
        (
            ByteCodes.MOVE_ADDR_IN_REG_REG, # Register
            ByteCodes.MOVE_ADDR_IN_REG_ADDR_IN_REG, # Address in register
            ByteCodes.MOVE_ADDR_IN_REG_CONST, # Constant
            ByteCodes.MOVE_ADDR_IN_REG_ADDR_LITERAL, # Address literal
        ),
        None, # Constant
        # Address literal
        (
            ByteCodes.MOVE_ADDR_LITERAL_REG, # Register
            ByteCodes.MOVE_ADDR_LITERAL_ADDR_IN_REG, # Address in register
            ByteCodes.MOVE_ADDR_LITERAL_CONST, # Constant
            ByteCodes.MOVE_ADDR_LITERAL_ADDR_LITERAL, # Address literal
        ),
    ),
    'mov2': (
        # Register
        (
            None, # Register
            ByteCodes.MOVE_REG_ADDR_IN_REG, # Address in register
            ByteCodes.MOVE_REG_CONST, # Constant
            ByteCodes.MOVE_REG_ADDR_LITERAL, # Address literal
        ),
        # Address in register
        (
            ByteCodes.MOVE_ADDR_IN_REG_REG, # Register
            ByteCodes.MOVE_ADDR_IN_REG_ADDR_IN_REG, # Address in register
            ByteCodes.MOVE_ADDR_IN_REG_CONST, # Constant
            ByteCodes.MOVE_ADDR_IN_REG_ADDR_LITERAL, # Address literal
        ),
        None, # Constant
        # Address literal
        (
            ByteCodes.MOVE_ADDR_LITERAL_REG, # Register
            ByteCodes.MOVE_ADDR_LITERAL_ADDR_IN_REG, # Address in register
            ByteCodes.MOVE_ADDR_LITERAL_CONST, # Constant
            ByteCodes.MOVE_ADDR_LITERAL_ADDR_LITERAL, # Address literal
        ),
    ),
    'mov4': (
        # Register
        (
            None, # Register
            ByteCodes.MOVE_REG_ADDR_IN_REG, # Address in register
            ByteCodes.MOVE_REG_CONST, # Constant
            ByteCodes.MOVE_REG_ADDR_LITERAL, # Address literal
        ),
        # Address in register
        (
            ByteCodes.MOVE_ADDR_IN_REG_REG, # Register
            ByteCodes.MOVE_ADDR_IN_REG_ADDR_IN_REG, # Address in register
            ByteCodes.MOVE_ADDR_IN_REG_CONST, # Constant
            ByteCodes.MOVE_ADDR_IN_REG_ADDR_LITERAL, # Address literal
        ),
        None, # Constant
        # Address literal
        (
            ByteCodes.MOVE_ADDR_LITERAL_REG, # Register
            ByteCodes.MOVE_ADDR_LITERAL_ADDR_IN_REG, # Address in register
            ByteCodes.MOVE_ADDR_LITERAL_CONST, # Constant
            ByteCodes.MOVE_ADDR_LITERAL_ADDR_LITERAL, # Address literal
        ),
    ),
    'mov8': (
        # Register
        (
            None, # Register
            ByteCodes.MOVE_REG_ADDR_IN_REG, # Address in register
            ByteCodes.MOVE_REG_CONST, # Constant
            ByteCodes.MOVE_REG_ADDR_LITERAL, # Address literal
        ),
        # Address in register
        (
            ByteCodes.MOVE_ADDR_IN_REG_REG, # Register
            ByteCodes.MOVE_ADDR_IN_REG_ADDR_IN_REG, # Address in register
            ByteCodes.MOVE_ADDR_IN_REG_CONST, # Constant
            ByteCodes.MOVE_ADDR_IN_REG_ADDR_LITERAL, # Address literal
        ),
        None, # Constant
        # Address literal
        (
            ByteCodes.MOVE_ADDR_LITERAL_REG, # Register
            ByteCodes.MOVE_ADDR_LITERAL_ADDR_IN_REG, # Address in register
            ByteCodes.MOVE_ADDR_LITERAL_CONST, # Constant
            ByteCodes.MOVE_ADDR_LITERAL_ADDR_LITERAL, # Address literal
        ),
    ),

    'push': (
        ByteCodes.PUSH_REG, # Register
    ),
    'push1': (
        None, # Register
        ByteCodes.PUSH_ADDR_IN_REG, # Address in register
        ByteCodes.PUSH_CONST, # Constant
        ByteCodes.PUSH_ADDR_LITERAL, # Address literal
    ),
    'push2': (
        None, # Register
        ByteCodes.PUSH_ADDR_IN_REG, # Address in register
        ByteCodes.PUSH_CONST, # Constant
        ByteCodes.PUSH_ADDR_LITERAL, # Address literal
    ),
    'push4': (
        None, # Register
        ByteCodes.PUSH_ADDR_IN_REG, # Address in register
        ByteCodes.PUSH_CONST, # Constant
        ByteCodes.PUSH_ADDR_LITERAL, # Address literal
    ),
    'push8': (
        None, # Register
        ByteCodes.PUSH_ADDR_IN_REG, # Address in register
        ByteCodes.PUSH_CONST, # Constant
        ByteCodes.PUSH_ADDR_LITERAL, # Address literal
    ),

    'pop': (
        ByteCodes.POP_REG, # Register
    ),
    'pop1': (
        None, # Register
        ByteCodes.POP_ADDR_IN_REG, # Address in register
        None, # Constant
        ByteCodes.POP_ADDR_LITERAL, # Address literal
    ),
    'pop2': (
        None, # Register
        ByteCodes.POP_ADDR_IN_REG, # Address in register
        None, # Constant
        ByteCodes.POP_ADDR_LITERAL, # Address literal
    ),
    'pop4': (
        None, # Register
        ByteCodes.POP_ADDR_IN_REG, # Address in register
        None, # Constant
        ByteCodes.POP_ADDR_LITERAL, # Address literal
    ),
    'pop8': (
        None, # Register
        ByteCodes.POP_ADDR_IN_REG, # Address in register
        None, # Constant
        ByteCodes.POP_ADDR_LITERAL, # Address literal
    ),

    # Control flow

    '@': (
        None, None, None, None, # Register, Address in register, Constant, Address literal
        ByteCodes.LABEL, # Label
    ),

    'jmp': (
        None, None, None, None, # Register, Address in register, Constant, Address literal
        ByteCodes.JUMP, # Label
    ),
    'cjmp': (
        None, None, None, None, # Register, Address in register, Constant, Address literal
        # Label
        (
            ByteCodes.JUMP_IF_TRUE_REG, # Register
        ),
    ),
    'njmp': (
        None, None, None, None, # Register, Address in register, Constant, Address literal
        # Label
        (
            ByteCodes.JUMP_IF_FALSE_REG, # Register
        ),
    ),

    # Comparison

    'cmp': (
        # Register
        (
            ByteCodes.COMPARE_REG_REG, # Register
        ),
    ),
    'cmp1': (
        # Register
        (
            None, # Register
            None, # Address in register
            ByteCodes.COMPARE_REG_CONST, # Constant
        ),
        None, # Address in register
        # Constant
        (
            ByteCodes.COMPARE_CONST_REG, # Register
            None, # Address in register
            ByteCodes.COMPARE_CONST_CONST, # Constant
        ),
    ),
    'cmp2': (
        # Register
        (
            None, # Register
            None, # Address in register
            ByteCodes.COMPARE_REG_CONST, # Constant
        ),
        None, # Address in register
        # Constant
        (
            ByteCodes.COMPARE_CONST_REG, # Register
            None, # Address in register
            ByteCodes.COMPARE_CONST_CONST, # Constant
        ),
    ),
    'cmp4': (
        # Register
        (
            None, # Register
            None, # Address in register
            ByteCodes.COMPARE_REG_CONST, # Constant
        ),
        None, # Address in register
        # Constant
        (
            ByteCodes.COMPARE_CONST_REG, # Register
            None, # Address in register
            ByteCodes.COMPARE_CONST_CONST, # Constant
        ),
    ),
    'cmp8': (
        # Register
        (
            None, # Register
            None, # Address in register
            ByteCodes.COMPARE_REG_CONST, # Constant
        ),
        None, # Address in register
        # Constant
        (
            ByteCodes.COMPARE_CONST_REG, # Register
            None, # Address in register
            ByteCodes.COMPARE_CONST_CONST, # Constant
        ),
    ),

    # Interrupts

    'prt': ByteCodes.PRINT, # No arguments
    'prtstr': ByteCodes.PRINT_STRING, # No arguments

    'inint': ByteCodes.INPUT_INT, # No arguments
    'instr': ByteCodes.INPUT_STRING, # No arguments
    
    'exit': ByteCodes.EXIT, # No arguments

}

