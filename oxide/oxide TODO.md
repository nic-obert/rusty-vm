
### Todo

- the intermediate code Tn system should assign to Tns only once. Tns are worm variables. Really, though? It doesn't seem to be necessary
- create a README with the project description and the language features and syntax
- eventually, merge irc and flow analyzer modules since they work on the same stuff. Or maybe not
- detect infinite recursion during flow analysis
- add warnings for casting references to different-sized types. is_castable_to returns an enum that specifies why it cannot be cast
- check if variable is initialized in every path (may be hard to implement)
- vscode extension for syntax highlighting
- catch as many errors as possible in each compiler pass. in each pass, keep a boolean variable like has_errors and or it whenevere an error is encountered
- implement indirection operator .
- eventually, parallelize function IR code optimization since functions are independent from each other (only if the number of functions is significant)
- eventually, implement compiler directives
- eventually, add better type inference for array elements
- implement tuples
- main function and entry point
- lifetimes? they may require to structure the compiler in a totally different way. this may be a feature for a future compiler
- implement module system
- implement enums
- implement structs
- Add the no-op operator to the oxide language to prevent the optimizer from optimizing out conditional blocks and loops without any operation in them. Or we could use compiler directives
- Add the slice type &[T]. This is a wide pointer.
- Make StringRef a wide pointer.
- Ensure static value uniqueness in the symbol table. This would require implementing Eq and Hash (or some surrogate) for Number and DataType. The challenging part would be implementing Eq and hash for the float varients of Number
- Test the stack frame size of a function that contains another function definition in its body. Inner function definitions should not contribute to the stack frame size of the outer function.

### In Progress

- Devise a calling convention that takes into account registers and the stack. It must be the same for every function.
- implement IR code to bytecode translation
- flow analysis (using directed graphs) & optimization
- implement const function evaluation

### Done ✓

- Add integer type inference. Implicit numeric casts have been disallowed, but they may be allowed only if no type was specified and if they are literals
- compile intermediate code to multiple targets
- implement oprimizations
- add tests and compile-time assertions
- add compile-time array bounds check for known index operations
- indexing a reference to an array returns a reference to the element. maybe add a new SyntaxTokenValue like ArrayIndexRef { mutable, array, index }
- allow constant propagation for immutable statics
- the symbol table should have a reference to functions' bodies to evaluate constant function calls
- pushscope and popscope ir instructions should take into account removed stack variables
- merge && into &, || into |, and ! into ~ in the IR code. Boolean logic operators just flip the bits, so we could use bitwise operators instead
- change the _ matcher for a patten list when matching TokenKind and Ops when handling them in order to know when something isn't implemented
- take into account side effects of expressions when removing code in optimizations
- implement multi-line comments
- split the the TokenNode into lexical nodes and syntax nodes. different structs, different enums in order to have less clutter. No more ChildrenType, no more match_unreachable! to get the node children
- move operation arguments into ops/tokenkind enum??? the borrow checker would not be happy, though
- Wrap LiteralValues into Rc to allow passing them around. Note that they never get mutated and the symbol table holds some that are currently being copied to substitute their symbol if the value is known
- hide the debug printing to the console behind the -v flag
- static keyword
- disallow changing mutability when casting references
- remove effectless operations in ir code as an optimization
- declare optimizations/string representation/cli argument using a single macro
- differentiate between dereference used to assign and dereference used to get a value
- generate simplified intermediate code on which flow analysis is easier
- show line numbers in error messages and warnings
- add warnings for unreachable code after return statements
- disallow capturing outer scope symbols in functions declared inside other functions --> tests now need to be done to ensure it's working correctly
- add the do-while loop
- allow custom type definition within scopes with something like "type A = B;"
- make tokens and symbols aware of the source file they were declared in. make a struct that holds the unit path (&'a str), the line and column, and the source code (&'a IRCode).?????? would it be useful ???
- add warnings for unused symbols (new used falag)
- add break and continue
- implement const declaration
- since all active code must be found inside a function, divide the code into separate functions at the end of parse_block_hierarchy(). There will be a vector of functions for the active code and the symbol table will contain all the defined symbols and types
- benchmark cloning DataType::I32 vs DataType::Function to see if Rust optimizes small enum variants by not copying all 40 bytes. If this is the case, copying DataTypes should be ok because it would rarely copy 40 bytes and would most often copy just 8-16 bytes. if using Rc<DataType>, cloning always copies exactly 8 bytes, but there's the overhead of reference counting. Benchmark to see which is more convenient
- use Rc for DataType to avoid cloning
- make symbols aware of where in the source code they were declared (line and column)
- implement mutable and immutable references
- Use Cow for data types to avoid cloning
- allow literal values for immutable symbols in symbol table
- differentiate between string literals (stack) and heap-allocated strings
- coherce raw string literals to &str and store their value in a static section of the symbol table
- introduce usize and isize types
- add a -o flag to enable optimizations. optimizations should be disabled by default
- implement type inference for variable declaration. valid only if variable is immediately initialized.
- write a function like "reduce_operations()" to evaluate compile-time operations
- implement while and if
- add more source context in error messages (and highlight the main line)
- use dedicated ChildrenTypes for tokens that don't have strictly list children
- implement array indexing
- make scopes into expressions
