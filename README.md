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
- [**Virtual Machine**](#virtual-machine)
  - [Table of contents](#table-of-contents)
  - [Project structure](#project-structure)
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


<br>


## Project structure

- The [`vm`](vm) directory contains the code for the virtual machine.
- The [`assembler`](assembler) directory contains the code for the assembler.
- The [`disassembler`](disassembler) directory contains the code for the disassembler.
- The [`rust_vm_lib`](rust_vm_lib) directory contains the code for the shared library used across all rust tools.
- The [`impl`](impl) directory contains examples of programs written in the VM's assembly language.
- The [`tests`](tests) directory contains tests for the VM tools.


## Assembly instructions

Every assembly intruction can be represented as a 1-byte integer code, internally identifie with the `ByteCodes` enum, that identifies a set of operations to be performed by the virtual machine. The precise machine instruction it gets traslated to depends on its arguments.

The first operand is treated as the destination by the processor, whereas the second operand is treated as the source.

### Arithmetical instructions

#### `add`
Add the values stored in registers `a` and `b`. Store the result in register `a`.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```
add
```

#### `sub`
Subtract the values stored in registers `a` and `b`. Store the result in register `a`.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```
sub
```

#### `mul`
Multiply the values stored in registers `a` and `b`. Store the result in register `a`.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```
mul
```

#### `div`
Divide the values stored in registers `a` and `b`. Store the result in register `a`.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.
Store the eventual integer remainder in register `rf`.

```
div
```

#### `mod`
Store the remainder of the division between the values stored in registers `a` and `b` in register `a`.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```
mod
```

#### `inc`
Increment the value stored in the specified register.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```
inc a
```

#### `inc1`
Increment the 1-byte value stored at the specified address.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```
inc1 [a]
inc1 [1234]
```

#### `inc2`
Increment the 2-byte value stored at the specified address.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```
inc2 [a]
inc2 [1234]
```

#### `inc4`
Increment the 4-byte value stored at the specified address.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```
inc4 [a]
inc4 [1234]
```

#### `inc8`
Increment the 8-byte value stored at the specified address.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```
inc8 [a]
inc8 [1234]
```

#### `dec`
Decrement the value stored in the specified register.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```
dec a
```

#### `dec1`
Decrement the 1-byte value stored at the specified address.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```
dec1 [a]
dec1 [1234]
```

#### `dec2`
Decrement the 2-byte value stored at the specified address.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```
dec2 [a]
dec2 [1234]
```

#### `dec4`
Decrement the 4-byte value stored at the specified address.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```
dec4 [a]
dec4 [1234]
```

#### `dec8`
Decrement the 8-byte value stored at the specified address.  
If the result is 0, set register `zf` to `1`.  
If the result is negative, set register `sf` to `1`.

```
dec8 [a]
dec8 [1234]
```

### No operation instructions

#### `nop`
Do nothing for this cycle.

```
nop
```

### Memory instructions

#### `mov`
Copy the second register value into the first register.

```
mov a b
```

#### `mov1`
Copy the 1-byte literal or value stored at the specified location into the specified location.

```
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

```
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

```
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

```
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

```
push a
```

#### `push1`
Push the 1-byte specified value (or the value stored at the specified address) onto the stack.

```
push1 [a]
push1 [1234]
push1 43
```

#### `push2`
Push the 2-byte specified value (or the value stored at the specified address) onto the stack.

```
push2 [a]
push2 [1234]
push2 43
```

#### `push4`
Push the 4-byte specified value (or the value stored at the specified address) onto the stack.

```
push4 [a]
push4 [1234]
push4 43
```

#### `push8`
Push the 8-byte specified value (or the value stored at the specified address) onto the stack.

```
push8 [a]
push8 [1234]
push8 43
```

#### `pop1`
Pop the 1-byte value from the top of the stack and store it in the specified address or register.

```
pop1 a
pop1 [a]
pop1 [1234]
```

#### `pop2`
Pop the 2-byte value from the top of the stack and store it in the specified address or register.

```
pop2 a
pop2 [a]
pop2 [1234]
```

#### `pop4`
Pop the 4-byte value from the top of the stack and store it in the specified address or register.

```
pop4 a
pop4 [a]
pop4 [1234]
```

#### `pop8`
Pop the 8-byte value from the top of the stack and store it in the specified address or register.

```
pop8 a
pop8 [a]
pop8 [1234]
```

### Flow control instructions

#### `label`
Declare a label whose name is the string after the `@` sign (in this case, "label").

```
@label
```

#### `jmp`
Jump to the specified label.

```
jmp label
```

#### `jmpnz`
Jump to the specified label if the specified register not zero.

```
jmpnz label a
```

#### `jmpz`
Jump to the specified label if the specified register is zero.

```
jmpz label a
```

### Comparison instructions

#### `cmp`
Compare the values stored in the specified registers.  
If the values are equal, set register `zf` to `1`.
Else, set register `zf` to `0`.

```
cmp a b
```

#### `cmp1`
Compare the 1-byte values (literals or stored in the specified register).
If the values are equal, set register `zf` to `1`.
Else, set register `zf` to `0`.

```
cmp1 a 14
cmp1 14 a
cmp1 14 14
```

#### `cmp2`
Compare the 2-byte values (literals or stored in the specified register).
If the values are equal, set register `zf` to `1`.
Else, set register `zf` to `0`.

```
cmp2 a 14
cmp2 14 a
cmp2 14 14
```

#### `cmp4`
Compare the 4-byte values (literals or stored in the specified register).
If the values are equal, set register `zf` to `1`.
Else, set register `zf` to `0`.

```
cmp4 a 14
cmp4 14 a
cmp4 14 14
```

#### `cmp8`
Compare the 8-byte values (literals or stored in the specified register).
If the values are equal, set register `zf` to `1`.
Else, set register `zf` to `0`.

```
cmp8 a 14
cmp8 14 a
cmp8 14 14
```

### Interrupts

#### `sprint`
Print the signed integer value stored in the `print` register.

```
print
```

#### `uprint`
Print the unsigned integer value stored in the `print` register.

```
uprint
```

#### `printc`
Print the unicode character stored in the `print` register.

```
printc
```

#### `printstr`
Print the string at the address stored in the `print` register.

```
printstr
```

#### `inputint`
Get the next integer input from the console and store it in the `input` register.
If the input is not a valid integer, set `error` register to `INVALID_INPUT`.
If the EOF is encountered, set `error` register to `END_OF_FILE`.
If another error is encountered, set `error` register to `GENERIC_ERROR`.
If no error is encountered, set `error` register to `NO_ERROR`.

```
inputint
```

#### `inputstr`
Get the next string input from the console, push it onto the stack, and store its address in the `input` register.
If the input is not a valid string, set `error` register to `INVALID_INPUT`.
If the EOF is encountered, set `error` register to `END_OF_FILE`.
If another error is encountered, set `error` register to `GENERIC_ERROR`.
If no error is encountered, set `error` register to `NO_ERROR`.

```
inputstr
```

#### `exit`
Exit the program with the exit code stored in the `exit` register.

```
exit
```

