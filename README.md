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
  - [Project structure](#project-structure)
  - [Registers](#registers)
  - [Assembly operators and symbols](#assembly-operators-and-symbols)
    - [$: current address](#-current-address)
    - [Address literals](#address-literals)
    - [Labels](#labels)
  - [Assembly instructions](#assembly-instructions)
    - [Arithmetical instructions](#arithmetical-instructions)
      - [`add`](#add)
      - [`sub`](#sub)
      - [`mul`](#mul)
      - [`div`](#div)
      - [`mod`](#mod)
      - [`inc`](#inc)
      - [`inc1`](#inc1)
      - [`inc2`](#inc2)
      - [`inc4`](#inc4)
      - [`inc8`](#inc8)
      - [`dec`](#dec)
      - [`dec1`](#dec1)
      - [`dec2`](#dec2)
      - [`dec4`](#dec4)
      - [`dec8`](#dec8)
    - [No operation instructions](#no-operation-instructions)
      - [`nop`](#nop)
    - [Memory instructions](#memory-instructions)
      - [`mov`](#mov)
      - [`mov1`](#mov1)
      - [`mov2`](#mov2)
      - [`mov4`](#mov4)
      - [`mov8`](#mov8)
      - [`push`](#push)
      - [`push1`](#push1)
      - [`push2`](#push2)
      - [`push4`](#push4)
      - [`push8`](#push8)
      - [`pop1`](#pop1)
      - [`pop2`](#pop2)
      - [`pop4`](#pop4)
      - [`pop8`](#pop8)
    - [Flow control instructions](#flow-control-instructions)
      - [`jmp`](#jmp)
      - [`jmpnz`](#jmpnz)
      - [`jmpz`](#jmpz)
    - [Comparison instructions](#comparison-instructions)
      - [`cmp`](#cmp)
      - [`cmp1`](#cmp1)
      - [`cmp2`](#cmp2)
      - [`cmp4`](#cmp4)
      - [`cmp8`](#cmp8)
    - [Interrupts](#interrupts)
      - [`sprint`](#sprint)
      - [`uprint`](#uprint)
      - [`printc`](#printc)
      - [`printstr`](#printstr)
      - [`inputint`](#inputint)
      - [`inputstr`](#inputstr)
      - [`exit`](#exit)
  - [Assembly unit sections](#assembly-unit-sections)
    - [Data section](#data-section)
    - [Text section](#text-section)
    - [Include section](#include-section)

## Project structure

- The [`vm`](vm) directory contains the code for the virtual machine.
- The [`assembler`](assembler) directory contains the code for the assembler.
- The [`disassembler`](disassembler) directory contains the code for the disassembler.
- The [`rust_vm_lib`](rust_vm_lib) directory contains the code for the shared library used across all Rust tools.
- The [`impl`](impl) directory contains examples of programs written in the VM's assembly language.
- The [`tests`](tests) directory contains tests for the VM tools.

## Registers

The virtual machine has 17 8-byte registers. Registers are identified by their name in the assembly code. In bytecode, they are identified by their 1-byte index in the `Registers` enum.

| Name    | Index | Description                                                                 |
| ------- | ----- | --------------------------------------------------------------------------- |
| `r1`    | 0     | General purpose register. Also used for most built-in operations. |
| `r2`    | 1     | General purpose register. Also used for most built-in operations. |
| `r3`    | 2     | General purpose register. |
| `r4`    | 3     | General purpose register. |
| `r5`    | 4     | General purpose register. |
| `r6`    | 5     | General purpose register. |
| `r7`    | 6     | General purpose register. |
| `r8`    | 7     | General purpose register. |
| `exit`  | 4     | Stores the program's exit code. |
| `input` | 5     | Stores the input from the console. |
| `error` | 6     | Stores the last error code. |
| `print` | 7     | Stores the value to print. |
| `sp`    | 8     | Stores the stack pointer. |
| `pc`    | 9     | Stores the program counter. |
| `zf`    | 10    | Stores the zero flag. |
| `sf`    | 11    | Stores the sign flag. |
| `rf`    | 12    | Stores the remainder flag. |

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

#### `add`

Add the values stored in registers `r1` and `r2`. Store the result in register `r1`.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```asm
add
```

#### `sub`

Subtract the values stored in registers `r1` and `r2`. Store the result in register `r1`.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```asm
sub
```

#### `mul`

Multiply the values stored in registers `r1` and `r2`. Store the result in register `r1`.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```asm
mul
```

#### `div`

Divide the values stored in registers `r1` and `r2`. Store the result in register `r1`.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.
Store the eventual integer remainder in register `rf`.

```asm
div
```

#### `mod`

Store the remainder of the division between the values stored in registers `r1` and `r2` in register `r1`.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```asm
mod
```

#### `inc`

Increment the value stored in the specified register.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```asm
inc r1
```

#### `inc1`

Increment the 1-byte value stored at the specified address.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```asm
inc1 [r1]
inc1 [1234]
```

#### `inc2`

Increment the 2-byte value stored at the specified address.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```asm
inc2 [r1]
inc2 [1234]
```

#### `inc4`

Increment the 4-byte value stored at the specified address.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```asm
inc4 [r1]
inc4 [1234]
```

#### `inc8`

Increment the 8-byte value stored at the specified address.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```asm
inc8 [r1]
inc8 [1234]
```

#### `dec`

Decrement the value stored in the specified register.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```asm
dec r1
```

#### `dec1`

Decrement the 1-byte value stored at the specified address.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```asm
dec1 [r1]
dec1 [1234]
```

#### `dec2`

Decrement the 2-byte value stored at the specified address.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```asm
dec2 [r1]
dec2 [1234]
```

#### `dec4`

Decrement the 4-byte value stored at the specified address.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```asm
dec4 [r1]
dec4 [1234]
```

#### `dec8`

Decrement the 8-byte value stored at the specified address.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```asm
dec8 [r1]
dec8 [1234]
```

### No operation instructions

#### `nop`

Do nothing for this cycle.

```asm
nop
```

### Memory instructions

#### `mov`

Copy the second register value into the first register.

```asm
mov r1 r2
```

#### `mov1`

Copy the 1-byte literal or value stored at the specified location into the specified location.

```asm
mov1 r1 [r2]
mov1 [r1] [123]
mov1 r1 123
mov1 [r1] r2
mov1 [r1] [r2]
mov1 [r1] [123]
mov1 [r1] 123
mov1 [123] r1
mov1 [123] [r1]
mov1 [123] 123
```

#### `mov2`

Copy the 2-byte literal or value stored at the specified location into the specified location.

```asm
mov2 r1 [r2]
mov2 [r1] [123]
mov2 r1 123
mov2 [r1] r2
mov2 [r1] [r2]
mov2 [r1] [123]
mov2 [r1] 123
mov2 [123] r1
mov2 [123] [r1]
mov2 [123] 123
```

#### `mov4`

Copy the 4-byte literal or value stored at the specified location into the specified location.

```asm
mov4 r1 [r2]
mov4 [r1] [123]
mov4 r1 123
mov4 [r1] r2
mov4 [r1] [r2]
mov4 [r1] [123]
mov4 [r1] 123
mov4 [123] r1
mov4 [123] [r1]
mov4 [123] 123
```

#### `mov8`

Copy the 8-byte literal or value stored at the specified location into the specified location.

```asm
mov8 r1 [r2]
mov8 [r1] [123]
mov8 r1 123
mov8 [r1] r2
mov8 [r1] [r2]
mov8 [r1] [123]
mov8 [r1] 123
mov8 [123] r1
mov8 [123] [r1]
mov8 [123] 123
```

#### `push`

Push the value stored in the specified register onto the stack.

```asm
push r1
```

#### `push1`

Push the 1-byte specified value (or the value stored at the specified address) onto the stack.

```asm
push1 [r1]
push1 [1234]
push1 43
```

#### `push2`

Push the 2-byte specified value (or the value stored at the specified address) onto the stack.

```asm
push2 [r1]
push2 [1234]
push2 43
```

#### `push4`

Push the 4-byte specified value (or the value stored at the specified address) onto the stack.

```asm
push4 [r1]
push4 [1234]
push4 43
```

#### `push8`

Push the 8-byte specified value (or the value stored at the specified address) onto the stack.

```asm
push8 [r1]
push8 [1234]
push8 43
```

#### `pop1`

Pop the 1-byte value from the top of the stack and store it in the specified address or register.

```asm
pop1 r1
pop1 [r1]
pop1 [1234]
```

#### `pop2`

Pop the 2-byte value from the top of the stack and store it in the specified address or register.

```asm
pop2 r1
pop2 [r1]
pop2 [1234]
```

#### `pop4`

Pop the 4-byte value from the top of the stack and store it in the specified address or register.

```asm
pop4 r1
pop4 [r1]
pop4 [1234]
```

#### `pop8`

Pop the 8-byte value from the top of the stack and store it in the specified address or register.

```asm
pop8 r1
pop8 [r1]
pop8 [1234]
```

### Flow control instructions

#### `jmp`

Jump to the specified label.

```asm
jmp label
```

#### `jmpnz`

Jump to the specified label if the specified register not zero.

```asm
jmpnz label r1
```

#### `jmpz`

Jump to the specified label if the specified register is zero.

```asm
jmpz label r1
```

### Comparison instructions

#### `cmp`

Compare the values stored in the specified registers.  
If the values are equal, set register `zf` to `1`.
Else, set register `zf` to `0`.

```asm
cmp r1 r2
```

#### `cmp1`

Compare the 1-byte values (literals or stored in the specified register).
If the values are equal, set register `zf` to `1`.
Else, set register `zf` to `0`.

```asm
cmp1 r1 14
cmp1 14 r1
cmp1 14 14
```

#### `cmp2`

Compare the 2-byte values (literals or stored in the specified register).
If the values are equal, set register `zf` to `1`.
Else, set register `zf` to `0`.

```asm
cmp2 r1 14
cmp2 14 r1
cmp2 14 14
```

#### `cmp4`

Compare the 4-byte values (literals or stored in the specified register).
If the values are equal, set register `zf` to `1`.
Else, set register `zf` to `0`.

```asm
cmp4 r1 14
cmp4 14 r1
cmp4 14 14
```

#### `cmp8`

Compare the 8-byte values (literals or stored in the specified register).
If the values are equal, set register `zf` to `1`.
Else, set register `zf` to `0`.

```asm
cmp8 r1 14
cmp8 14 r1
cmp8 14 14
```

### Interrupts

#### `sprint`

Print the signed integer value stored in the `print` register.

```asm
iprint
```

#### `uprint`

Print the unsigned integer value stored in the `print` register.

```asm
uprint
```

#### `printc`

Print the unicode character stored in the `print` register.

```asm
printc
```

#### `printstr`

Print the string at the address stored in the `print` register.

```asm
printstr
```

#### `inputint`

Get the next integer input from the console and store it in the `input` register.
If the input is not a valid integer, set `error` register to `INVALID_INPUT`.
If the EOF is encountered, set `error` register to `END_OF_FILE`.
If another error is encountered, set `error` register to `GENERIC_ERROR`.
If no error is encountered, set `error` register to `NO_ERROR`.

```asm
inputint
```

#### `inputstr`

Get the next string input from the console, push it onto the stack, and store its address in the `input` register.
If the input is not a valid string, set `error` register to `INVALID_INPUT`.
If the EOF is encountered, set `error` register to `END_OF_FILE`.
If another error is encountered, set `error` register to `GENERIC_ERROR`.
If no error is encountered, set `error` register to `NO_ERROR`.

```asm
inputstr
```

#### `exit`

Exit the program with the exit code stored in the `exit` register.

```asm
exit
```

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
