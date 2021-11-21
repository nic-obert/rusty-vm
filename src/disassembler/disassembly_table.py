from typing import Tuple

from shared.token import TokenType

"""
Conversion table for the byte code to assembly.
Structure:
    (
        string representation for the instruction,
        (argument type1, argument type2, ...),
    )

An empty argument type list means the instruction does not take any arguments.
"""
disassembly_table: Tuple[Tuple[str, Tuple[TokenType]]] = \
(
    ('add', ()),

    ('sub', ()),

    ('mul', ()),

    ('div', ()),

    ('mod', ()),

    
    ('inc', ())

    ('inc1', (TokenType.ADDRESS_IN_REGISTER,)),
    ('inc1', (TokenType.ADDRESS_LITERAL,)),

    ('inc2', (TokenType.ADDRESS_IN_REGISTER,)),
    ('inc2', (TokenType.ADDRESS_LITERAL,)),

    ('inc4', (TokenType.ADDRESS_IN_REGISTER,)),
    ('inc4', (TokenType.ADDRESS_LITERAL,)),

    ('inc8', (TokenType.ADDRESS_IN_REGISTER,)),
    ('inc8', (TokenType.ADDRESS_LITERAL,)),


    ('dec', ())

    ('dec1', (TokenType.ADDRESS_IN_REGISTER,)),
    ('dec1', (TokenType.ADDRESS_LITERAL,)),

    ('dec2', (TokenType.ADDRESS_IN_REGISTER,)),
    ('dec2', (TokenType.ADDRESS_LITERAL,)),

    ('dec4', (TokenType.ADDRESS_IN_REGISTER,)),
    ('dec4', (TokenType.ADDRESS_LITERAL,)),

    ('dec8', (TokenType.ADDRESS_IN_REGISTER,)),
    ('dec8', (TokenType.ADDRESS_LITERAL,)),


    ('nop', ()),


    ('ld', (TokenType.REGISTER, TokenType.REGISTER)),

    ('ld1', (TokenType.REGISTER, TokenType.ADDRESS_IN_REGISTER)),
    ('ld1', (TokenType.REGISTER, TokenType.NUMBER)),
    ('ld1', (TokenType.REGISTER, TokenType.ADDRESS_LITERAL)),

    ('ld2', (TokenType.REGISTER, TokenType.ADDRESS_IN_REGISTER)),
    ('ld2', (TokenType.REGISTER, TokenType.NUMBER)),
    ('ld2', (TokenType.REGISTER, TokenType.ADDRESS_LITERAL)),

    ('ld4', (TokenType.REGISTER, TokenType.ADDRESS_IN_REGISTER)),
    ('ld4', (TokenType.REGISTER, TokenType.NUMBER)),
    ('ld4', (TokenType.REGISTER, TokenType.ADDRESS_LITERAL)),

    ('ld8', (TokenType.REGISTER, TokenType.ADDRESS_IN_REGISTER)),
    ('ld8', (TokenType.REGISTER, TokenType.NUMBER)),
    ('ld8', (TokenType.REGISTER, TokenType.ADDRESS_LITERAL)),


    ('mov', (TokenType.REGISTER, TokenType.REGISTER)),

    ('mov1', (TokenType.REGISTER, TokenType.ADDRESS_IN_REGISTER)),
    ('mov1', (TokenType.REGISTER, TokenType.NUMBER)),
    ('mov1', (TokenType.REGISTER, TokenType.ADDRESS_LITERAL)),
    ('mov1', (TokenType.ADDRESS_IN_REGISTER, TokenType.REGISTER)),
    ('mov1', (TokenType.ADDRESS_IN_REGISTER, TokenType.ADDRESS_IN_REGISTER)),
    ('mov1', (TokenType.ADDRESS_IN_REGISTER, TokenType.NUMBER)),
    ('mov1', (TokenType.ADDRESS_IN_REGISTER, TokenType.ADDRESS_LITERAL)),
    ('mov1', (TokenType.ADDRESS_LITERAL, TokenType.REGISTER)),
    ('mov1', (TokenType.ADDRESS_LITERAL, TokenType.ADDRESS_IN_REGISTER)),
    ('mov1', (TokenType.ADDRESS_LITERAL, TokenType.NUMBER)),
    ('mov1', (TokenType.ADDRESS_LITERAL, TokenType.ADDRESS_LITERAL)),

    ('mov2', (TokenType.REGISTER, TokenType.ADDRESS_IN_REGISTER)),
    ('mov2', (TokenType.REGISTER, TokenType.NUMBER)),
    ('mov2', (TokenType.REGISTER, TokenType.ADDRESS_LITERAL)),
    ('mov2', (TokenType.ADDRESS_IN_REGISTER, TokenType.REGISTER)),
    ('mov2', (TokenType.ADDRESS_IN_REGISTER, TokenType.ADDRESS_IN_REGISTER)),
    ('mov2', (TokenType.ADDRESS_IN_REGISTER, TokenType.NUMBER)),
    ('mov2', (TokenType.ADDRESS_IN_REGISTER, TokenType.ADDRESS_LITERAL)),
    ('mov2', (TokenType.ADDRESS_LITERAL, TokenType.REGISTER)),
    ('mov2', (TokenType.ADDRESS_LITERAL, TokenType.ADDRESS_IN_REGISTER)),
    ('mov2', (TokenType.ADDRESS_LITERAL, TokenType.NUMBER)),
    ('mov2', (TokenType.ADDRESS_LITERAL, TokenType.ADDRESS_LITERAL)),

    ('mov4', (TokenType.REGISTER, TokenType.ADDRESS_IN_REGISTER)),
    ('mov4', (TokenType.REGISTER, TokenType.NUMBER)),
    ('mov4', (TokenType.REGISTER, TokenType.ADDRESS_LITERAL)),
    ('mov4', (TokenType.ADDRESS_IN_REGISTER, TokenType.REGISTER)),
    ('mov4', (TokenType.ADDRESS_IN_REGISTER, TokenType.ADDRESS_IN_REGISTER)),
    ('mov4', (TokenType.ADDRESS_IN_REGISTER, TokenType.NUMBER)),
    ('mov4', (TokenType.ADDRESS_IN_REGISTER, TokenType.ADDRESS_LITERAL)),
    ('mov4', (TokenType.ADDRESS_LITERAL, TokenType.REGISTER)),
    ('mov4', (TokenType.ADDRESS_LITERAL, TokenType.ADDRESS_IN_REGISTER)),
    ('mov4', (TokenType.ADDRESS_LITERAL, TokenType.NUMBER)),
    ('mov4', (TokenType.ADDRESS_LITERAL, TokenType.ADDRESS_LITERAL)),

    ('mov8', (TokenType.REGISTER, TokenType.ADDRESS_IN_REGISTER)),
    ('mov8', (TokenType.REGISTER, TokenType.NUMBER)),
    ('mov8', (TokenType.REGISTER, TokenType.ADDRESS_LITERAL)),
    ('mov8', (TokenType.ADDRESS_IN_REGISTER, TokenType.REGISTER)),
    ('mov8', (TokenType.ADDRESS_IN_REGISTER, TokenType.ADDRESS_IN_REGISTER)),
    ('mov8', (TokenType.ADDRESS_IN_REGISTER, TokenType.NUMBER)),
    ('mov8', (TokenType.ADDRESS_IN_REGISTER, TokenType.ADDRESS_LITERAL)),
    ('mov8', (TokenType.ADDRESS_LITERAL, TokenType.REGISTER)),
    ('mov8', (TokenType.ADDRESS_LITERAL, TokenType.ADDRESS_IN_REGISTER)),
    ('mov8', (TokenType.ADDRESS_LITERAL, TokenType.NUMBER)),
    ('mov8', (TokenType.ADDRESS_LITERAL, TokenType.ADDRESS_LITERAL)),


    ('st1', (TokenType.ADDRESS_IN_REGISTER, TokenType.REGISTER)),
    ('st1', (TokenType.ADDRESS_LITERAL, TokenType.REGISTER)),

    ('st2', (TokenType.ADDRESS_IN_REGISTER, TokenType.REGISTER)),
    ('st2', (TokenType.ADDRESS_LITERAL, TokenType.REGISTER)),

    ('st4', (TokenType.ADDRESS_IN_REGISTER, TokenType.REGISTER)),
    ('st4', (TokenType.ADDRESS_LITERAL, TokenType.REGISTER)),

    ('st8', (TokenType.ADDRESS_IN_REGISTER, TokenType.REGISTER)),
    ('st8', (TokenType.ADDRESS_LITERAL, TokenType.REGISTER)),


    ('push', (TokenType.REGISTER,)),

    ('push1', (TokenType.ADDRESS_IN_REGISTER,)),
    ('push1', (TokenType.NUMBER,)),
    ('push1', (TokenType.ADDRESS_LITERAL,)),

    ('push2', (TokenType.ADDRESS_IN_REGISTER,)),
    ('push2', (TokenType.NUMBER,)),
    ('push2', (TokenType.ADDRESS_LITERAL,)),

    ('push4', (TokenType.ADDRESS_IN_REGISTER,)),
    ('push4', (TokenType.NUMBER,)),
    ('push4', (TokenType.ADDRESS_LITERAL,)),

    ('push8', (TokenType.ADDRESS_IN_REGISTER,)),
    ('push8', (TokenType.NUMBER,)),
    ('push8', (TokenType.ADDRESS_LITERAL,)),


    ('pop', (TokenType.REGISTER,)),

    ('pop1', (TokenType.ADDRESS_IN_REGISTER,)),
    ('pop1', (TokenType.ADDRESS_LITERAL,)),

    ('pop2', (TokenType.ADDRESS_IN_REGISTER,)),
    ('pop2', (TokenType.ADDRESS_LITERAL,)),

    ('pop4', (TokenType.ADDRESS_IN_REGISTER,)),
    ('pop4', (TokenType.ADDRESS_LITERAL,)),

    ('pop8', (TokenType.ADDRESS_IN_REGISTER,)),
    ('pop8', (TokenType.ADDRESS_LITERAL,)),


    ('@', ()),


    ('jmp', (TokenType.NUMBER,)),
    ('cjmp', (TokenType.NUMBER, TokenType.REGISTER)),
    ('njmp', (TokenType.REGISTER,)),


    ('cmp', (TokenType.REGISTER, TokenType.REGISTER)),

    ('cmp1', (TokenType.REGISTER, TokenType.NUMBER)),
    ('cmp1', (TokenType.NUMBER, TokenType.REGISTER)),
    ('cmp1', (TokenType.NUMBER, TokenType.NUMBER)),

    ('cmp2', (TokenType.REGISTER, TokenType.NUMBER)),
    ('cmp2', (TokenType.NUMBER, TokenType.REGISTER)),
    ('cmp2', (TokenType.NUMBER, TokenType.NUMBER)),

    ('cmp4', (TokenType.REGISTER, TokenType.NUMBER)),
    ('cmp4', (TokenType.NUMBER, TokenType.REGISTER)),
    ('cmp4', (TokenType.NUMBER, TokenType.NUMBER)),

    ('cmp8', (TokenType.REGISTER, TokenType.NUMBER)),
    ('cmp8', (TokenType.NUMBER, TokenType.REGISTER)),
    ('cmp8', (TokenType.NUMBER, TokenType.NUMBER)),


    ('prt', ()),

    ('prtstr', (None,)), # TODO: add string literal


    ('exit', ()),


)

