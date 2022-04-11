from typing import Tuple, Union

from shared.token import TokenType

"""
Conversion table for the byte code to assembly.
Structure:
    (
        "string representation for the instruction",
        (argument type1, argument type2, ...),
        (argument size1, argument size2, ...), # Elements are None if they are sized
        (sized_operand_index_1, sized_operand_index_2, ...), # Or None if no sized operands, empty if size is not important
    )

An empty argument type list means the instruction does not take any arguments.
"""
disassembly_table: Tuple[
    Tuple[
        str,
        Tuple[TokenType],
        Tuple[int],
        Union[Tuple[int], None],
    ],
] = \
(
    ('add', (), (), None),
    ('sub', (), (), None),
    ('mul', (), (), None),
    ('div', (), (), None),
    ('mod', (), (), None),

    
    ('inc', (TokenType.REGISTER), (1,), None),
    ('inc', (TokenType.ADDRESS_IN_REGISTER,), (None,), (0,)),
    ('inc', (TokenType.ADDRESS_LITERAL,), (8,), ()),

    ('dec', (TokenType.REGISTER), (1,), None),
    ('dec', (TokenType.ADDRESS_IN_REGISTER,), (1,), (0,)),
    ('dec', (TokenType.ADDRESS_LITERAL,), (8,), ()),


    ('nop', (), (), None),


    ('mov', (TokenType.REGISTER, TokenType.REGISTER), (1, 1), None),
    ('mov', (TokenType.REGISTER, TokenType.ADDRESS_IN_REGISTER), (1, 1), ()),
    ('mov', (TokenType.REGISTER, TokenType.NUMBER), (1, None), (1,)),
    ('mov', (TokenType.REGISTER, TokenType.ADDRESS_LITERAL), (1, 8), ()),
    ('mov', (TokenType.ADDRESS_IN_REGISTER, TokenType.REGISTER), (1, 1), ()),
    ('mov', (TokenType.ADDRESS_IN_REGISTER, TokenType.ADDRESS_IN_REGISTER), (1, 1), ()),
    ('mov', (TokenType.ADDRESS_IN_REGISTER, TokenType.NUMBER), (1, None), (1,)),
    ('mov', (TokenType.ADDRESS_IN_REGISTER, TokenType.ADDRESS_LITERAL), (1, 8), ()),
    ('mov', (TokenType.ADDRESS_LITERAL, TokenType.REGISTER), (8, 1), ()),
    ('mov', (TokenType.ADDRESS_LITERAL, TokenType.ADDRESS_IN_REGISTER), (8, 1), ()),
    ('mov', (TokenType.ADDRESS_LITERAL, TokenType.NUMBER), (8, None), (1,)),
    ('mov', (TokenType.ADDRESS_LITERAL, TokenType.ADDRESS_LITERAL), (8, 8), ()),


    ('push', (TokenType.REGISTER,), (1,), None),
    ('push', (TokenType.ADDRESS_IN_REGISTER,), (1,), ()),
    ('push', (TokenType.NUMBER,), (None,), (0,)),
    ('push', (TokenType.ADDRESS_LITERAL,), (8,), ()),

    ('pop', (TokenType.REGISTER,), (1,), None),
    ('pop', (TokenType.ADDRESS_IN_REGISTER,), (1,), ()),
    ('pop', (TokenType.ADDRESS_LITERAL,), (8,), ()),


    ('@', (TokenType.LABEL), (8,), None), # Doesn't get used, but it's here for completeness.


    ('jmp', (TokenType.NUMBER,), (8,), None),
    ('cjmp', (TokenType.NUMBER, TokenType.REGISTER), (8, 1), None),
    ('njmp', (TokenType.NUMBER, TokenType.REGISTER,), (8, 1), None),


    ('cmp', (TokenType.REGISTER, TokenType.REGISTER), (1, 1), None),
    ('cmp', (TokenType.REGISTER, TokenType.NUMBER), (1, None), (1,)),
    ('cmp', (TokenType.NUMBER, TokenType.REGISTER), (None, 1), (0,)),
    ('cmp', (TokenType.NUMBER, TokenType.NUMBER), (None, None), (0, 1)),


    ('prt', (), (), None),
    ('prtstr', (None,), (None), None), # TODO: add string literal

    ('inint', (), (), None),
    ('instr', (), (), None),

    ('exit', (), (), None),


)

