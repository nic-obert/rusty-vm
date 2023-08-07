# Virtual Machine

### Todo

- [ ] substitute panics for unreachable  
- [ ] substitute vectors for static arrays in arguments table for performance and safety  
- [ ] update the disassembler  
- [ ] improve argument errors by displaying which arguments are required by operators  
- [ ] build a byte code inspector (maybe extension for vscode or standalone program)  
- [ ] add compiletime arithmetics in the data section  
- [ ] write asm libraries to include or call  
- [ ] add programs written in asm  
- [ ] test the new vm  

### In Progress ···


### Done ✅

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

