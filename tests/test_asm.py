import subprocess
import os
from sys import argv

TEST_FILE_NAME = "test.asm.test"
OUTPUT_FILE_NAME = "test.test.bc"


def address(addr: int) -> bytes:
    return addr.to_bytes(8, "little")


# Asm statement: expected byte code
INSTRUCTIONS = {
    # 'add': [0],
    # 'sub': [1],
    # 'mul': [2],
    # 'div': [3],
    # 'mod': [4],
    # 'inc a': [5, 0],
    # 'inc b': [5, 1],
    # 'inc c': [5, 2],
    # 'inc d': [5, 3],
    # 'dec a': [8, 0],
    # 'dec b': [8, 1],
    # 'dec c': [8, 2],
    # 'dec d': [8, 3],
    # 'inc1 [a]': [6, 1, 0],
    # 'inc1 [b]': [6, 1, 1],
    # 'inc2 [c]': [6, 2, 2],
    # 'inc2 [d]': [6, 2, 3],
    # 'inc4 [a]': [6, 4, 0],
    # 'inc8 [b]': [6, 8, 1],
    # 'dec1 [a]': [9, 1, 0],
    # 'dec1 [b]': [9, 1, 1],
    # 'dec2 [c]': [9, 2, 2],
    # 'dec2 [d]': [9, 2, 3],
    # 'dec4 [a]': [9, 4, 0],
    # 'dec8 [b]': [9, 8, 1],
    # 'nop': [11],
    # 'inc1 [1]': [7, 1, *address(1)],
    # 'inc2 [1234]': [7, 2, *address(1234)],
    # 'inc4 [1230000]': [7, 4, *address(1230000)],
    # 'inc8 [123456789010]': [7, 8, *address(123456789010)],
    # 'dec1 [123]': [10, 1, *address(123)],
    # 'dec2 [1234]': [10, 2, *address(1234)],
    # 'dec4 [1230000]': [10, 4, *address(1230000)],
    # 'dec8 [123456789010]': [10, 8, *address(123456789010)],


}


def main() -> None:

    success = True
    keep_files = '-k' in argv
    
    with open(TEST_FILE_NAME, "w") as f:
        for instruction, byte_code in INSTRUCTIONS.items():
            f.write(instruction + "\n")

    if subprocess.run(["./assembler.sh", TEST_FILE_NAME, "-o", OUTPUT_FILE_NAME]).returncode != 0:
        print("Assembler failed")
        exit(1)

    with open(OUTPUT_FILE_NAME, "rb") as f:
        byte_code = f.read()

    for instruction, expected_byte_code in INSTRUCTIONS.items():
        print(f"Testing '{instruction}'...")

        if byte_code.startswith(bytes(expected_byte_code)):
            print("OK")
            byte_code = byte_code[len(expected_byte_code):]
            continue

        print("FAIL")
        print(f"Expected: {expected_byte_code}")
        decimal_bytes = [int(b) for b in byte_code[:len(expected_byte_code)]]
        print(f"Got: {decimal_bytes}")
        success = False
        break

    if not keep_files:
        os.remove(TEST_FILE_NAME)
        os.remove(OUTPUT_FILE_NAME)

    if success:
        print("All tests passed!")
    else:
        print("Some tests failed!")


if __name__ == "__main__":
    main()

