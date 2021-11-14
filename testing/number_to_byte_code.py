#!/usr/bin/env python3
from src.shared.byte_code import byte_code_names


def main() -> None:
    
    while True:
        try:
            code = input("Byte code: ").strip()
            if code == "":
                continue
            
            try:
                byte_code = int(code, 16)
                print(f" --> {byte_code_names[byte_code]}")
            except (ValueError, IndexError):
                print("Invalid byte code")

        except EOFError:
            break
        except KeyboardInterrupt:
            print()
            continue


if __name__ == "__main__":
    main()

