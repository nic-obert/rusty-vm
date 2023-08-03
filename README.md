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
  - [Assembly operators and symbols](#assembly-operators-and-symbols)
    - [$: current address](#-current-address)
    - [Address literals](#address-literals)
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
      - [`label`](#label)
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
  - [Binary program structure](#binary-program-structure)
    - [Data section](#data-section)
    - [Text section](#text-section)
    - [Program start address](#program-start-address)

## Project structure

- The [`vm`](vm) directory contains the code for the virtual machine.
- The [`assembler`](assembler) directory contains the code for the assembler.
- The [`disassembler`](disassembler) directory contains the code for the disassembler.
- The [`rust_vm_lib`](rust_vm_lib) directory contains the code for the shared library used across all Rust tools.
- The [`impl`](impl) directory contains examples of programs written in the VM's assembly language.
- The [`tests`](tests) directory contains tests for the VM tools.

## Assembly operators and symbols

### $: current address

The `$` symbol represents the current address in the binary as it's being assembled.  
The assembler will replace every `$` symbol with the literal current address at the time of the assembly.

```asm
mov1 a $
```

### Address literals

Address literals are used to specify a memory address in the binary.
Address lierals are 8-byte unsigned integers enclosed in square brackets.

```asm
mov1 a [1234]
```

## Assembly instructions

Every assembly intruction can be represented as a 1-byte integer code, internally identifie with the `ByteCodes` enum, that identifies a set of operations to be performed by the virtual machine. The precise machine instruction it gets traslated to depends on its arguments.

The first operand is treated as the destination by the processor, whereas the second operand is treated as the source.

### Arithmetical instructions

#### `add`

Add the values stored in registers `a` and `b`. Store the result in register `a`.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```asm
add
```

#### `sub`

Subtract the values stored in registers `a` and `b`. Store the result in register `a`.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```asm
sub
```

#### `mul`

Multiply the values stored in registers `a` and `b`. Store the result in register `a`.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```asm
mul
```

#### `div`

Divide the values stored in registers `a` and `b`. Store the result in register `a`.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.
Store the eventual integer remainder in register `rf`.

```asm
div
```

#### `mod`

Store the remainder of the division between the values stored in registers `a` and `b` in register `a`.  
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
inc a
```

#### `inc1`

Increment the 1-byte value stored at the specified address.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```asm
inc1 [a]
inc1 [1234]
```

#### `inc2`

Increment the 2-byte value stored at the specified address.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```asm
inc2 [a]
inc2 [1234]
```

#### `inc4`

Increment the 4-byte value stored at the specified address.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```asm
inc4 [a]
inc4 [1234]
```

#### `inc8`

Increment the 8-byte value stored at the specified address.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```asm
inc8 [a]
inc8 [1234]
```

#### `dec`

Decrement the value stored in the specified register.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```asm
dec a
```

#### `dec1`

Decrement the 1-byte value stored at the specified address.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```asm
dec1 [a]
dec1 [1234]
```

#### `dec2`

Decrement the 2-byte value stored at the specified address.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```asm
dec2 [a]
dec2 [1234]
```

#### `dec4`

Decrement the 4-byte value stored at the specified address.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```asm
dec4 [a]
dec4 [1234]
```

#### `dec8`

Decrement the 8-byte value stored at the specified address.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```asm
dec8 [a]
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
mov a b
```

#### `mov1`

Copy the 1-byte literal or value stored at the specified location into the specified location.

```asm
mov1 a [b]
mov1 [a] [123]
mov1 a 123
mov1 [a] b
mov1 [a] [b]
mov1 [a] [123]
mov1 [a] 123
mov1 [123] a
mov1 [123] [a]
mov1 [123] 123
```

#### `mov2`

Copy the 2-byte literal or value stored at the specified location into the specified location.

```asm
mov2 a [b]
mov2 [a] [123]
mov2 a 123
mov2 [a] b
mov2 [a] [b]
mov2 [a] [123]
mov2 [a] 123
mov2 [123] a
mov2 [123] [a]
mov2 [123] 123
```

#### `mov4`

Copy the 4-byte literal or value stored at the specified location into the specified location.

```asm
mov4 a [b]
mov4 [a] [123]
mov4 a 123
mov4 [a] b
mov4 [a] [b]
mov4 [a] [123]
mov4 [a] 123
mov4 [123] a
mov4 [123] [a]
mov4 [123] 123
```

#### `mov8`

Copy the 8-byte literal or value stored at the specified location into the specified location.

```asm
mov8 a [b]
mov8 [a] [123]
mov8 a 123
mov8 [a] b
mov8 [a] [b]
mov8 [a] [123]
mov8 [a] 123
mov8 [123] a
mov8 [123] [a]
mov8 [123] 123
```

#### `push`

Push the value stored in the specified register onto the stack.

```asm
push a
```

#### `push1`

Push the 1-byte specified value (or the value stored at the specified address) onto the stack.

```asm
push1 [a]
push1 [1234]
push1 43
```

#### `push2`

Push the 2-byte specified value (or the value stored at the specified address) onto the stack.

```asm
push2 [a]
push2 [1234]
push2 43
```

#### `push4`

Push the 4-byte specified value (or the value stored at the specified address) onto the stack.

```asm
push4 [a]
push4 [1234]
push4 43
```

#### `push8`

Push the 8-byte specified value (or the value stored at the specified address) onto the stack.

```asm
push8 [a]
push8 [1234]
push8 43
```

#### `pop1`

Pop the 1-byte value from the top of the stack and store it in the specified address or register.

```asm
pop1 a
pop1 [a]
pop1 [1234]
```

#### `pop2`

Pop the 2-byte value from the top of the stack and store it in the specified address or register.

```asm
pop2 a
pop2 [a]
pop2 [1234]
```

#### `pop4`

Pop the 4-byte value from the top of the stack and store it in the specified address or register.

```asm
pop4 a
pop4 [a]
pop4 [1234]
```

#### `pop8`

Pop the 8-byte value from the top of the stack and store it in the specified address or register.

```asm
pop8 a
pop8 [a]
pop8 [1234]
```

### Flow control instructions

#### `label`

Declare a label whose name is the string after the `@` sign (in this case, "label").

```asm
@label
```

#### `jmp`

Jump to the specified label.

```asm
jmp label
```

#### `jmpnz`

Jump to the specified label if the specified register not zero.

```asm
jmpnz label a
```

#### `jmpz`

Jump to the specified label if the specified register is zero.

```asm
jmpz label a
```

### Comparison instructions

#### `cmp`

Compare the values stored in the specified registers.  
If the values are equal, set register `zf` to `1`.
Else, set register `zf` to `0`.

```asm
cmp a b
```

#### `cmp1`

Compare the 1-byte values (literals or stored in the specified register).
If the values are equal, set register `zf` to `1`.
Else, set register `zf` to `0`.

```asm
cmp1 a 14
cmp1 14 a
cmp1 14 14
```

#### `cmp2`

Compare the 2-byte values (literals or stored in the specified register).
If the values are equal, set register `zf` to `1`.
Else, set register `zf` to `0`.

```asm
cmp2 a 14
cmp2 14 a
cmp2 14 14
```

#### `cmp4`

Compare the 4-byte values (literals or stored in the specified register).
If the values are equal, set register `zf` to `1`.
Else, set register `zf` to `0`.

```asm
cmp4 a 14
cmp4 14 a
cmp4 14 14
```

#### `cmp8`

Compare the 8-byte values (literals or stored in the specified register).
If the values are equal, set register `zf` to `1`.
Else, set register `zf` to `0`.

```asm
cmp8 a 14
cmp8 14 a
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

## Binary program structure

A binary program is composed of three main sections: the data, the text, and the start address.

### Data section

The data section contains static data that can be accessed by the program. The data section should be immutable and should not be modified by the program.  
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

The text section contains the bytecode instructions that will be executed by the virtual machine. The text section should be immutable and should not be modified by the program.

### Program start address

The program start address is the address of the first instruction to be executed by the virtual machine. The start address should be immutable and should not be modified by the program.  
The program start address is automatically set by the assembler to the address if the first instruction in the `.text` section.
The program start address is equivalent to the main function in other programming languages.
