# **Easy Virtual Machine**

A simple virtual machine and related tools.  
The toolchain includes:

- the **virtual machine** (written in Rust)
- the **assembler** (written in Rust)
- the **disassembler** (written in Rust)
  
This is just an amateur project and should not be used in a production environment.
There are a few known vulnerabilities, plus it's not very efficient.

---

## Table of contents

- [**Easy Virtual Machine**](#easy-virtual-machine)
  - [Table of contents](#table-of-contents)
  - [Basic usage](#basic-usage)
    - [Assembler](#assembler)
    - [Disassembler](#disassembler)
    - [Virtual machine](#virtual-machine)
  - [Project structure](#project-structure)
  - [Registers](#registers)
  - [Assembly operators and symbols](#assembly-operators-and-symbols)
    - [$: current address](#-current-address)
    - [Address literals](#address-literals)
    - [Labels](#labels)
  - [Assembly instructions](#assembly-instructions)
    - [Arithmetical instructions](#arithmetical-instructions)
    - [No operation instructions](#no-operation-instructions)
    - [Memory instructions](#memory-instructions)
    - [Flow control instructions](#flow-control-instructions)
    - [Comparison instructions](#comparison-instructions)
    - [Logical bitwise instructions](#logical-bitwise-instructions)
    - [Interrupts](#interrupts)
  - [Assembly unit sections](#assembly-unit-sections)
    - [Data section](#data-section)
    - [Text section](#text-section)
    - [Include section](#include-section)
  - [Errors and error codes](#errors-and-error-codes)

## Basic usage

### Assembler

```bash
# Assemble a file
./assembler.sh my_file.asm
```

For full usage instructions, run with the `--help` flag.

### Disassembler

The disassembler is currently outdated and not working.

### Virtual machine

```bash
# Run a file
./vm.sh my_file.bc
```

For full usage instructions, run with the `--help` flag.

## Project structure

- The [`vm`](vm) directory contains the code for the virtual machine.
- The [`assembler`](assembler) directory contains the code for the assembler.
- [`assembler/lib`](assembler/lib) contains shared assembly libraries to include in the assembly source code.
- The [`disassembler`](disassembler) directory contains the code for the disassembler.
- The [`rust_vm_lib`](rust_vm_lib) directory contains the code for the shared library used across all Rust tools.
- The [`impl`](impl) directory contains examples of programs written in the VM's assembly language.
- The [`tests`](tests) directory contains tests for the VM tools.

## Registers

The virtual machine has 17 8-byte registers. Registers are identified by their name in the assembly code. In bytecode, they are identified by their 1-byte index in the `Registers` enum.

| Register  | Description                                                                 |
| -------   | --------------------------------------------------------------------------- |
| `r1`      | General purpose register. Also used for most built-in operations. |
| `r2`      | General purpose register. Also used for most built-in operations. |
| `r3`      | General purpose register. |
| `r4`      | General purpose register. |
| `r5`      | General purpose register. |
| `r6`      | General purpose register. |
| `r7`      | General purpose register. |
| `r8`      | General purpose register. |
| `exit`    | Stores the program's exit code. |
| `input`   | Stores the input from the console. |
| `error`   | Stores the last error code. |
| `print`   | Stores the value to print. |
| `sp`      | Stores the stack pointer. |
| `pc`      | Stores the program counter. |
| `zf`      | Stores the zero flag. |
| `sf`      | Stores the sign flag. |
| `rf`      | Stores the remainder flag. |
| `cf`      | Stores the carry flag. |
| `of`      | Stores the overflow flag. |

## Assembly operators and symbols

### $: current address

The `$` symbol represents the current address in the binary as it's being assembled.  
The assembler will replace every `$` symbol with the literal current address at the time of the assembly.

```asm
mov1 r1 $
```

### Address literals

Address literals are used to specify a memory address in the binary.
Address lierals are 8-byte unsigned integers enclosed in square brackets.

```asm
mov1 r1 [1234]
```

### Labels

A label is a compile time symbol that represents a memory address in the binary. Labels are declared in the assembly code using the `@` symbol. To export a label, prefix it with `@@` instead.  
Label names can only contain alphabetic characters and underscores. Also, they must not overwrite any of the reserved assembly instructions or registers.

```asm
# Regular label
@my_label

# Exported label
@@my_exported_label
```

## Assembly instructions

Every assembly intruction can be represented as a 1-byte integer code, internally identifie with the `ByteCodes` enum, that identifies a set of operations to be performed by the virtual machine. The precise machine instruction it gets traslated to depends on its arguments.

The first operand is treated as the destination by the processor, whereas the second operand is treated as the source.

### Arithmetical instructions

| Instruction | Description                                                                 |
| ----------- | ----------------------------------------------------------------------------------- |
| `add`       | Add the values stored in registers `r1` and `r2`. Store the result in register `r1`. Update the arithmetical flags. |
| `sub`       | Subtract the values stored in registers `r1` and `r2`. Store the result in register `r1`. Update the arithmetical flags. |
| `mul`       | Multiply the values stored in registers `r1` and `r2`. Store the result in register `r1`. Update the arithmetical flags. |
| `div`       | Divide the values stored in registers `r1` and `r2`. Store the result in register `r1`. Update the arithmetical flags. |
| `mod`       | Store the remainder of the division between the values stored in registers `r1` and `r2` in register `r1`. Update the arithmetical flags. |
| `inc a`     | Increment the value stored in the specified register `a`. Update the arithmetical flags. |
| `inc1 a`    | Increment the 1-byte value stored at `a`. Update the arithmetical flags. |
| `inc2 a`    | Increment the 2-byte value stored at `a`. Update the arithmetical flags. |
| `inc4 a`    | Increment the 4-byte value stored at `a`. Update the arithmetical flags. |
| `inc8 a`    | Increment the 8-byte value stored at `a`. Update the arithmetical flags. |
| `dec a`     | Decrement the value stored in the specified register `a`. Update the arithmetical flags. |
| `dec1 a`    | Decrement the 1-byte value stored at `a`. Update the arithmetical flags. |
| `dec2 a`    | Decrement the 2-byte value stored at `a`. Update the arithmetical flags. |
| `dec4 a`    | Decrement the 4-byte value stored at `a`. Update the arithmetical flags. |
| `dec8 a`    | Decrement the 8-byte value stored at `a`. Update the arithmetical flags. |

### No operation instructions

| Instruction | Description                                                                 |
| ----------- | ----------------------------------------------------------------------------------- |
| `nop`       | Do nothing for this cycle. |

### Memory instructions

| Instruction | Description                                                                 |
| ----------- | ----------------------------------------------------------------------------------- |
| `mov a b`   | Copy the second register `b` value into the first register `a`. |
| `mov1 a b`  | Copy 1 byte from `b` into the location `a`. |
| `mov2 a b`  | Copy 2 bytes from `b` into the location `a`. |
| `mov4 a b`  | Copy 4 bytes from `b` into the location `a`. |
| `mov8 a b`  | Copy 8 bytes from `b` into the location `a`. |
| `push a`    | Push the value stored in the specified register `a` onto the stack. |
| `push1 a`   | Push 1 bytes from `a` onto the stack. |
| `push2 a`   | Push 2 bytes from `a` onto the stack. |
| `push4 a`   | Push 4 bytes from `a` onto the stack. |
| `push8 a`   | Push 8 bytes from `a` onto the stack. |
| `pop1 a`    | Pop 1 byte from the top of the stack and store it at `a`. |
| `pop2 a`    | Pop 2 bytes from the top of the stack and store it at `a`. |
| `pop4 a`    | Pop 4 bytes from the top of the stack and store it at `a`. |
| `pop8 a`    | Pop 8 bytes from the top of the stack and store it at `a`. |

### Flow control instructions

| Instruction | Description                                                                 |
| ----------- | ----------------------------------------------------------------------------------- |
| `jmp a`     | Jump to the specified label `a`. |
| `jmpnz a`   | Jump to the specified label `a` if `zf` = zero. |
| `jmpz a`    | Jump to the specified label `a` if `zf` != zero. |
| `jmpgr a`   | Jump to the specified label `a` if `sf` = `of` and `zf` = 0. |
| `jmpge a`   | Jump to the specified label `a` if `sf` = `of`. |
| `jmplt a`   | Jump to the specified label `a` if `sf` != `of`. |
| `jmple a`   | Jump to the specified label `a` if `sf` != `of` or `zf` = 1. |
| `jmpof a`   | Jump to the specified label `a` if `of` = 1. |
| `jmpnof a`  | Jump to the specified label `a` if `of` = 0. |
| `jmpcr a`   | Jump to the specified label `a` if `cf` = 1. |
| `jmpncr a`  | Jump to the specified label `a` if `cf` = 0. |
| `jmpsn a`   | Jump to the specified label `a` if `sf` = 1. |
| `jmpnsn a`  | Jump to the specified label `a` if `sf` = 0. |
| `call a`    | Push the current `pc` onto the stack and jump to the specified label `a`. |
| `ret`       | Pop 8 bytes from the top of the stack and jump to the popped address. |

### Comparison instructions

| Instruction | Description                                                                 |
| ----------- | ----------------------------------------------------------------------------------- |
| `cmp a b`   | Compare the values stored in the specified registers. If the values are equal, set register `zf` to `1`. Else, set register `zf` to `0`. |
| `cmp1 a b`  | Compare 1 byte from `a` and `b`. If the values are equal, set register `zf` to `1`. Else, set register `zf` to `0`. |
| `cmp2 a b`  | Compare 2 bytes from `a` and `b`. If the values are equal, set register `zf` to `1`. Else, set register `zf` to `0`. |
| `cmp4 a b`  | Compare 4 bytes from `a` and `b`. If the values are equal, set register `zf` to `1`. Else, set register `zf` to `0`. |
| `cmp8 a b`  | Compare 8 bytes from `a` and `b`. If the values are equal, set register `zf` to `1`. Else, set register `zf` to `0`. |

### Logical bitwise instructions

| Instruction | Description                                                                 |
| ----------- | ----------------------------------------------------------------------------------- |
| `and`       | Perform a bitwise AND between the values stored in registers `r1` and `r2`. Store the result in register `r1`. Update the status flags. |
| `or`        | Perform a bitwise OR between the values stored in registers `r1` and `r2`. Store the result in register `r1`. Update the status flags. |
| `xor`       | Perform a bitwise XOR between the values stored in registers `r1` and `r2`. Store the result in register `r1`. Update the status flags. |
| `not`       | Perform a bitwise NOT on the value stored in register `r1`. Store the result in register `r1`. Update the status flags. |
| `shl`       | Perform a bitwise left shift on the value stored in register `r1` by the value stored in register `r2`. Store the result in register `r1`. |
| `shr`       | Perform a bitwise right shift on the value stored in register `r1` by the value stored in register `r2`. Store the result in register `r1`. |

### Interrupts

| Instruction | Description                                                                 |
| ----------- | ----------------------------------------------------------------------------------- |
| `printi`    | Print the signed integer value stored in the `print` register. |
| `printu`    | Print the unsigned integer value stored in the `print` register. |
| `printc`    | Print the unicode character stored in the `print` register. |
| `printstr`  | Print the string at the address stored in the `print` register. |
| `printbytes`| Print the bytes at the address stored in the `print` register up to the length stored in the `r1` register. |
| `inputsint`  | Get the next signed integer input from the console and store it in the `input` register. |
| `inputuint`  | Get the next unsigned integer input from the console and store it in the `input` register. |
| `inputstr`  | Get the next string input from the console, push it onto the stack, and store its address in the `input` register.<br>If the input is not a valid string, set `error` register to `INVALID_INPUT`.<br>If the EOF is encountered, set `error` register to `END_OF_FILE`.<br>If another error is encountered, set `error` register to `GENERIC_ERROR`.<br>If no error is encountered, set `error` register to `NO_ERROR`. |
| `exit`      | Exit the program with the exit code stored in the `exit` register. |

## Assembly unit sections

### Data section

The data section contains static data that can be accessed by the program.
The data section is declared in the assembly code using the `.data:` diretive.  
A static data declaration is composed of a label, a data type, and a value.

```asm
.data:
  my_string string "Hello, world!"
  my_char char 'a'
  number i4 42
```

Available data types are:

- `string`: a string literal
- `char`: a character literal
- `i1`: a signed 1-byte integer
- `i2`: a signed 2-byte integer
- `i4`: a signed 4-byte integer
- `i8`: a signed 8-byte integer
- `u1`: an unsigned 1-byte integer
- `u2`: an unsigned 2-byte integer
- `u4`: an unsigned 4-byte integer
- `u8`: an unsigned 8-byte integer

### Text section

The text section contains the bytecode instructions that will be executed by the virtual machine.

### Include section

The include section, declared with the `.include:` directive, is used to include other assembly files in the current assembly file. Assembly units are included in the order they are declared. Any assembly unit will be included only once in the final binary program.  
Assembly units to include are searched for in the same directory as the current assembly file.

```asm
.include:
  my_file.asm
```

## Errors and error codes

When the virtual machine encounters an error, it will set the `error` register to a specific error code. It's the programmer's responsibility to check the `error` register after fallible operations and handle eventual errors.  
An error code is represented as a 1-byte unsigned integer.

| Error Code      |Description                                                                 |
| ----------      | ----------------------------------------------------------------------------------- |
| `NO_ERROR`      | No error occurred. |
| `END_OF_FILE`   | End of file reached while reading input. |
| `INVALID_INPUT` | The input from the console was not a valid integer. |
| `ZERO_DIVISION` | A division by zero occurred. |
| `GENERIC_ERROR` | A generic error occurred. |
