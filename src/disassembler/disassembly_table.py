from typing import Tuple

from shared.token import TokenType

"""
Conversion table for the byte code to assembly.
Structure:
    (
        "string representation for the instruction",
        (argument type1, argument type2, ...),
        (argument size1, argument size2, ...),
    )

An empty argument type list means the instruction does not take any arguments.
"""
disassembly_table: Tuple[Tuple[str, Tuple[TokenType], Tuple[int], bool]] = \
(
    ('add', (), (), False),
    ('sub', (), (), False),
    ('mul', (), (), False),
    ('div', (), (), False),
    ('mod', (), (), False),

    
    ('inc', (TokenType.REGISTER), (1,), False),
    ('inc', (TokenType.ADDRESS_IN_REGISTER,), (1,), True),
    ('inc', (TokenType.ADDRESS_LITERAL,), (8,), True),

    ('dec', (TokenType.REGISTER), (1,), False),
    ('dec', (TokenType.ADDRESS_IN_REGISTER,), (1,), True),
    ('dec', (TokenType.ADDRESS_LITERAL,), (8,), True),


    ('nop', (), (), False),


    ('mov', (TokenType.REGISTER, TokenType.REGISTER), (1, 1), False),
    ('mov', (TokenType.REGISTER, TokenType.ADDRESS_IN_REGISTER), (1, 1), True),
    ('mov', (TokenType.REGISTER, TokenType.NUMBER), (1, 1), True),
    ('mov', (TokenType.REGISTER, TokenType.ADDRESS_LITERAL), (1, 8), True),
    ('mov', (TokenType.ADDRESS_IN_REGISTER, TokenType.REGISTER), (1, 1), True),
    ('mov', (TokenType.ADDRESS_IN_REGISTER, TokenType.ADDRESS_IN_REGISTER), (1, 1), True),
    ('mov', (TokenType.ADDRESS_IN_REGISTER, TokenType.NUMBER), (1, 1), True),
    ('mov', (TokenType.ADDRESS_IN_REGISTER, TokenType.ADDRESS_LITERAL), (1, 8), True),
    ('mov', (TokenType.ADDRESS_LITERAL, TokenType.REGISTER), (8, 1), True),
    ('mov', (TokenType.ADDRESS_LITERAL, TokenType.ADDRESS_IN_REGISTER), (8, 1), True),
    ('mov', (TokenType.ADDRESS_LITERAL, TokenType.NUMBER), (8, 1), True),
    ('mov', (TokenType.ADDRESS_LITERAL, TokenType.ADDRESS_LITERAL), (8, 8), True),


    ('push', (TokenType.REGISTER,), (1,), False),
    ('push', (TokenType.ADDRESS_IN_REGISTER,), (1,), True),
    ('push', (TokenType.NUMBER,), (1,), True),
    ('push', (TokenType.ADDRESS_LITERAL,), (8,), True),

    ('pop', (TokenType.REGISTER,), (1,), False),
    ('pop', (TokenType.ADDRESS_IN_REGISTER,), (1,), True),
    ('pop', (TokenType.ADDRESS_LITERAL,), (8,), True),


    ('@', (TokenType.LABEL), (8,), False), # Doesn't get used, but it's here for completeness.


    ('jmp', (TokenType.NUMBER,), (8,), False),
    ('cjmp', (TokenType.NUMBER, TokenType.REGISTER), (8, 1), False),
    ('njmp', (TokenType.NUMBER, TokenType.REGISTER,), (8, 1), False),


    ('cmp', (TokenType.REGISTER, TokenType.REGISTER), (1, 1), False),
    ('cmp', (TokenType.REGISTER, TokenType.NUMBER), (1, 1), True),
    ('cmp', (TokenType.NUMBER, TokenType.REGISTER), (1, 1), True),
    ('cmp', (TokenType.NUMBER, TokenType.NUMBER), (1, 1), True),


    ('prt', (), (), False),
    ('prtstr', (None,), (None), False), # TODO: add string literal

    ('inint', (), (), False),
    ('instr', (), (), False),

    ('exit', (), (), False),


)

