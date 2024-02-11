# Oxide

Project Description

<em>[TODO.md spec & Kanban Board](https://bit.ly/3fCwKfM)</em>

### Todo

- [ ] disallow capturing outer scope symbols in functions declared inside other functions  
- [ ] add warnings for unreachable code after return statements  
- [ ] eventually, add option to parallelize ir code generation since every function is independent  
- [ ] implement tuples  
- [ ] main function  
- [ ] lifetimes?  
- [ ] move operation arguments into ops enum???  
- [ ] implement const functions  
- [ ] implement module system  
- [ ] implement enums  
- [ ] implement structs  

### In Progress

- [ ] generate simplified intermediate code on which flow analysis is easier  
- [ ] add tests and compile-time assertions  
- [ ] implement oprimizations  

### Done ✓

- [x] allow custom type definition within scopes with something like "type A = B;"  
- [x] make tokens and symbols aware of the source file they were declared in. make a struct that holds the unit path (&'a str), the line and column, and the source code (&'a IRCode).?????? would it be useful ???  
- [x] add warnings for unused symbols (new used falag)  
- [x] add break and continue  
- [x] implement const declaration  
- [x] since all active code must be found inside a function, divide the code into separate functions at the end of parse_block_hierarchy(). There will be a vector of functions for the active code and the symbol table will contain all the defined symbols and types  
- [x] benchmark cloning DataType::I32 vs DataType::Function to see if Rust optimizes small enum variants by not copying all 40 bytes. If this is the case, copying DataTypes should be ok because it would rarely copy 40 bytes and would most often copy just 8-16 bytes. if using Rc<DataType>, cloning always copies exactly 8 bytes, but there's the overhead of reference counting. Benchmark to see which is more convenient  
- [x] use Rc for DataType to avoid cloning  
- [x] make symbols aware of where in the source code they were declared (line and column)  
- [x] implement mutable and immutable references  
- [x] Use Cow for data types to avoid cloning  
- [x] allow literal values for immutable symbols in symbol table  
- [x] differentiate between string literals (stack) and heap-allocated strings  
- [x] coherce raw string literals to &str and store their value in a static section of the symbol table  
- [x] introduce usize and isize types  
- [x] add a -o flag to enable optimizations. optimizations should be disabled by default  
- [x] implement type inference for variable declaration. valid only if variable is immediately initialized.  
- [x] write a function like "reduce_operations()" to evaluate compile-time operations  
- [x] implement while and if  
- [x] add more source context in error messages (and highlight the main line)  
- [x] use dedicated ChildrenTypes for tokens that don't have strictly list children  
- [x] implement array indexing  
- [x] make scopes into expressions  

