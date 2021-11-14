from typing import Dict, Tuple, Union

from shared.byte_code import ByteCodes


arguments_table : Dict[str, Tuple[Union[Tuple[Union[Tuple, ByteCodes]], ByteCodes]]] = \
{
    # Arithmetic
    'add': (
        (
            ByteCodes.ADD,
        )
    ),
    'sub': (
        (
            ByteCodes.SUB,
        )
    ),
    'mul': (
        (
            ByteCodes.MUL,
        )
    ),
    'div': (
        (
            ByteCodes.DIV,
        )
    ),
    'mod': (
        (
            ByteCodes.MOD,
        )
    ),

    'inc': (
        # Register
        ByteCodes.INC_REG
    ),
    'inc1': (
        None, # Register
        # Address
        ByteCodes.INC1_ADDR
    ),
    'inc2': (
        None, # Register
        # Address
        ByteCodes.INC2_ADDR
    ),
    'inc4': (
        None, # Register
        # Address
        ByteCodes.INC4_ADDR
    ),
    'inc8': (
        None, # Register
        # Address
        ByteCodes.INC8_ADDR
    ),

    'dec': (
        # Register
        ByteCodes.DEC_REG
    ),
    'dec1': (
        None, # Register
        # Address
        ByteCodes.DEC1_ADDR
    ),
    'dec2': (
        None, # Register
        # Address
        ByteCodes.DEC2_ADDR
    ),
    'dec4': (
        None, # Register
        # Address
        ByteCodes.DEC4_ADDR
    ),
    'dec8': (
        None, # Register
        # Address
        ByteCodes.DEC8_ADDR
    ),

    # No operation

    'nop': ByteCodes.NO_OPERATION,

    # Memory

    'ld1': (
        # Register
        (
            ByteCodes.LOAD1_REG_REG,
            ByteCodes.LOAD1_REG_ADDR,
            ByteCodes.LOAD1_REG_CONST
        )
    ),
    'ld2': (
        # Register
        (
            ByteCodes.LOAD2_REG_REG,
            ByteCodes.LOAD2_REG_ADDR,
            ByteCodes.LOAD2_REG_CONST
        )
    ),
    'ld4': (
        # Register
        (
            ByteCodes.LOAD4_REG_REG,
            ByteCodes.LOAD4_REG_ADDR,
            ByteCodes.LOAD4_REG_CONST
        )
    ),
    'ld8': (
        # Register
        (
            ByteCodes.LOAD8_REG_REG,
            ByteCodes.LOAD8_REG_ADDR,
            ByteCodes.LOAD8_REG_CONST
        )
    ),

    'mov1': (
        # Register
        (
            ByteCodes.MOVE1_REG_REG,
            ByteCodes.MOVE1_REG_ADDR,
            ByteCodes.MOVE1_REG_CONST
        ),
        # Address
        (
            ByteCodes.MOVE1_ADDR_REG,
            ByteCodes.MOVE1_ADDR_ADDR,
            ByteCodes.MOVE1_ADDR_CONST
        ),
    ),
    'mov2': (
        # Register
        (
            ByteCodes.MOVE2_REG_REG,
            ByteCodes.MOVE2_REG_ADDR,
            ByteCodes.MOVE2_REG_CONST
        ),
        # Address
        (
            ByteCodes.MOVE2_ADDR_REG,
            ByteCodes.MOVE2_ADDR_ADDR,
            ByteCodes.MOVE2_ADDR_CONST
        ),
    ),
    'mov4': (
        # Register
        (
            ByteCodes.MOVE4_REG_REG,
            ByteCodes.MOVE4_REG_ADDR,
            ByteCodes.MOVE4_REG_CONST
        ),
        # Address
        (
            ByteCodes.MOVE4_ADDR_REG,
            ByteCodes.MOVE4_ADDR_ADDR,
            ByteCodes.MOVE4_ADDR_CONST
        ),
    ),
    'mov8': (
        # Register
        (
            ByteCodes.MOVE8_REG_REG,
            ByteCodes.MOVE8_REG_ADDR,
            ByteCodes.MOVE8_REG_CONST
        ),
        # Address
        (
            ByteCodes.MOVE8_ADDR_REG,
            ByteCodes.MOVE8_ADDR_ADDR,
            ByteCodes.MOVE8_ADDR_CONST
        ),
    ),
    
    'st1': (
        # Register
        (
            ByteCodes.STORE1_REG_REG
        ),
        # Address
        (
            ByteCodes.STORE1_ADDR_REG
        ),
        # Constant
        (
            ByteCodes.STORE1_CONST_REG
        )
    ),
    'st2': (
        # Register
        (
            ByteCodes.STORE2_REG_REG
        ),
        # Address
        (
            ByteCodes.STORE2_ADDR_REG
        ),
        # Constant
        (
            ByteCodes.STORE2_CONST_REG
        )
    ),
    'st4': (
        # Register
        (
            ByteCodes.STORE4_REG_REG
        ),
        # Address
        (
            ByteCodes.STORE4_ADDR_REG
        ),
        # Constant
        (
            ByteCodes.STORE4_CONST_REG
        )
    ),
    'st8': (
        # Register
        (
            ByteCodes.STORE8_REG_REG
        ),
        # Address
        (
            ByteCodes.STORE8_ADDR_REG
        ),
        # Constant
        (
            ByteCodes.STORE8_CONST_REG
        )
    ),

    # Control flow

    '@': (
        ByteCodes.LABEL
    ),

    'jmp': (
        None, None, None, # Register, Address, Constant
        # Label
        ByteCodes.JUMP
    ),
    'cjmp': (
        None, None, None, # Register, Address, Constant
        # Label
        (
            ByteCodes.JUMP_IF_TRUE_REG,
        ),
    ),
    'njmp': (
        None, None, None, # Register, Address, Constant
        # Label
        (
            ByteCodes.JUMP_IF_FALSE_REG,
        ),
    ),

    # Comparison

    'cmp': (
        # Register
        (
            ByteCodes.COMPARE_REG_REG,
            None, # Address
            ByteCodes.COMPARE_REG_CONST
        ),
        None, # Address
        # Constant
        (
            ByteCodes.COMPARE_CONST_REG,
            None, # Address
            ByteCodes.COMPARE_CONST_CONST
        ),
    ),
    

}

