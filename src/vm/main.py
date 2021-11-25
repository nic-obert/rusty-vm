from sys import argv

from shared import files
from processor import Processor


MEMORY_SIZE = 1024


def main() -> None:
    if len(argv) < 2:
        print(f"Usage: {argv[0]} <file_path>")
        exit(1)
    
    byte_code = files.load_byte_code(argv[1])
    processor = Processor(MEMORY_SIZE)

    verbose = '-v' in argv
    processor.execute(byte_code, verbose)


if __name__ == "__main__":
    main()

