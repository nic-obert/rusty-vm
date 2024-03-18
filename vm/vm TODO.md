# Virtual Machine

### TODO

- [ ] probably the allocator doesn't belong here. A program should be written (like an assembly library) to handle heap memory allocation  
- [ ] test host_fs module (with copilot)  
- [ ] add colors to built-in terminal  
- [ ] add video module (separate process/thread that reads from video memory and writes to screen, don't worry about race conditions)  
- [ ] print byte operands when vm is in interactive mode (use expected operand size + handled size) (kind of a disassembly, wait till disassembler is done)  

### In Progress


### Done ✓

- [x] use only one stack pointer register. the other will be derived if necessary. this is to avoid setting both stack pointers when pushing the stack  

