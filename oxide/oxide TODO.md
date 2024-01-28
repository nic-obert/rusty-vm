# Oxide

Project Description

<em>[TODO.md spec & Kanban Board](https://bit.ly/3fCwKfM)</em>

### Todo

- [ ] implement mutable and immutable references  
- [ ] implement const functions  
- [ ] implement module system  
- [ ] implement enums  
- [ ] implement structs  

### In Progress

- [ ] add tests and compile-time assertions  
- [ ] implement oprimizations  

### Done ✓

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

