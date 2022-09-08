import subprocess
import os
from sys import argv


TEST_FILE_NAME = "test.asm.test"
OUTPUT_FILE_NAME = "test.test.bc"


def b(num: int, size: int) -> bytes:
    return num.to_bytes(size, "little")


def addr(addr: int) -> bytes:
    return b(addr, 8)


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

    # 'inc1 [1]': [7, 1, *addr(1)],
    # 'inc2 [1234]': [7, 2, *addr(1234)],
    # 'inc4 [1230000]': [7, 4, *addr(1230000)],
    # 'inc8 [123456789010]': [7, 8, *addr(123456789010)],
    # 'dec1 [123]': [10, 1, *addr(123)],
    # 'dec2 [1234]': [10, 2, *addr(1234)],
    # 'dec4 [1230000]': [10, 4, *addr(1230000)],
    # 'dec8 [123456789010]': [10, 8, *addr(123456789010)],

    # 'mov a b': [12, 0, 1],
    # 'mov b c': [12, 1, 2],

    # 'mov1 a [b]': [13, 1, 0, 1],
    # 'mov1 b [c]': [13, 1, 1, 2],
    # 'mov2 a [b]': [13, 2, 0, 1],
    # 'mov2 b [c]': [13, 2, 1, 2],
    # 'mov4 a [b]': [13, 4, 0, 1],
    # 'mov4 b [c]': [13, 4, 1, 2],
    # 'mov8 a [b]': [13, 8, 0, 1],
    # 'mov8 b [c]': [13, 8, 1, 2],

    # 'mov1 a 43': [14, 1, 0, 43],
    # 'mov1 b 43': [14, 1, 1, 43],
    # 'mov2 a 4343': [14, 2, 0, *b(4343, 2)],
    # 'mov2 b 4343': [14, 2, 1, *b(4343, 2)],
    # 'mov4 a 43434343': [14, 4, 0, *b(43434343, 4)],
    # 'mov4 b 43434343': [14, 4, 1, *b(43434343, 4)],
    # 'mov8 a 4343434343434343': [14, 8, 0, *b(4343434343434343, 8)],
    # 'mov8 b 4343434343434343': [14, 8, 1, *b(4343434343434343, 8)],
    
    # 'mov1 a [1]': [15, 1, 0, *addr(1)],
    # 'mov1 b [1234]': [15, 1, 1, *addr(1234)],
    # 'mov2 a [1230000]': [15, 2, 0, *addr(1230000)],
    # 'mov2 b [123456789010]': [15, 2, 1, *addr(123456789010)],
    # 'mov4 a [123]': [15, 4, 0, *addr(123)],
    # 'mov4 b [1234]': [15, 4, 1, *addr(1234)],
    # 'mov8 a [1230000]': [15, 8, 0, *addr(1230000)],
    # 'mov8 b [123456789010]': [15, 8, 1, *addr(123456789010)],

    # 'mov1 [a] b': [16, 1, 0, 1],
    # 'mov1 [b] c': [16, 1, 1, 2],
    # 'mov2 [a] b': [16, 2, 0, 1],
    # 'mov2 [b] c': [16, 2, 1, 2],
    # 'mov4 [a] b': [16, 4, 0, 1],
    # 'mov4 [b] c': [16, 4, 1, 2],
    # 'mov8 [a] b': [16, 8, 0, 1],
    # 'mov8 [b] c': [16, 8, 1, 2],
    
    # 'mov1 [a] [b]': [17, 1, 0, 1],
    # 'mov2 [a] [b]': [17, 2, 0, 1],
    # 'mov4 [a] [b]': [17, 4, 0, 1],
    # 'mov8 [a] [b]': [17, 8, 0, 1],

    # 'mov1 [a] 43': [18, 1, 0, 43],
    # 'mov2 [a] 4343': [18, 2, 0, *b(4343, 2)],
    # 'mov4 [a] 43434343': [18, 4, 0, *b(43434343, 4)],
    # 'mov8 [a] 4343434343434343': [18, 8, 0, *b(4343434343434343, 8)],

    # 'mov1 [a] [1]': [19, 1, 0, *addr(1)],
    # 'mov2 [a] [1230000]': [19, 2, 0, *addr(1230000)],
    # 'mov4 [a] [123]': [19, 4, 0, *addr(123)],
    # 'mov8 [a] [1230000]': [19, 8, 0, *addr(1230000)],

    # 'mov1 [1] a': [20, 1, *addr(1), 0],
    # 'mov2 [1234] b': [20, 2, *addr(1234), 1],
    # 'mov4 [1230000] a': [20, 4, *addr(1230000), 0],
    # 'mov8 [123456789010] b': [20, 8, *addr(123456789010), 1],

    # 'mov1 [1] [a]': [21, 1, *addr(1), 0],
    # 'mov2 [1234] [b]': [21, 2, *addr(1234), 1],
    # 'mov4 [1230000] [a]': [21, 4, *addr(1230000), 0],
    # 'mov8 [123456789010] [b]': [21, 8, *addr(123456789010), 1],

    # 'mov1 [1] 43': [22, 1, *addr(1), 43],
    # 'mov2 [1234] 4343': [22, 2, *addr(1234), *b(4343, 2)],
    # 'mov4 [1230000] 43434343': [22, 4, *addr(1230000), *b(43434343, 4)],
    # 'mov8 [123456789010] 4343434343434343': [22, 8, *addr(123456789010), *b(4343434343434343, 8)],

    # 'mov1 [1] [123]': [23, 1, *addr(1), *addr(123)],
    # 'mov2 [1234] [1230000]': [23, 2, *addr(1234), *addr(1230000)],
    # 'mov4 [1230000] [123456789010]': [23, 4, *addr(1230000), *addr(123456789010)],
    # 'mov8 [123456789010] [123]': [23, 8, *addr(123456789010), *addr(123)],

    # 'push a': [24, 0],
    # 'push b': [24, 1],

    # 'push1 [a]': [25, 1, 0],
    # 'push1 [b]': [25, 1, 1],
    # 'push2 [a]': [25, 2, 0],
    # 'push2 [b]': [25, 2, 1],
    # 'push4 [a]': [25, 4, 0],
    # 'push4 [b]': [25, 4, 1],
    # 'push8 [a]': [25, 8, 0],
    # 'push8 [b]': [25, 8, 1],

    # 'push1 123': [26, 1, *b(123, 1)],
    # 'push2 1234': [26, 2, *b(1234, 2)],
    # 'push4 12345678': [26, 4, *b(12345678, 4)],
    # 'push8 1234567890123456': [26, 8, *b(1234567890123456, 8)],

    # 'push1 [123]': [27, 1, *addr(123)],
    # 'push2 [1234]': [27, 2, *addr(1234)],
    # 'push4 [1230000]': [27, 4, *addr(1230000)],
    # 'push8 [123456789010]': [27, 8, *addr(123456789010)],

    # 'pop a': [28, 0],
    # 'pop b': [28, 1],

    # 'pop1 [a]': [29, 1, 0],
    # 'pop1 [b]': [29, 1, 1],
    # 'pop2 [a]': [29, 2, 0],
    # 'pop2 [b]': [29, 2, 1],
    # 'pop4 [a]': [29, 4, 0],
    # 'pop4 [b]': [29, 4, 1],
    # 'pop8 [a]': [29, 8, 0],
    # 'pop8 [b]': [29, 8, 1],

    # 'pop1 [123]': [30, 1, *addr(123)],
    # 'pop2 [1234]': [30, 2, *addr(1234)],
    # 'pop4 [1230000]': [30, 4, *addr(1230000)],
    # 'pop8 [123456789010]': [30, 8, *addr(123456789010)],

    # '@label': []

}


def main() -> None:

    success = True
    keep_files = '-k' in argv
    
    with open(TEST_FILE_NAME, "w") as f:
        for instruction, byte_code in INSTRUCTIONS.items():
            f.write(instruction + "\n")

    if subprocess.run(["./assembler.sh", TEST_FILE_NAME, "-o", OUTPUT_FILE_NAME]).returncode != 0:
        print("\nAssembler failed")
        exit(1)

    with open(OUTPUT_FILE_NAME, "rb") as f:
        byte_code = f.read()

    for instruction, expected_byte_code in INSTRUCTIONS.items():
        print(f"* Testing '{instruction}'...")

        if byte_code.startswith(bytes(expected_byte_code)):
            print("OK")
            byte_code = byte_code[len(expected_byte_code):]
            continue

        print("FAIL")
        print(f"Expected: {expected_byte_code}")
        decimal_bytes = [int(b) for b in byte_code[:len(expected_byte_code)]]
        print(f"Got:      {decimal_bytes}")
        success = False
        break

    if not keep_files:
        os.remove(TEST_FILE_NAME)
        os.remove(OUTPUT_FILE_NAME)

    if success:
        print("\nAll tests passed!")
    else:
        print("\nSome tests failed!")


if __name__ == "__main__":
    main()

