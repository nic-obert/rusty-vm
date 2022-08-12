# **Virtual Machine**

A simple virtual machine and related tools created in Python (I'm currently translating it into C++).  
The toolchain includes:
- the virtual machine
- the assembler
- the disassembler
  
---

## Table of contents
- [**Virtual Machine**](#virtual-machine)
  - [Table of contents](#table-of-contents)
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
  - [Byte Code instructions](#byte-code-instructions)
    - [Arithmetical operations](#arithmetical-operations)
      - [`add`](#add-1)


<br>


## Assembly instructions

Every assembly intruction can be represented as a 1-byte integer code that identifies a set of operations to be performed by the virtual machine. The precise machine instruction it gets traslated to depends on its arguments.

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
Copy the value stored in 


## Byte Code instructions

These are the byte code instructions that the virtual machine executes. 
Each instruction is uniquely identified and represented by a 1-byte unisgned integer code.
The instruction arguments are stored in the memory following the instruction code.

### Arithmetical operations

#### `add`

(work in progress)
