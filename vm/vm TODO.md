# Virtual Machine

### TODO

- [ ] write a tool to automatically generate host_fs module library  
- [ ] test host_fs module (with copilot)  
- [ ] add colors to built-in terminal  
- [ ] add video module (separate process/thread that reads from video memory and writes to screen, don't worry about race conditions)  

### In Progress


### Done ✓

- [x] print byte operands when vm is in interactive mode (use expected operand size + handled size) (kind of a disassembly, wait till disassembler is done)  
- [x] Find a way to get a string input without using an allocator. Create a buffer and use it to get user input. The user input buffer is not unlimited, like real buffers. The VM has a "hardware" console input buffer. Interrupts can be used to read from the buffer (and write the bytes to a specified pointer.  
- [x] use only one stack pointer register. the other will be derived if necessary. this is to avoid setting both stack pointers when pushing the stack  

