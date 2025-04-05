# Assembler

### TODO

- [ ] add signed integer smaller than 8 bytes
- [ ] add 32-bit floats
- [ ] create a test script or rust tests to ensure correct behavior of the assembler
- [ ] eventually, add compiletime arithmetics like "1+1"

### In Progress


### Done ✓

- [x] add optional debug information in compiled binaries with -d assembler flag
- [x] implement a parser for complex array literals in static data declaration. Complex array literals may contain nested arrays, sized numbers, etc
- [x] add a pseudo instruction that perform constant arithmetics (or gives you the offset from a specific label to know the size of static data)
- [x] test unique symbol &
- [x] update documentation
- [x] test behavior with circular dependencies
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
