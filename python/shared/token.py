import enum
from typing import Any, List, Tuple


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
        return f"<{str(self.type)}: {self.value}>"
    
    def __repr__(self) -> str:
        return f"<{str(self.type)}: {self.value}>"

