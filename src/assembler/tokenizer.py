from __future__ import annotations
from typing import List, Union

from shared.registers import register_table
from shared.token import Token, TokenType


def is_name_character(char: str) -> bool:
    return char.isalpha() or char == '_'


def tokenize_operands(operands: str) -> List[Token]:
    tokens: List[Token] = []
    # Remove redundant spaces and add a semicolon at the end in order to make the loop iterate one more time
    operands = operands.strip() + ';'
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

                register = register_table.get(current_token.value)
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

                register = register_table.get(current_token.value)
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

    return tokens

