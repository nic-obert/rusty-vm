from sys import argv

from shared import files
from processor import Processor
from memory import Memory


MEMORY_SIZE = 1024


def main() -> None:
    if len(argv) != 2:
        print("Usage: python3 main.py <file_path>")
        exit(1)
    
    byte_code = files.load_byte_code(argv[1])
    processor = Processor(MEMORY_SIZE)

    processor.execute(byte_code)


if __name__ == "__main__":
    main()

