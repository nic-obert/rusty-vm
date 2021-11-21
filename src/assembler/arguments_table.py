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
        ByteCodes.INC1_ADDR_IN_REG, # Address in register
        None, # Constant
        ByteCodes.INC1_ADDR_LITERAL, # Address literal
    ),
    'inc2': (
        None, # Register
        ByteCodes.INC2_ADDR_IN_REG, # Address in register
        None, # Constant
        ByteCodes.INC2_ADDR_LITERAL, # Address literal
    ),
    'inc4': (
        None, # Register
        ByteCodes.INC4_ADDR_IN_REG, # Address in register
        None, # Constant
        ByteCodes.INC4_ADDR_LITERAL, # Address literal
    ),
    'inc8': (
        None, # Register
        ByteCodes.INC8_ADDR_IN_REG, # Address in register
        None, # Constant
        ByteCodes.INC8_ADDR_LITERAL, # Address literal
    ),

    'dec': (
        ByteCodes.DEC_REG, # Register
    ),
    'dec1': (
        None, # Register
        ByteCodes.DEC1_ADDR_IN_REG, # Address in register
        None, # Constant
        ByteCodes.DEC1_ADDR_LITERAL, # Address literal
    ),
    'dec2': (
        None, # Register
        ByteCodes.DEC2_ADDR_IN_REG, # Address in register
        None, # Constant
        ByteCodes.DEC2_ADDR_LITERAL, # Address literal
    ),
    'dec4': (
        None, # Register
        ByteCodes.DEC4_ADDR_IN_REG, # Address in register
        None, # Constant
        ByteCodes.DEC4_ADDR_LITERAL, # Address literal
    ),
    'dec8': (
        None, # Register
        ByteCodes.DEC8_ADDR_IN_REG, # Address in register
        None, # Constant
        ByteCodes.DEC8_ADDR_LITERAL, # Address literal
    ),

    # No operation

    'nop': ByteCodes.NO_OPERATION, # No arguments

    # Memory

    'ld': (
        # Register
        (
            ByteCodes.LOAD_REG_REG, # Register
        ),
    ),
    'ld1': (
        # Register
        (
            None, # Register
            ByteCodes.LOAD1_REG_ADDR_IN_REG, # Address in register
            ByteCodes.LOAD1_REG_CONST, # Constant
            ByteCodes.LOAD1_REG_ADDR_LITERAL, # Address literal
        ),
    ),
    'ld2': (
        # Register
        (
            None, # Register
            ByteCodes.LOAD2_REG_ADDR_IN_REG, # Address in register
            ByteCodes.LOAD2_REG_CONST, # Constant
            ByteCodes.LOAD2_REG_ADDR_LITERAL, # Address literal
        ),
    ),
    'ld4': (
        # Register
        (
            None, # Register
            ByteCodes.LOAD4_REG_ADDR_IN_REG, # Address in register
            ByteCodes.LOAD4_REG_CONST, # Constant
            ByteCodes.LOAD4_REG_ADDR_LITERAL, # Address literal
        ),
    ),
    'ld8': (
        # Register
        (
            None, # Register
            ByteCodes.LOAD8_REG_ADDR_IN_REG, # Address in register
            ByteCodes.LOAD8_REG_CONST, # Constant
            ByteCodes.LOAD8_REG_ADDR_LITERAL, # Address literal
        ),
    ),

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
            ByteCodes.MOVE1_REG_ADDR_IN_REG, # Address in register
            ByteCodes.MOVE1_REG_CONST, # Constant
            ByteCodes.MOVE1_REG_ADDR_LITERAL, # Address literal
        ),
        # Address in register
        (
            ByteCodes.MOVE1_ADDR_IN_REG_REG, # Register
            ByteCodes.MOVE1_ADDR_IN_REG_ADDR_IN_REG, # Address in register
            ByteCodes.MOVE1_ADDR_IN_REG_CONST, # Constant
            ByteCodes.MOVE1_ADDR_IN_REG_ADDR_LITERAL, # Address literal
        ),
        None, # Constant
        # Address literal
        (
            ByteCodes.MOVE1_ADDR_LITERAL_REG, # Register
            ByteCodes.MOVE1_ADDR_LITERAL_ADDR_IN_REG, # Address in register
            ByteCodes.MOVE1_ADDR_LITERAL_CONST, # Constant
            ByteCodes.MOVE1_ADDR_LITERAL_ADDR_LITERAL, # Address literal
        ),
    ),
    'mov2': (
        # Register
        (
            None, # Register
            ByteCodes.MOVE2_REG_ADDR_IN_REG, # Address in register
            ByteCodes.MOVE2_REG_CONST, # Constant
            ByteCodes.MOVE2_REG_ADDR_LITERAL, # Address literal
        ),
        # Address in register
        (
            ByteCodes.MOVE2_ADDR_IN_REG_REG, # Register
            ByteCodes.MOVE2_ADDR_IN_REG_ADDR_IN_REG, # Address in register
            ByteCodes.MOVE2_ADDR_IN_REG_CONST, # Constant
            ByteCodes.MOVE2_ADDR_IN_REG_ADDR_LITERAL, # Address literal
        ),
        None, # Constant
        # Address literal
        (
            ByteCodes.MOVE2_ADDR_LITERAL_REG, # Register
            ByteCodes.MOVE2_ADDR_LITERAL_ADDR_IN_REG, # Address in register
            ByteCodes.MOVE2_ADDR_LITERAL_CONST, # Constant
            ByteCodes.MOVE2_ADDR_LITERAL_ADDR_LITERAL, # Address literal
        ),
    ),
    'mov4': (
        # Register
        (
            None, # Register
            ByteCodes.MOVE4_REG_ADDR_IN_REG, # Address in register
            ByteCodes.MOVE4_REG_CONST, # Constant
            ByteCodes.MOVE4_REG_ADDR_LITERAL, # Address literal
        ),
        # Address in register
        (
            ByteCodes.MOVE4_ADDR_IN_REG_REG, # Register
            ByteCodes.MOVE4_ADDR_IN_REG_ADDR_IN_REG, # Address in register
            ByteCodes.MOVE4_ADDR_IN_REG_CONST, # Constant
            ByteCodes.MOVE4_ADDR_IN_REG_ADDR_LITERAL, # Address literal
        ),
        None, # Constant
        # Address literal
        (
            ByteCodes.MOVE4_ADDR_LITERAL_REG, # Register
            ByteCodes.MOVE4_ADDR_LITERAL_ADDR_IN_REG, # Address in register
            ByteCodes.MOVE4_ADDR_LITERAL_CONST, # Constant
            ByteCodes.MOVE4_ADDR_LITERAL_ADDR_LITERAL, # Address literal
        ),
    ),
    'mov8': (
        # Register
        (
            None, # Register
            ByteCodes.MOVE8_REG_ADDR_IN_REG, # Address in register
            ByteCodes.MOVE8_REG_CONST, # Constant
            ByteCodes.MOVE8_REG_ADDR_LITERAL, # Address literal
        ),
        # Address in register
        (
            ByteCodes.MOVE8_ADDR_IN_REG_REG, # Register
            ByteCodes.MOVE8_ADDR_IN_REG_ADDR_IN_REG, # Address in register
            ByteCodes.MOVE8_ADDR_IN_REG_CONST, # Constant
            ByteCodes.MOVE8_ADDR_IN_REG_ADDR_LITERAL, # Address literal
        ),
        None, # Constant
        # Address literal
        (
            ByteCodes.MOVE8_ADDR_LITERAL_REG, # Register
            ByteCodes.MOVE8_ADDR_LITERAL_ADDR_IN_REG, # Address in register
            ByteCodes.MOVE8_ADDR_LITERAL_CONST, # Constant
            ByteCodes.MOVE8_ADDR_LITERAL_ADDR_LITERAL, # Address literal
        ),
    ),

    
    'st1': (
        None, # Register
        # Address in register
        (
            ByteCodes.STORE1_ADDR_IN_REG_REG, # Register
        ),
        None, # Constant
        # Address literal
        (
            ByteCodes.STORE1_ADDR_LITERAL_REG, # Register
        ),
    ),
    'st2': (
        None, # Register
        # Address in register
        (
            ByteCodes.STORE2_ADDR_IN_REG_REG, # Register
        ),
        None, # Constant
        # Address literal
        (
            ByteCodes.STORE2_ADDR_LITERAL_REG, # Register
        ),
    ),
    'st4': (
        None, # Register
        # Address in register
        (
            ByteCodes.STORE4_ADDR_IN_REG_REG, # Register
        ),
        None, # Constant
        # Address literal
        (
            ByteCodes.STORE4_ADDR_LITERAL_REG, # Register
        ),
    ),
    'st8': (
        None, # Register
        # Address in register
        (
            ByteCodes.STORE8_ADDR_IN_REG_REG, # Register
        ),
        None, # Constant
        # Address literal
        (
            ByteCodes.STORE8_ADDR_LITERAL_REG, # Register
        ),
    ),

    'push': (
        ByteCodes.PUSH_REG, # Register
    ),
    'push1': (
        None, # Register
        ByteCodes.PUSH1_ADDR_IN_REG, # Address in register
        ByteCodes.PUSH1_CONST, # Constant
        ByteCodes.PUSH1_ADDR_LITERAL, # Address literal
    ),
    'push2': (
        None, # Register
        ByteCodes.PUSH2_ADDR_IN_REG, # Address in register
        ByteCodes.PUSH2_CONST, # Constant
        ByteCodes.PUSH2_ADDR_LITERAL, # Address literal
    ),
    'push4': (
        None, # Register
        ByteCodes.PUSH4_ADDR_IN_REG, # Address in register
        ByteCodes.PUSH4_CONST, # Constant
        ByteCodes.PUSH4_ADDR_LITERAL, # Address literal
    ),
    'push8': (
        None, # Register
        ByteCodes.PUSH8_ADDR_IN_REG, # Address in register
        ByteCodes.PUSH8_CONST, # Constant
        ByteCodes.PUSH8_ADDR_LITERAL, # Address literal
    ),

    'pop': (
        ByteCodes.POP_REG, # Register
    ),
    'pop1': (
        None, # Register
        ByteCodes.POP1_ADDR_IN_REG, # Address in register
        ByteCodes.POP1_CONST, # Constant
        ByteCodes.POP1_ADDR_LITERAL, # Address literal
    ),
    'pop2': (
        None, # Register
        ByteCodes.POP2_ADDR_IN_REG, # Address in register
        ByteCodes.POP2_CONST, # Constant
        ByteCodes.POP2_ADDR_LITERAL, # Address literal
    ),
    'pop4': (
        None, # Register
        ByteCodes.POP4_ADDR_IN_REG, # Address in register
        ByteCodes.POP4_CONST, # Constant
        ByteCodes.POP4_ADDR_LITERAL, # Address literal
    ),
    'pop8': (
        None, # Register
        ByteCodes.POP8_ADDR_IN_REG, # Address in register
        ByteCodes.POP8_CONST, # Constant
        ByteCodes.POP8_ADDR_LITERAL, # Address literal
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
            ByteCodes.COMPARE1_REG_CONST, # Constant
        ),
        None, # Address in register
        # Constant
        (
            ByteCodes.COMPARE1_CONST_REG, # Register
            None, # Address in register
            ByteCodes.COMPARE1_CONST_CONST, # Constant
        ),
    ),
    'cmp2': (
        # Register
        (
            None, # Register
            None, # Address in register
            ByteCodes.COMPARE2_REG_CONST, # Constant
        ),
        None, # Address in register
        # Constant
        (
            ByteCodes.COMPARE2_CONST_REG, # Register
            None, # Address in register
            ByteCodes.COMPARE2_CONST_CONST, # Constant
        ),
    ),
    'cmp4': (
        # Register
        (
            None, # Register
            None, # Address in register
            ByteCodes.COMPARE4_REG_CONST, # Constant
        ),
        None, # Address in register
        # Constant
        (
            ByteCodes.COMPARE4_CONST_REG, # Register
            None, # Address in register
            ByteCodes.COMPARE4_CONST_CONST, # Constant
        ),
    ),
    'cmp8': (
        # Register
        (
            None, # Register
            None, # Address in register
            ByteCodes.COMPARE8_REG_CONST, # Constant
        ),
        None, # Address in register
        # Constant
        (
            ByteCodes.COMPARE8_CONST_REG, # Register
            None, # Address in register
            ByteCodes.COMPARE8_CONST_CONST, # Constant
        ),
    ),

    # Interrupts

    'prt': ByteCodes.PRINT, # No arguments
    'prtstr': ByteCodes.PRINT_STRING, # No arguments
    
    'exit': ByteCodes.EXIT, # No arguments

}

