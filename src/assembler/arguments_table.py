from typing import Dict, List, Union

from src.shared.byte_code import ByteCodes


arguments_table : Dict[str, List[Union[List[Union[List, ByteCodes]], ByteCodes]]] = \
{
    # Arithmetic
    'add': [
        [
            ByteCodes.ADD,
        ]
    ],
    'sub': [
        [
            ByteCodes.SUB,
        ]
    ],
    'mul': [
        [
            ByteCodes.MUL,
        ]
    ],
    'div': [
        [
            ByteCodes.DIV,
        ]
    ],
    'mod': [
        [
            ByteCodes.MOD,
        ]
    ],

    # No operation

    'nop': ByteCodes.NO_OPERATION,

    # Memory

    'ld1': [
        # Register
        [
            ByteCodes.LOAD1_REG_REG,
            ByteCodes.LOAD1_REG_ADDR,
            ByteCodes.LOAD1_REG_CONST
        ]
    ],
    'ld2': [
        # Register
        [
            ByteCodes.LOAD2_REG_REG,
            ByteCodes.LOAD2_REG_ADDR,
            ByteCodes.LOAD2_REG_CONST
        ]
    ],
    'ld4': [
        # Register
        [
            ByteCodes.LOAD4_REG_REG,
            ByteCodes.LOAD4_REG_ADDR,
            ByteCodes.LOAD4_REG_CONST
        ]
    ],
    'ld8': [
        # Register
        [
            ByteCodes.LOAD8_REG_REG,
            ByteCodes.LOAD8_REG_ADDR,
            ByteCodes.LOAD8_REG_CONST
        ]
    ],

    'mov1': [
        # Register
        [
            ByteCodes.MOVE1_REG_REG,
            ByteCodes.MOVE1_REG_ADDR,
            ByteCodes.MOVE1_REG_CONST
        ],
        # Address
        [
            ByteCodes.MOVE1_ADDR_REG,
            ByteCodes.MOVE1_ADDR_ADDR,
            ByteCodes.MOVE1_ADDR_CONST
        ],
    ],
    'mov2': [
        # Register
        [
            ByteCodes.MOVE2_REG_REG,
            ByteCodes.MOVE2_REG_ADDR,
            ByteCodes.MOVE2_REG_CONST
        ],
        # Address
        [
            ByteCodes.MOVE2_ADDR_REG,
            ByteCodes.MOVE2_ADDR_ADDR,
            ByteCodes.MOVE2_ADDR_CONST
        ],
    ],
    'mov4': [
        # Register
        [
            ByteCodes.MOVE4_REG_REG,
            ByteCodes.MOVE4_REG_ADDR,
            ByteCodes.MOVE4_REG_CONST
        ],
        # Address
        [
            ByteCodes.MOVE4_ADDR_REG,
            ByteCodes.MOVE4_ADDR_ADDR,
            ByteCodes.MOVE4_ADDR_CONST
        ],
    ],
    'mov8': [
        # Register
        [
            ByteCodes.MOVE8_REG_REG,
            ByteCodes.MOVE8_REG_ADDR,
            ByteCodes.MOVE8_REG_CONST
        ],
        # Address
        [
            ByteCodes.MOVE8_ADDR_REG,
            ByteCodes.MOVE8_ADDR_ADDR,
            ByteCodes.MOVE8_ADDR_CONST
        ],
    ],


  

}

