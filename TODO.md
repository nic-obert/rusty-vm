# Virtual Machine

### Todo

- [ ] print byte operands when vm is in interactive mode (use expected operand size + handled size) (kind of a disassembly, wait till disassembler is done)  
- [ ] add support for floating point arithmetic  
- [ ] add bitwise operations  
- [ ] implement strncmp  
- [ ] add macros  
- [ ] update the disassembler  
- [ ] build a byte code inspector (maybe extension for vscode or standalone program)  
- [ ] add compiletime arithmetics in the data section  
- [ ] write asm libraries to include or call  
- [ ] add programs written in asm  
- [ ] test the new vm  

### In Progress ···


### Done ✅

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

