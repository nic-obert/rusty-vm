from typing import List


def load_assembly(file_path) -> List[str]:
    """
    Loads a file and returns its contents as a list of strings.
    """
    try:
        with open(file_path, 'r') as f:
           return f.readlines()
    except FileNotFoundError:
        print(f'File {file_path} not found.')
        exit(1)


def load_byte_code(file_path) -> bytes:
    """
    Loads a file and returns its contents as a byte string.
    """
    try:
        with open(file_path, 'rb') as f:
            return f.read()
    except FileNotFoundError:
        print(f'File {file_path} not found.')
        exit(1)


def save_byte_code(byte_code: bytes, file_path: str) -> None:
    """
    Saves a byte string to a file.
    """
    with open(file_path, 'wb') as f:
        f.write(byte_code)

