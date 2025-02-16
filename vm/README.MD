# Rusty Virtual Machine

# **OUTDATED**

- [Rusty Virtual Machine](#rusty-virtual-machine)
  - [Introduction](#introduction)
  - [Basic usage](#basic-usage)
  - [Registers](#registers)
  - [Instruction Set](#instruction-set)

## Introduction

This virtual machine is a stack-based processor with extended functionalities. It has a comprehensive instruction set and a small number of registers. It's designed to be simple to use and easy to understand.

The VM can interact with the host system through the use of system calls.
The VM also provides a built-in primitive memory allocator that does not require an operating system.

## Basic usage

To execute a program:

```bash
./vm my_program.bc
```

Run with the `--help` flag for full usage instructions.

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
| `stp`     | Stores the stack top pointer. |
| `sbp`     | Stores the stack base pointer. |
| `pc`      | Stores the program counter. |
| `zf`      | Zero flag. Set to 1 if the last operation returned 0. Set to 0 otherwise. |
| `sf`      | Sign flag. Set to 1 if the last operation returned a negative number. Set to 0 otherwise. |
| `rf`      | Remainder flag. Stores the remainder of the last operation. Set to zero if there was no remainder.<br>In floating point arithmetic, it's set to 1 if the result was `NaN`. Set to zero otherwise. |
| `cf`      | Carry flag. Set to 1 if the last operation had an overflowing carry bit. Set to zero otherwise.<br>In floating point arithmetic, it's set to 1 if the result was `+inf`. Set to zero otherwise. |
| `of`      | Overflow flag. Set to 1 if the last operation overflowed or underflowed. Set to zero otherwise.<br>In floating point arithmetic, it's set to 1 if the result was `-inf`. Set to zero otherwise. |

## Instruction Set

The virtual machine currently has 87 instructions. More may be added in the future.
For a full list of instructions, see the source file ['byte_code.rs'](../rusty_vm_lib/src/byte_code.rs).
The effect of each instruction is clearly shown in the source code file ['processor.rs'](src/processor.rs).
