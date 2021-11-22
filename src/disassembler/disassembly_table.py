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
disassembly_table: Tuple[Tuple[str, Tuple[TokenType], Tuple[int]]] = \
(
    ('add', (), ()),

    ('sub', (), ()),

    ('mul', (), ()),

    ('div', (), ()),

    ('mod', (), ()),

    
    ('inc', (), ()),

    ('inc1', (TokenType.ADDRESS_IN_REGISTER,), (1,)),
    ('inc1', (TokenType.ADDRESS_LITERAL,), (8,)),

    ('inc2', (TokenType.ADDRESS_IN_REGISTER,), (1,)),
    ('inc2', (TokenType.ADDRESS_LITERAL,), (8,)),

    ('inc4', (TokenType.ADDRESS_IN_REGISTER,), (1,)),
    ('inc4', (TokenType.ADDRESS_LITERAL,), (8,)),

    ('inc8', (TokenType.ADDRESS_IN_REGISTER,), (1,)),
    ('inc8', (TokenType.ADDRESS_LITERAL,), (8,)),


    ('dec', (), ()),

    ('dec1', (TokenType.ADDRESS_IN_REGISTER,), (1,)),
    ('dec1', (TokenType.ADDRESS_LITERAL,), (8,)),

    ('dec2', (TokenType.ADDRESS_IN_REGISTER,), (1,)),
    ('dec2', (TokenType.ADDRESS_LITERAL,), (8,)),

    ('dec4', (TokenType.ADDRESS_IN_REGISTER,), (1,)),
    ('dec4', (TokenType.ADDRESS_LITERAL,), (8,)),

    ('dec8', (TokenType.ADDRESS_IN_REGISTER,), (1,)),
    ('dec8', (TokenType.ADDRESS_LITERAL,), (8,)),


    ('nop', (), ()),


    ('ld', (TokenType.REGISTER, TokenType.REGISTER), (1, 1)),

    ('ld1', (TokenType.REGISTER, TokenType.ADDRESS_IN_REGISTER), (1, 1)),
    ('ld1', (TokenType.REGISTER, TokenType.NUMBER), (1, 1)),
    ('ld1', (TokenType.REGISTER, TokenType.ADDRESS_LITERAL), (1, 8)),

    ('ld2', (TokenType.REGISTER, TokenType.ADDRESS_IN_REGISTER), (1, 1)),
    ('ld2', (TokenType.REGISTER, TokenType.NUMBER), (1, 2)),
    ('ld2', (TokenType.REGISTER, TokenType.ADDRESS_LITERAL), (1, 8)),

    ('ld4', (TokenType.REGISTER, TokenType.ADDRESS_IN_REGISTER), (1, 1)),
    ('ld4', (TokenType.REGISTER, TokenType.NUMBER), (1, 4)),
    ('ld4', (TokenType.REGISTER, TokenType.ADDRESS_LITERAL)),

    ('ld8', (TokenType.REGISTER, TokenType.ADDRESS_IN_REGISTER), (1, 1)),
    ('ld8', (TokenType.REGISTER, TokenType.NUMBER), (1, 8)),
    ('ld8', (TokenType.REGISTER, TokenType.ADDRESS_LITERAL), (1, 8)),


    ('mov', (TokenType.REGISTER, TokenType.REGISTER), (1, 1)),

    ('mov1', (TokenType.REGISTER, TokenType.ADDRESS_IN_REGISTER), (1, 1)),
    ('mov1', (TokenType.REGISTER, TokenType.NUMBER), (1, 1)),
    ('mov1', (TokenType.REGISTER, TokenType.ADDRESS_LITERAL), (1, 8)),
    ('mov1', (TokenType.ADDRESS_IN_REGISTER, TokenType.REGISTER), (1, 1)),
    ('mov1', (TokenType.ADDRESS_IN_REGISTER, TokenType.ADDRESS_IN_REGISTER), (1, 1)),
    ('mov1', (TokenType.ADDRESS_IN_REGISTER, TokenType.NUMBER), (1, 1)),
    ('mov1', (TokenType.ADDRESS_IN_REGISTER, TokenType.ADDRESS_LITERAL), (1, 8)),
    ('mov1', (TokenType.ADDRESS_LITERAL, TokenType.REGISTER), (8, 1)),
    ('mov1', (TokenType.ADDRESS_LITERAL, TokenType.ADDRESS_IN_REGISTER), (8, 1)),
    ('mov1', (TokenType.ADDRESS_LITERAL, TokenType.NUMBER), (8, 1)),
    ('mov1', (TokenType.ADDRESS_LITERAL, TokenType.ADDRESS_LITERAL), (8, 8)),

    ('mov2', (TokenType.REGISTER, TokenType.ADDRESS_IN_REGISTER), (1, 1)),
    ('mov2', (TokenType.REGISTER, TokenType.NUMBER), (1, 2)),
    ('mov2', (TokenType.REGISTER, TokenType.ADDRESS_LITERAL), (1, 8)),
    ('mov2', (TokenType.ADDRESS_IN_REGISTER, TokenType.REGISTER), (1, 1)),
    ('mov2', (TokenType.ADDRESS_IN_REGISTER, TokenType.ADDRESS_IN_REGISTER), (1, 1)),
    ('mov2', (TokenType.ADDRESS_IN_REGISTER, TokenType.NUMBER), (1, 2)),
    ('mov2', (TokenType.ADDRESS_IN_REGISTER, TokenType.ADDRESS_LITERAL), (1, 8)),
    ('mov2', (TokenType.ADDRESS_LITERAL, TokenType.REGISTER), (8, 1)),
    ('mov2', (TokenType.ADDRESS_LITERAL, TokenType.ADDRESS_IN_REGISTER), (8, 1)),
    ('mov2', (TokenType.ADDRESS_LITERAL, TokenType.NUMBER), (8, 2)),
    ('mov2', (TokenType.ADDRESS_LITERAL, TokenType.ADDRESS_LITERAL), (8, 8)),

    ('mov4', (TokenType.REGISTER, TokenType.ADDRESS_IN_REGISTER), (1, 1)),
    ('mov4', (TokenType.REGISTER, TokenType.NUMBER), (1, 4)),
    ('mov4', (TokenType.REGISTER, TokenType.ADDRESS_LITERAL), (1, 8)),
    ('mov4', (TokenType.ADDRESS_IN_REGISTER, TokenType.REGISTER), (1, 1)),
    ('mov4', (TokenType.ADDRESS_IN_REGISTER, TokenType.ADDRESS_IN_REGISTER), (1, 1)),
    ('mov4', (TokenType.ADDRESS_IN_REGISTER, TokenType.NUMBER), (1, 4)),
    ('mov4', (TokenType.ADDRESS_IN_REGISTER, TokenType.ADDRESS_LITERAL), (1, 8)),
    ('mov4', (TokenType.ADDRESS_LITERAL, TokenType.REGISTER), (8, 1)),
    ('mov4', (TokenType.ADDRESS_LITERAL, TokenType.ADDRESS_IN_REGISTER), (8, 1)),
    ('mov4', (TokenType.ADDRESS_LITERAL, TokenType.NUMBER), (8, 4)),
    ('mov4', (TokenType.ADDRESS_LITERAL, TokenType.ADDRESS_LITERAL), (8, 8)),

    ('mov8', (TokenType.REGISTER, TokenType.ADDRESS_IN_REGISTER), (1, 1)),
    ('mov8', (TokenType.REGISTER, TokenType.NUMBER), (1, 8)),
    ('mov8', (TokenType.REGISTER, TokenType.ADDRESS_LITERAL), (1, 8)),
    ('mov8', (TokenType.ADDRESS_IN_REGISTER, TokenType.REGISTER), (1, 1)),
    ('mov8', (TokenType.ADDRESS_IN_REGISTER, TokenType.ADDRESS_IN_REGISTER), (1, 1)),
    ('mov8', (TokenType.ADDRESS_IN_REGISTER, TokenType.NUMBER), (1, 8)),
    ('mov8', (TokenType.ADDRESS_IN_REGISTER, TokenType.ADDRESS_LITERAL), (1, 8)),
    ('mov8', (TokenType.ADDRESS_LITERAL, TokenType.REGISTER), (8, 1)),
    ('mov8', (TokenType.ADDRESS_LITERAL, TokenType.ADDRESS_IN_REGISTER), (8, 1)),
    ('mov8', (TokenType.ADDRESS_LITERAL, TokenType.NUMBER), (8, 8)),
    ('mov8', (TokenType.ADDRESS_LITERAL, TokenType.ADDRESS_LITERAL), (8, 8)),


    ('st1', (TokenType.ADDRESS_IN_REGISTER, TokenType.REGISTER), (1, 1)),
    ('st1', (TokenType.ADDRESS_LITERAL, TokenType.REGISTER), (8, 1)),

    ('st2', (TokenType.ADDRESS_IN_REGISTER, TokenType.REGISTER), (1, 1)),
    ('st2', (TokenType.ADDRESS_LITERAL, TokenType.REGISTER), (8, 1)),

    ('st4', (TokenType.ADDRESS_IN_REGISTER, TokenType.REGISTER), (1, 1)),
    ('st4', (TokenType.ADDRESS_LITERAL, TokenType.REGISTER), (8, 1)),

    ('st8', (TokenType.ADDRESS_IN_REGISTER, TokenType.REGISTER), (1, 1)),
    ('st8', (TokenType.ADDRESS_LITERAL, TokenType.REGISTER), (8, 1)),


    ('push', (TokenType.REGISTER,), (1,)),

    ('push1', (TokenType.ADDRESS_IN_REGISTER,), (1,)),
    ('push1', (TokenType.NUMBER,), (1,)),
    ('push1', (TokenType.ADDRESS_LITERAL,), (8,)),

    ('push2', (TokenType.ADDRESS_IN_REGISTER,), (1,)),
    ('push2', (TokenType.NUMBER,), (2,)),
    ('push2', (TokenType.ADDRESS_LITERAL,), (8,)),

    ('push4', (TokenType.ADDRESS_IN_REGISTER,), (1,)),
    ('push4', (TokenType.NUMBER,), (4,)),
    ('push4', (TokenType.ADDRESS_LITERAL,), (8,)),

    ('push8', (TokenType.ADDRESS_IN_REGISTER,), (1,)),
    ('push8', (TokenType.NUMBER,), (8,)),
    ('push8', (TokenType.ADDRESS_LITERAL,), (8,)),


    ('pop', (TokenType.REGISTER,), (1,)),

    ('pop1', (TokenType.ADDRESS_IN_REGISTER,), (1,)),
    ('pop1', (TokenType.ADDRESS_LITERAL,), (8,)),

    ('pop2', (TokenType.ADDRESS_IN_REGISTER,), (1,)),
    ('pop2', (TokenType.ADDRESS_LITERAL,), (8,)),

    ('pop4', (TokenType.ADDRESS_IN_REGISTER,), (1,)),
    ('pop4', (TokenType.ADDRESS_LITERAL,), (8,)),

    ('pop8', (TokenType.ADDRESS_IN_REGISTER,), (1,)),
    ('pop8', (TokenType.ADDRESS_LITERAL,), (8,)),


    ('@', (TokenType.LABEL), (8,)), # Doesn't get used, but it's here for completeness.


    ('jmp', (TokenType.NUMBER,), (8,)),
    ('cjmp', (TokenType.NUMBER, TokenType.REGISTER), (8, 1)),
    ('njmp', (TokenType.NUMBER, TokenType.REGISTER,), (8, 1)),


    ('cmp', (TokenType.REGISTER, TokenType.REGISTER), (1, 1)),

    ('cmp1', (TokenType.REGISTER, TokenType.NUMBER), (1, 1)),
    ('cmp1', (TokenType.NUMBER, TokenType.REGISTER), (1, 1)),
    ('cmp1', (TokenType.NUMBER, TokenType.NUMBER), (1, 1)),

    ('cmp2', (TokenType.REGISTER, TokenType.NUMBER), (1, 2)),
    ('cmp2', (TokenType.NUMBER, TokenType.REGISTER), (2, 1)),
    ('cmp2', (TokenType.NUMBER, TokenType.NUMBER), (2, 2)),

    ('cmp4', (TokenType.REGISTER, TokenType.NUMBER), (1, 4)),
    ('cmp4', (TokenType.NUMBER, TokenType.REGISTER), (4, 1)),
    ('cmp4', (TokenType.NUMBER, TokenType.NUMBER), (4, 4)),

    ('cmp8', (TokenType.REGISTER, TokenType.NUMBER), (1, 8)),
    ('cmp8', (TokenType.NUMBER, TokenType.REGISTER), (8, 1)),
    ('cmp8', (TokenType.NUMBER, TokenType.NUMBER), (8, 8)),


    ('prt', (), ()),

    ('prtstr', (None,), (None)), # TODO: add string literal


    ('exit', (), ()),


)

