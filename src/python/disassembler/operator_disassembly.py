from typing import Callable, Tuple

from shared.registers import register_names


operator_disassembly_table: Tuple[Callable[[bytes], str]] = \
(
    lambda byte_code: register_names[byte_code[0]],
    lambda byte_code: f'[{register_names[byte_code[0]]}]',
    lambda byte_code: str(int.from_bytes(byte_code, byteorder='little')),
    lambda byte_code: f'[0x{byte_code.hex()}]',
)

