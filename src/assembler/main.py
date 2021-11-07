import pathlib
from sys import argv

from src.shared import files


def main() -> None:
    if len(argv) != 2:
        print("Usage: python3 assembler.py <file_path>")
        exit(1)

    assembly = files.load_file(argv[1])
    byte_code = assemble(assembly)
    
    new_file_name = pathlib.Path(argv[1]).stem + '.bc'
    files.save_byte_code(byte_code, new_file_name)


if __name__ == "__main__":
    main()

