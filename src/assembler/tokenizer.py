from __future__ import annotations
import enum
from typing import Any, Dict, List, Tuple, Union

from shared.registers import Registers


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
    'pc': Registers.PROGRAM_COUNTER,
    'zf': Registers.ZERO_FLAG,
    'sf': Registers.SIGN_FLAG,
}


def is_name_character(char: str) -> bool:
    return char.isalpha() or char == '_'


token_type_names_table: Tuple[str] = \
(
    "REGISTER",
    "ADDRESS_IN_REGISTER",
    "NUMBER",
    "ADDRESS_LITERAL",
    "LABEL",
    "NAME",
    "ADDRESS_GENERIC",
    "CURRENT_POSITION"
)


@enum.unique
class TokenType(enum.IntEnum):

    def _generate_next_value_(name: str, start: int, count: int, last_values: List[int]) -> int:
        return count

    REGISTER = enum.auto()
    ADDRESS_IN_REGISTER = enum.auto()
    NUMBER = enum.auto()
    ADDRESS_LITERAL = enum.auto()

    LABEL = enum.auto()
    NAME = enum.auto()
    ADDRESS_GENERIC = enum.auto()
    
    CURRENT_POSITION = enum.auto()


    def __str__(self) -> str:
        return token_type_names_table[self.value]
    
    def __repr__(self) -> str:
        return token_type_names_table[self.value]


class Token:
    
    def __init__(self, type: TokenType, value: Any):
        self.type = type
        self.value = value

    def __str__(self) -> str:
        return f"<{self.type}: {self.value}>"
    
    def __repr__(self) -> str:
        return f"<{self.type}: {self.value}>"


def tokenize_operands(operands: str) -> List[Token]:
    tokens: List[Token] = []
    operands = operands.strip()
    if operands == "":
        return tokens
     
    current_token: Union[Token, None] = None
    for char in operands:

        if current_token is not None:

            if current_token.type == TokenType.ADDRESS_GENERIC:
                if char.isdigit():
                    current_token = Token(TokenType.ADDRESS_LITERAL, int(char))
                elif is_name_character(char):
                    current_token = Token(TokenType.ADDRESS_IN_REGISTER, char)
                
                continue   
                
            elif current_token.type == TokenType.ADDRESS_LITERAL:
                if char.isdigit():
                    current_token.value *= 10
                    current_token.value += int(char)
                    continue
                tokens.append(current_token)
                current_token = None 

            elif current_token.type == TokenType.ADDRESS_IN_REGISTER:
                if is_name_character(char):
                    current_token.value += char
                    continue

                if char == ' ':
                    continue
                if char != ']':
                    print(f'Expected a \']\' after address in argument list "{operands}", but \'{char}\' was provided.')
                    exit(1)

                register = register_map.get(current_token.value)
                if register is not None:
                    tokens.append(Token(TokenType.ADDRESS, register))
                else:
                    print(f"Unknown register {current_token.value} in argument list \"{operands}\".")
                    exit(1)
                
                continue

            elif current_token.type == TokenType.NAME:
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


        if char == '[':
            current_token = Token(TokenType.ADDRESS_GENERIC, None)
            continue
        
        if is_name_character(char):
            current_token = Token(TokenType.NAME, char)
            continue

        if char.isdigit():
            current_token = Token(TokenType.NUMBER, int(char))
            continue

        if char == ';':
            break
        if char == ',' or char == ' ':
            continue
        

        # If the character isn't handled, raise an error
        raise ValueError(f"Unhandled character: '{char}' in argument list \"{operands}\".")

    if current_token is not None:
        tokens.append(current_token)
    return tokens

