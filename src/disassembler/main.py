from sys import argv

from shared import files
from disassembler import disassemble


def main() -> None:
    if len(argv) != 2:
        print(f"Usage: {argv[0]} <file_path>")
        exit(1)

    byte_code = files.load_byte_code(argv[1])
    assembly = disassemble(byte_code)

    i = 1
    for line in assembly:
        print(f'{i}: {line}')
        i += 1


if __name__ == "__main__":
    main()

