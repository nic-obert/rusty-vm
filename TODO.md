# Virtual Machine

### Project

- [ ] change project name to something better  
- [ ] build a byte code inspector (maybe extension for vscode or standalone program)  

### VM

- [ ] test host_fs module (with copilot)  
- [ ] add colors to built-in terminal  
- [ ] add video module (separate process/thread that reads from video memory and writes to screen, don't worry about race conditions)  
- [ ] implement a default built-in allocator for unlimited memory  
- [ ] print byte operands when vm is in interactive mode (use expected operand size + handled size) (kind of a disassembly, wait till disassembler is done)  

### Assembler

- [ ] add optional debug information in compiled binaries with -d assembler flag (and -d flag also in the vm to execute debug builds)  
- [ ] add compiletime arithmetics in the data section (maybe allow const arithmetic inside {})  

### Assembly

- [ ] implement a fixed size array in assembly  
- [ ] implement a linked list in assembly  

### Disassembler

- [ ] update the disassembler (when the vm and assembler become more stable)  

### IRC

- [ ] differentiate between string literals (stack) and heap-allocated strings  
- [ ] add conditional statements  

