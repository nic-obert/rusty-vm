from __future__ import annotations
import enum
from typing import Any, Dict, List, Union

from src.shared.registers import Registers


register_map: Dict[str, Registers] = \
{
    'a': Registers.A,
    'b': Registers.B,
    'c': Registers.C,
    'd': Registers.D,
    'e': Registers.E,
    'f': Registers.F,
    'g': Registers.G,
    'h': Registers.H,
    'sp': Registers.STACK_POINTER,
    'pc': Registers.PROGRAM_COUNTER
}


def is_name_character(char: str) -> bool:
    return char.isalpha() or char == '_'


class TokenType(enum.IntEnum):

    def _generate_next_value_(name: str, start: int, count: int, last_values: List[int]) -> int:
        return count

    REGISTER = enum.auto()
    LABEL = enum.auto()
    NUMBER = enum.auto()
    PARENTHESIS = enum.auto()
    ADDRESS_OF = enum.auto()
    NAME = enum.auto()
    ADDRESS = enum.auto()
    CURRENT_POSITION = enum.auto()

    names_table: List[str] = [
        "REGISTER",
        "LABEL",
        "NUMBER",
        "PARENTHESIS",
        "ADDRESS_OF ",
        "NAME",
        "ADDRESS",
        "CURRENT_POSITION"
    ]

    def __str__(self) -> str:
        return self.names_table[self.value]
    
    def __repr__(self) -> str:
        return self.names_table[self.value]


class Token:
    
    def __init__(self, type: TokenType, value: Any):
        self.type = type
        self.value = value

    def __str__(self) -> str:
        return f"<{self.type}: {self.value}>"


def tokenize_operands(operands: str) -> List[Token]:
    tokens: List[Token] = []
    operands = operands.strip()
    if operands == "":
        return tokens
     
    current_token: Union[Token, None] = None
    for char in operands:

        if current_token is not None:

            if current_token.type == TokenType.ADDRESS_OF:
                if is_name_character(char):
                    current_token.value += char # TODO to complete
                current_token.value += char

            if current_token.type == TokenType.NAME:
                if is_name_character(char):
                    current_token.value += char
                    continue
                if char == ':':
                    tokens.append(Token(TokenType.LABEL, current_token.value))
                    current_token = None
                    continue

                register = register_map.get(current_token.value)
                if register is not None:
                    tokens.append(Token(TokenType.REGISTER, register))
                else:
                    tokens.append(current_token)
                current_token = None
        
            elif current_token.type == TokenType.NUMBER:
                if char.isdigit():
                    current_token.value *= 10
                    current_token.value += int(char)
                    continue
                tokens.append(current_token)
                current_token = None


        if char == '(':
            tokens.append(Token(TokenType.PARENTHESIS, None))
            continue
        if char == ')':
            tokens.append(Token(TokenType.PARENTHESIS, None))
            continue

        if char == '[':
            current_token = Token(TokenType.ADDRESS_OF, '')
            continue
        
        if is_name_character(char):
            current_token = Token(TokenType.NAME, char)
            continue

        if char == ';':
            break
        if char == ',' or char == ' ':
            continue
        

        # If the character isn't handled, raise an error
        raise ValueError(f"Unhandled character: {char} in argument list {operands}")

    if current_token is not None:
        tokens.append(current_token)
    return tokens

