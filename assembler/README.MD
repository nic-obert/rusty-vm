# Assembler

- [Assembler](#assembler)
  - [Introduction](#introduction)
  - [Basic usage](#basic-usage)
  - [Assembly language](#assembly-language)
    - [$: current address](#-current-address)
    - [\&: unique symbol](#-unique-symbol)
    - [Address literals](#address-literals)
    - [Labels](#labels)
    - [Sections](#sections)
      - [.text section](#text-section)
      - [.include section](#include-section)
    - [Function-like macros](#function-like-macros)
    - [Inline macros](#inline-macros)
  - [Assembly instructions](#assembly-instructions)
    - [Arithmetical instructions](#arithmetical-instructions)
    - [No operation instructions](#no-operation-instructions)
    - [Memory instructions](#memory-instructions)
    - [Flow control instructions](#flow-control-instructions)
    - [Comparison instructions](#comparison-instructions)
    - [Logical bitwise instructions](#logical-bitwise-instructions)
    - [Special intructions](#special-intructions)
    - [Interrupts](#interrupts)
    - [Pseudo instructions](#pseudo-instructions)
      - [Array data types](#array-data-types)
  - [Errors and error codes](#errors-and-error-codes)

## Introduction

This is an assembler for the Rusty Virtual Machine. It's designed to be simple to use and easy to understand, yet powerful enough to write complex programs.

## Basic usage

To assemble a file:

```bash
./assembler my_file.asm
```

For full usage instructions, run with the `--help` flag.

## Assembly language

### $: current address

The `$` symbol is a special label that represents the current address in the binary as it's being assembled.  
The assembler will replace every `$` symbol with the literal current address at the time of the assembly.  
Note that `$` has no knowledge of runtime stack pointers, so it's undefined behavior to use it inside procedures.

```bash
mov1 r1 $
```

### &: unique symbol

The `&` symbol represents a unique symbol that will be replaced with a unique text string at the time of the assembly.

```bash
# Define a unique label
@&
```


### Address literals

Address literals are used to specify a memory address in the binary.
Address lierals are 8-byte unsigned integers enclosed in square brackets.

```bash
mov1 r1 [1234]
```

### Labels

A label is a compile time symbol that represents a memory address in the binary. Labels are declared in the assembly code using the `@` symbol. To export a label, prefix it with `@@` instead.  
Label names can only contain alphabetic characters and underscores. Also, they must not overwrite any of the reserved assembly instructions or registers.

```bash
# Regular label
@my_label

# Exported label
@@my_exported_label
```

### Sections

A section is a distinct of the assembly program, identified by a label that points to the start of the section.

```bash
.my_section:
  # The assembly code here is inside of the .my_section section
```

Some section names are reserved for specific functions:

#### .text section

The `.text` section is treated as the entry point of the program. This means that the label `text` points to the first byte that will be executed by the VM.

#### .include section

The `.include` section is used to include other assembly units and uses a special syntax.  
The paths of the units to include are speficied as a string literal, optionally preceded by a `@@` token to re-export the included unit's contents.

```bash
.include:

  # Include the `archlib.asm` file
  "archlib.asm"

  # Include the `stdio.asm` file and re-export its contents
  @@ "stdio.asm"
```

Re-exporting symbols works as follows:
- unit `foo.asm` exports a `@@foo` label
- unit `bar.asm` includes and re-exports `foo.asm` via `@@ "foo.asm"`
- unit `foobar.asm` includes `bar.asm` and has access to the `foo` label

The assembly unit path resolution works as follows:
1. If the provided path is absolute, stop searching.
2. Check if the provided path is relative to the current working directory.
3. Check if the provided path is relative to any specified include paths.

Include paths are directories where the assembler searches for included units. Include paths can be specified through the `-L` option or via the `RUSTYVM_ASM_LIB` environment variable.

### Function-like macros

A function-like macro is a parametrized compile-time metaprogramming tool that allows you to generate assembly code in-place.
Function-like macros are declared thorugh the `%` prefix and invoked via the `!` operator.  
Macros work internally by replacing every invocation with the expanded macro body.

```bash
# Defining a macro
%my_macro:

  mov1 r1 1
  mov1 r2 2
  mov1 r3 3

# End of macro definition
%endmacro

# Invoking a macro
!my_macro
```

Macros may accept a number of positional parameters, specified after the macro name.  
The parameters are referenced in the macro body by enclosing their name in curly brackets: `{arg_name}`.

```bash
# Defining a macro with arguments
%my_macro arg1 arg2:

  mov1 r1 {arg1}
  mov1 r2 {arg2}

# End of macro definition
%endmacro

# Using a macro with arguments
!my_macro 1 2
```

To export a macro, prefix it with a double `%%` instead.

```bash
# Exporting a macro
%%my_macro:

  mov1 r1 1
  mov1 r2 2
  mov1 r3 3

# End of macro definition
%endmacro
```

### Inline macros

Inline macros are a non-parametrized compile-time metaprogramming tool that allows you to paste a stream of tokens in-place.  
Inline macros are declared in the assembly code through the `%-` prefix.
To expand a constant macro, prefix it with the `=` symbol.  

```bash
# Defining an inline macro
%-ZERO: 0

# Expanding an inline macro
mov1 r1 =ZERO

# Note that inline macros can be recursively expanded
%-r1_zero: r1 =ZERO

# This expands to `mov1 r1 0`
mov1 =r1_zero 
```

To export an inline macro, prefix it with `%%-` instead.

```bash
# Exporting an inline macro
%%-ZERO: 0
```

Inline macros are useful to define compile-time constants and they don't take up program space, unlike static data.  

## Assembly instructions

Every assembly intruction can be represented as a 1-byte integer code.

By convention, move-like operations treat the first argument as the destination and the second argument as the source.

### Arithmetical instructions

| Instruction | Description                                                                 |
| ----------- | ----------------------------------------------------------------------------------- |
| `iadd`      | Add the integer values stored in registers `r1` and `r2`. Store the result in register `r1`. Update the arithmetical flags. |
| `isub`      | Subtract the integer values stored in registers `r1` and `r2`. Store the result in register `r1`. Update the arithmetical flags. |
| `imul`      | Multiply the integer values stored in registers `r1` and `r2`. Store the result in register `r1`. Update the arithmetical flags. |
| `idiv`      | Divide the integer values stored in registers `r1` and `r2`. Store the result in register `r1`. Update the arithmetical flags. |
| `imod`      | Store the remainder of the integer division between the values stored in registers `r1` and `r2` in register `r1`. Update the arithmetical flags. |
| `fadd`      | Add the floating point values stored in registers `r1` and `r2`. Store the result in register `r1`. Update the arithmetical flags. |
| `fsub`      | Subtract the floating point values stored in registers `r1` and `r2`. Store the result in register `r1`. Update the arithmetical flags. |
| `fmul`      | Multiply the floating point values stored in registers `r1` and `r2`. Store the result in register `r1`. Update the arithmetical flags. |
| `fdiv`      | Divide the floating point values stored in registers `r1` and `r2`. Store the result in register `r1`. Update the arithmetical flags. |
| `fmod`      | Store the remainder of the floating point division between the values stored in registers `r1` and `r2` in register `r1`. Update the arithmetical flags. |
| `inc a`     | Increment the integer value stored in the specified register `a`. Update the arithmetical flags. |
| `inc1 a`    | Increment the 1-byte integer value stored at `a`. Update the arithmetical flags. |
| `inc2 a`    | Increment the 2-byte integer value stored at `a`. Update the arithmetical flags. |
| `inc4 a`    | Increment the 4-byte integer value stored at `a`. Update the arithmetical flags. |
| `inc8 a`    | Increment the 8-byte integer value stored at `a`. Update the arithmetical flags. |
| `dec a`     | Decrement the integer value stored in the specified register `a`. Update the arithmetical flags. |
| `dec1 a`    | Decrement the 1-byte integer value stored at `a`. Update the arithmetical flags. |
| `dec2 a`    | Decrement the 2-byte integer value stored at `a`. Update the arithmetical flags. |
| `dec4 a`    | Decrement the 4-byte integer value stored at `a`. Update the arithmetical flags. |
| `dec8 a`    | Decrement the 8-byte integer value stored at `a`. Update the arithmetical flags. |

### No operation instructions

| Instruction | Description                                                                 |
| ----------- | ----------------------------------------------------------------------------------- |
| `nop`       | Do nothing for this cycle. |

### Memory instructions

| Instruction | Description                                                                 |
| ----------- | ----------------------------------------------------------------------------------- |
| `mov a b`   | Copy the second register `b` value into the first register `a`. |
| `mov1 a b`  | Copy 1 byte from `b` into `a`. |
| `mov2 a b`  | Copy 2 bytes from `b` into `a`. |
| `mov4 a b`  | Copy 4 bytes from `b` into `a`. |
| `mov8 a b`  | Copy 8 bytes from `b` into `a`. |
| `push a`    | Push the value stored in the specified register `a` onto the stack. |
| `push1 a`   | Push 1 bytes from `a` onto the stack. |
| `push2 a`   | Push 2 bytes from `a` onto the stack. |
| `push4 a`   | Push 4 bytes from `a` onto the stack. |
| `push8 a`   | Push 8 bytes from `a` onto the stack. |
| `pushsp a`  | Increase the stack pointer by `a`. |
| `pushsp1 a` | Increase the stack pointer by the 1-byte value `a`. |
| `pushsp2 a` | Increase the stack pointer by the 2-byte value `a`. |
| `pushsp4 a` | Increase the stack pointer by the 4-byte value `a`. |
| `pushsp1 a` | Increase the stack pointer by the 8-byte value `a`. |
| `pop1 a`    | Pop 1 byte from the top of the stack and store it at `a`. |
| `pop2 a`    | Pop 2 bytes from the top of the stack and store it at `a`. |
| `pop4 a`    | Pop 4 bytes from the top of the stack and store it at `a`. |
| `pop8 a`    | Pop 8 bytes from the top of the stack and store it at `a`. |
| `popsp a`   | Decrease the stack pointer by `a`. |
| `popsp1 a`  | Decrease the stack pointer by the 1-byte value `a`. |
| `popsp2 a`  | Decrease the stack pointer by the 2-byte value `a`. |
| `popsp4 a`  | Decrease the stack pointer by the 4-byte value `a`. |
| `popsp8 a`  | Decrease the stack pointer by the 8-byte value `a`. |

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

### Special intructions

| Instruction | Description                                                                 |
| ----------- | ----------------------------------------------------------------------------------- |
| `intr`      | Trigger the interrupt with the interrupt code speficied in the `int` register. |
| `exit`      | Exit the program with the exit code stored in the `exit` register. |

### Interrupts

Interrupts are defined in the [`archlib.asm`](../asm_lib/archlib.asm) library file.

| Instruction | Description                                                                 |
| ----------- | ----------------------------------------------------------------------------------- |
| `PRINT_SIGNED`    | Print the signed integer value stored in the `print` register. |
| `PRINT_UNSIGNED`    | Print the unsigned integer value stored in the `print` register. |
| `PRINT_CHAR`    | Print the unicode character stored in the `print` register. |
| `PRINT_STRING`  | Print the string at the address stored in the `print` register. |
| `PRINT_BYTES`| Print the bytes at the address stored in the `print` register up to the length stored in the `r1` register. |
| `INPUT_SIGNED`  | Get the next signed integer input from the console and store it in the `input` register. |
| `INPUT_UNSIGNED`  | Get the next unsigned integer input from the console and store it in the `input` register. |
| `INPUT_STRING`  | Get the next string input from the console and allocate it on the heap. Store its address in the `input` register and its length in the `r1` register. The returned string is not to be considered null-terminated.<br>If the input is not a valid string, set `error` register to `INVALID_INPUT`.<br>If the EOF is encountered, set `error` register to `END_OF_FILE`.<br>If another error is encountered, set `error` register to `GENERIC_ERROR`.<br>If no error is encountered, set `error` register to `NO_ERROR`. |
| `RANDOM`          | Generate a random 8-byte number and store it in the `r1` register. |
| `HOST_TIME_NANOS` | Get the current host system time in nanoseconds and store it in the `r1` register. |
| `ELAPSED_TIME_NANOS` | Get the elapsed time since the program started in nanoseconds and store it in the `r1` register. |
| `DISK_READ`     | Read `r3` bytes from local storage at the address specified in `r1` and store them at the address specified in `r2`. If the read fails, set the `error` register. |
| `DISK_WRITE`    | Write `r3` bytes from the address specified in `r2` to local storage at the address specified in `r1`. If the write fails, set the `error` register. |
| `FLUSH_STDOUT`  | Flush the stdout buffer. |
... more to be implemented

### Pseudo instructions

Pseudo instructions are compile-time operators that affect the generated binary.

| Instruction                   | Description |
| ----------------------------- | ----------- |
| `dn <n> <number>`    | Define number. Insert an `n`-byte number in-place into the binary. Note that the only allowed sizes are 1, 2, 4, and 8. |
| `ds <string>`        | Define string. Insert a string in-place into the binary. Note that the assembler does not automatically a null-termination byte to string literals, so it's up to the programmer to correctly terminate the string according to the program needs. |
| `db <byte array>`    | Define bytes. Insert a byte array in-place into the binary. This is a shorthand form for `da u8 \<byte array>`. See the [Array data types](#Array-data-types) section. |
| `da <element type> <array>` | Define array. Insert an array in-place into the binary. |
| `offsetfrom <label>` | Offset from label. Calculate the memory offset in bytes between the current position and `label`, and insert the result in-place into the binary. Note that the resulting offset is an 8-byte unsigned integer and `label` must be defined prior to this instruction. |
| `printstr <string>` | Print a string literal declared in-place in the byte code. This instruction is equivalent to defining a string literal in-place and printing it. Note that the string does not need to be null-terminated. This instruction is useful for debugging purposes. |

#### Array data types

| Data type | Description |
| ---- | --- |
| `u8` | 1-byte unsigned integer |
| `u16` | 2-bytes unsigned integer |
| `u32` | 4-bytes unsigned integer |
| `u64` | 8-bytes unsigned integer |
| `i8` | 1-byte signed integer | 
| `i16` | 2-bytes signed integer |
| `i32` | 4-bytes signed integer |
| `i64` | 8-bytes signed integr |
| `f64` | 8-bytes float |
| `[<element type> : <length>]` | Array of `element type` with `length` elements. Example: `[i32:3]` is an array of 3 `i32` values. |

## Errors and error codes

When the virtual machine encounters an error, it will set the `error` register to a specific error code. It's the programmer's responsibility to check the `error` register after fallible operations and handle eventual errors.  
An error code is represented as a 1-byte unsigned integer.

| Error Code          | Description                                                                 |
| ----------          | ----------------------------------------------------------------------------------- |
| `NO_ERROR`          | No error occurred. |
| `END_OF_FILE`       | End of file reached while reading input. |
| `INVALID_INPUT`     | The input from the console was not a valid integer. |
| `ZERO_DIVISION`     | A division by zero occurred. |
| `STACK_OVERFLOW`    | The stack overflowed. |
| `OUT_OF_BOUNDS`     | An out of bounds memory access occurred. |
| `UNALIGNED_ADDRESS` | An unaligned memory access occurred. |
| `PERMISSION_DENIED` | A permission denied error occurred. |
| `TIMED_OUT`         | An operation timed out. |
| `NOT_FOUNT`         | A resource was not found. |
| `ALREADY_EXISTS`    | A resource already exists. |
| `INVALID_DATA`      | Provided data is invalid. |
| `INTERRUPTED`       | The process was interrupted. |
| `OUT_OF_MEMORY`     | The VM memory is either full or not enough to perform an operation. |
| `WRITE_ZERO`        | The last IO operation write 0 bytes. |
| `MODULE_UNAVAILABLE`| The specified VM module is not available in the current context. |
| `GENERIC_ERROR`     | A generic error occurred. |
