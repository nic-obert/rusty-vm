# Assembler

### TODO

- [ ] update documentation  
- [ ] add a pseudo instruction that perform constant arithmetics (or gives you the offset from a specific label to know the size of static data)  
- [ ] test behavior with circular dependencies  
- [ ] implement a parser for complex array literals in static data declaration. Complex array literals may contain nested arrays, sized numbers, etc  
- [ ] add optional debug information in compiled binaries with -d assembler flag (and -d flag also in the vm to execute debug builds)  
- [ ] add compiletime arithmetics in the data section (maybe allow const arithmetic inside {})  

### In Progress


### Done ✓

- [x] test multiline strings (probably fails)  
- [x] chech validity of specified size in dn  
- [x] implement "[$]", see todo!() in parser.rs  
- [x] Allow macro arguments to be grouped together "!println_int [ADDR]" should group [ADDR] together instead of treating each token as a separate argument  
- [x] reimplement pseudo instructions like in-place define data. pseudo instructions could probably be included in the AsmInstructions enums and then evaluated when generating the bytecode  
- [x] put all include paths in quotes to make it easier to parse  
- [x] fix misrepresented float numbers. during tokenization, all numbers are converted to i64. here we need to use a union or an enum to distinguish between u64, i64, and f64  
- [x] eventually, upgrade the assembler to use a parsing tree to allow for more complex expressions (const expressions, for example)  
- [x] add sized operations for registers aloing with whole-register operations. for example PUSH_FROM_REG can have a twin like PUSH_FROM_REG_SIZED to specify how many bytes to push from the register. Not every reg instruction needs a sized counterpart, though. increment and decrement instructiond don't need a sized twin.  
- [x] reimplement the tokenizer using regex  
- [x] update eventual libraries and code to remove the stp register  

