# Virtual Machine

### Todo

- [ ] implement a default built-in allocator for unlimited memory  
- [ ] add optional debug information in compiled binaries with -d assembler flag (and -d flag also in the vm to execute debug builds)  
- [ ] print byte operands when vm is in interactive mode (use expected operand size + handled size) (kind of a disassembly, wait till disassembler is done)  
- [ ] update the disassembler (when the vm and assembler become more stable)  
- [ ] build a byte code inspector (maybe extension for vscode or standalone program)  
- [ ] add compiletime arithmetics in the data section (maybe allow const arithmetic inside {})  

### In Progress ···

- [ ] implement a fixed size array in assembly  
- [ ] test the new vm  

### Done ✅

- [ ] write built-in asm terminal library  
- [ ] add library for boolean logic  
- [ ] update error documentation  
- [ ] add a modules struct to vm to contain additional modules like allocator, terminal, video  
- [ ] add basic io errors to the vm error codes  
- [ ] add disk operations (interrupts)  
- [ ] implement some kind of virtual disk (without a file system, since it should be managed by the os)  
- [ ] add an attach storage option to the vm  
- [ ] use assembler-like errors in vm without panicking  
- [ ] add colored output  
- [ ] add automatic jumping before dd  
- [ ] fix macro call print_char with space as argument (handled as 2 args instead of 1) (assembler.rs line 952)  
- [ ] debug the push intruction  
- [ ] rewrite all library functions using the new register state macros (beware of invalidated returns through registes since the previous register states get restored)  
- [ ] reimplement calloc using a function pattern  
- [ ] adapt procedures into self-countaned functions wrapped in a macro  
- [ ] implement data declaration inside .text  
- [ ] add programs written in asm  
- [ ] add inline macros  
- [ ] add support for floating point arithmetic  
- [ ] implement other more appropriate allocators  
- [ ] add stackoverflow checks when pushing  
- [ ] add .bss section  
- [ ] update memmove with new stack pointers  
- [ ] allow procedure calls inside macros  
- [ ] implement heap allocation interrupts  
- [ ] implement an allocator for allocating on the heap  
- [ ] move the stack to the end of program memory  
- [ ] write asm libraries to include or call  
- [ ] add intr instruction to execute an interrupt. replace current interrupts (except for exit) with macros  
- [ ] add macros  
- [ ] implement byte array data type (tests still to be performed)  
- [ ] add hex number constants  
- [ ] add instruction to push and pop the stack pointer n bytes (pushsp, popsp)  
- [ ] implement strncmp  
- [ ] add bitwise operations  
- [ ] differentiate between unsigned int input and signed int input  
- [ ] add usage to docs  
- [ ] implement atou1 (ascii to u1)  
- [ ] implement reexporting includes  
- [ ] add other jump instructions  
- [ ] add overflow flag  
- [ ] add sign flag  
- [ ] add reserved keywords and constant values like error codes  
- [ ] add printbytes  
- [ ] add interactive vm execution for debugging asm  
- [ ] substitute vectors for static arrays in arguments table for performance and safety  
- [ ] substitute panics for unreachable  
- [ ] improve argument errors by displaying which arguments are required by operators  
- [ ] remove sized operations from arguments table where size is not 8 bytes and operator is label  
- [ ] Add jump at address in register and jump at address literal bytecode instructions (and update arguments table to include jumping at addresses)  
- [ ] add compare at address bytecode instructions (and update argument table)  
- [ ] update documentation with new instructions  
- [ ] update documentation with new register names  
- [ ] implement standard assembly library  
- [ ] add include directive  
- [ ] add export directive  
- [ ] identify assembly unit in assembler error messages  
- [ ] add data section  
- [ ] implement program start in the vm (last 8 bytes in the binary)  
- [ ] add literal chars to asm code  
- [ ] rewrite the virtual machine in rust  
- add sized pop into register and remove unsized pop  
- add printc to print the unicode character stored in print  
- Add proper errors to disassembler  
- Add proper errors to assembler  
- refactor the project to have only one implementation of each component  
- rewrite the disassembler  
- rewrite the assembler  
- add verbose mode to assembler  
- add verbose mode to disassembler  

