

A debugging format is required for the debugger to be able to read information about the program.
Such a debugging format shall be comprised of separate sections, each storing different kinds of information about the program. Sections shall be referenced in a section index table located at a known location, preferrably the start of the program memory.

The various secondary debug info sections may be stored at any location in the program binary. Thus, the start and end address of each section is required to locate them within the binary.

## Sections table

The sections table contains the start and end addresses of each debug info section.
Section ordering matters. If a section is omitted, its start and end addresses shall coincide.

"DEBUG SECTIONS\0"
[start][end] labels section
[start][end] instructions section
[start][end] source files section
[start][end] label names section


## Labels section

[label name][label address]
  8 bytes     8 bytes

The label name field is the address of the null-terminated string that represents the label name, found in the label names section.


## Label names section

This section contains the label names as null-terminated strings

## Source files section

This section contains the source file paths as null-terminated strings

## Instructions section

[program counter][source line][source file]
   8 bytes          8 bytes     8 bytes

The pc field is the address of the source instruction's first machine operation in the binary program. A source instruction may be comprised of multiple machine operations.
