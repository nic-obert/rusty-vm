# Assembler

### TODO

- [ ] reimplement pseudo instructions like in-place define data. pseudo instructions could probably be included in the AsmInstructions enums and then evaluated when generating the bytecode  
- [ ] reimplement a parser for arrays  
- [ ] add the possibility to assemble a file without an entry point  
- [ ] add optional debug information in compiled binaries with -d assembler flag (and -d flag also in the vm to execute debug builds)  
- [ ] add compiletime arithmetics in the data section (maybe allow const arithmetic inside {})  

### In Progress


### Done ✓

- [x] put all include paths in quotes to make it easier to parse  
- [x] fix misrepresented float numbers. during tokenization, all numbers are converted to i64. here we need to use a union or an enum to distinguish between u64, i64, and f64  
- [x] eventually, upgrade the assembler to use a parsing tree to allow for more complex expressions (const expressions, for example)  
- [x] add sized operations for registers aloing with whole-register operations. for example PUSH_FROM_REG can have a twin like PUSH_FROM_REG_SIZED to specify how many bytes to push from the register. Not every reg instruction needs a sized counterpart, though. increment and decrement instructiond don't need a sized twin.  
- [x] reimplement the tokenizer using regex  
- [x] update eventual libraries and code to remove the stp register  

