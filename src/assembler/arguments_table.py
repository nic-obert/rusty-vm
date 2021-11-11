from typing import Dict, List

from src.shared.byte_code import ByteCodes


arguments_table : Dict[str, List] = \
{
    'add': [
        # Register
        [
            ByteCodes.ADD_REG_REG,
            ByteCodes.ADD_REG_ADDR,
            ByteCodes.ADD_ADDR_CONST
        ],
        # At
        [
            ByteCodes.ADD_ADDR_REG,
            ByteCodes.ADD_ADDR_ADDR,
            ByteCodes.ADD_ADDR_CONST
        ],
    ],
    'sub': [
        # Register
        
    ]
}
