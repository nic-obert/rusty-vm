import os

TEST_FILE_NAME = "test.asm.test"
OUTPUT_FILE_NAME = "test.bc.test"

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
    'inc1 [a]': [6, 1, 0],
}


def main() -> None:

    success = True
    
    with open(TEST_FILE_NAME, "w") as f:
        for instruction, byte_code in INSTRUCTIONS.items():
            f.write(instruction + "\n")

    os.system(f"./assembler.sh {TEST_FILE_NAME} -o {OUTPUT_FILE_NAME}")

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

    os.remove(TEST_FILE_NAME)
    os.remove(OUTPUT_FILE_NAME)

    if success:
        print("All tests passed!")
    else:
        print("Some tests failed!")


if __name__ == "__main__":
    main()

