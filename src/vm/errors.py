import enum
from typing import List


@enum.unique
class Errors(enum.IntEnum):

    def _generate_next_value_(name: str, start: int, count: int, last_values: List[int]) -> int:
        return count
    
    NO_ERROR = enum.auto()

    END_OF_FILE = enum.auto()
    INVALID_INPUT = enum.auto()
    GENERIC_ERROR = enum.auto()
    
