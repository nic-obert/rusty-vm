
The new compiler will be remade from the start while keeping in mind the overall structure and requirements

This new compiler will have support for multiple modules from the start.
The module manager owns the source code
It will also be necessary to implement namespaces.
A namespace is an abstract named container for names.
Names can be symbols and modules. It will be necessary to discriminate between symbols and namespaces when compiling the code.
Namespaces could have a special store in the symbol table and a different SyntaxNodeValue
Import statements will look and behave like Rust:
```rust
use module_name;
use module_name::{symbol1, symbol2};
pub use module_name::{symbol};
```
the module manager will need to keep track of exported symbols, their signatures, and eventual static values.
The module manager will thus hold exported symbols as Rc<RefCell<Symbol<'a>>>, where 'a is the lifetime of the symbol table (and of the source code, which is the lifetime of the module manager).
Exported symbols need to be mutable in order to mark them as read.

Every module has its own symbol table.

```rust

enum Name<'a> {
    Symbol (&'a RefCell<Symbol<'a>>),
    NameSpace (&'a RefCell<NameSpace<'a>>)
}

struct Module<'a> {
    name: &'a str, // Owned by the source code
    symbol_table: SymbolTable<'a>, // Borrows symbol names from the source code
    source_code: String,
    source_lines: Box<[&'a str]>, // Owned by the source code
    exports: HashMap<&'a str, &'a Name<'a>>, // maps the exported symbol name to its symbol in the symbol table.
}
```

source code is parsed as a whole string, and not line-by-line.
the line-by-line approach may be used by a text preprocessor, though

This compiler may include compiler directives, if it sounds feasible

The compiler doesn't abort on the first error, but it continues on with the current pass to identify as many errors as possible.
Errors are collected in a vector and output all at once at the end of the pass.
By this approach, it's also possible to include the current compiler stage in the error message,
which is useful for debugging purposes and may be interesting for the programmer to see.


```rust
enum DataType {
    Bool,
    Char,
    Array { element_type: Rc<DataType>, length: Option<usize> },
    Slice,
    StringRef,
    RawString, ???
    Ref,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Usize,
    Isize,
    Void,
    Function { signature: FunctionSignature }
}
```

Data type implicit casts are to be strict.
Automatic type inference is instead a feature to work on.



Stages:

# initialize the module manager and load the source code
the source code is owned by the module manager.
the source code is logically split into lines.
`Box<[&'a str]>` is the indexable table of source code lines, used when printing errors to the console.
The source code lines table needs to be constructed before tokenization because printing errors requires the lines table to be completely initialized.

# maybe perform a prepass with the text preprocessor??.
this may implement macros and compiler directives.

# tokenize the source code into language-specific tokens
a SourceToken struct may be used and included in a Token wrapper struct
At this stage, a parsing priority is assigned to each token based on its type and location (inside delimiters or not)
Invalid tokens are identified now.

Do not use regex. regex may be slower than a language-specific algorithm.
The tokenizer runs in O(n) and does only one pass.
Using regex would require to first split the source code into matches and then iterate over the matches.
It's unknown which algorithm regex uses and, although optimized, you don't get any faster than a case-specific O(n) algorithm.
So, old school iter over all characters and gradually build the tokens

```rust
struct SourceToken<'a> {
    pub line: usize,
    pub column: usize,
    pub string: &'a str // Borrowed from the source code
}
```

Also, include statement separator tokens like `;` tokens.
statement separators are needed to know where to search for high-priority tokens and not go beyond statements

String literals are escaped, if any escape character is present. They will subsequently be
passed around as `Cow<'a, str>`, where `'a` is the lifetime of the source code.

# Construct an abstract syntax tree from the tokens based on the priority
Syntax errors should be caught here
tokens are converted into syntax nodes. Syntax nodes are also double linked list nodes and tree nodes.
Or we could use the OpenLinkedList crate. `OpenLinkedList<SyntaxNode>`
```rust
struct SyntaxNode<'a> {
    pub source: Rc<SourceToken<'a>>,
    pub next: *mut SyntaxNode<'a>,
    pub prev: *mut SyntaxNode<'a>,
    pub value: SyntaxNodeValue<'a>,
    /// The data type this expression node evaluates to.
    /// This type is to be considered uncertain before and during the type resolution phase.
    /// Symbols should access their type from the symbol table and update this field accordingly.
    pub inferred_type: Rc<DataType>,
    pub has_side_effects: bool
}

enum SyntaxNodeValue<'a> {
    Add { left: *mut SyntaxNode<'a>, right: *mut SyntaxNode<'a> },
    Block { content: Box<[*mut SyntaxNode<'a>]> }, // Or Vec<_>
    Symbol { symbol_id: SymbolID<'a> },
    ...
}
```

Symbols should be declared in the symbol table at this stage
Scopes are also initialized at this stage.
Each block node signals the beginning of a new scope.
let, const, and static keywords are removed here since the token has been declared.
const and static declarations must have an explicit type.
New types are also declared and defined here.
Functions are declared here as well.
Eventually, this will also apply to enums and structs.

At this point, imports will be evaluated.
import statements will load the imported module into the module manager and queue the new module for compilation.
Take care of avoiding duplicate modules.
Consider that:
- only top-level symbols can be exported
- top-level symbols can only be statics, consts, functions, types
- statics, consts, functions, and types must have an explicit signature (DataType)
- to be imported, symbols must be exported by some module
Because of this, we can assume that imported symbols are initialized and have a signature.
Thus, it's ok to import and use them before type inference has been performed on this module.


```rust
struct SymbolTable<'a> {
    statics: Vec<LiteralValue>, // Indexed by StaticID
    constants: Vec<?>,
    scopes: Vec<Scope> // Indexed by ScopeID
}

struct Symbol<'a> {
    pub source: Rc<SourceToken<'a>>,
    pub data_type: Rc<DataType>,
    pub symbol_value: SymbolValue,
    /// Whether the symbol is referenced anywhere for reading. Unread symbols may generate warnings and could be optimized out
    pub is_read: bool,
    pub is_public: bool, // ??? maybe it's not needed
    /// Whether the symbol has been optimized out
    pub removed: bool,
    /// It's considered an error to have uninitialized symbols
    pub initialized: bool,
    /// Function parameters may be handled differently by the bytecode generator because of stack frame stuff. See symbol_table.rs
    pub is_function_parameter: bool,
}

enum SymbolValue {
    Mutable,
    Immutable { value: Option<LiteralValue> },
    Constant { value: LiteralValue },
    Function {  },
    Static { mutable: bool, init_value: StaticID },

    UninitializedConstant,
    UninitializedStatic { mutable: bool },
}

struct NameSpace<'a> {
    inner_names: Vec<Name<'a>>,
}

struct Scope<'a> {
    symbols: HashMap<&'a str, RefCell<Symbol<'a>>>, // RefCell without Rc is better because symbols are owned by the symbol table. They will be passed around through &RefCell
    parent: Option<ScopeID>,
    types: HashMap<&'a str, TypeDef<'a>>, // Custom types
    children: Vec<ScopeID>,
    namespaces: Vec<RefCell<NameSpace<'a>>> // Namespaces should also be owned exclusively by the symbol table
}

struct SymbolID<'a> {
    name: &'a str,
    scope_id: ScopeID
}

struct StaticID (usize);

struct TypeDef<'a> {
    pub definition: Rc<DataType>,
    pub source: &SourceToken<'a>
}

struct ScopeID(usize);
```

No type checks are performed at this stage.
The building of the syntax tree determines the shape of the program, but not its functionality.
Only syntax checks are performed at this stage. E.g. `let i: i32 = 43;` is correct and `let :i = 43` is incorrect.
Some tokens will be dropped and won't correspond to any syntax node.
For example, a function declaration in the form
```rust
fn foo(a: i32, b: i32) -> i32 {
    a + b
}
```
will be represented as somethink like the following tree:
fn:
- name: foo
- params: [a, b]
- return: i32
- body: block:
        - +:
          - a
          - b

- function division
Split the code into functions
Iterate over the top nodes and isolate functions into separate structs

```rust
struct FunctionSignature {
    pub params: Box<[Rc<DataType>]>,
    pub return_type: Rc<DataType>
}

struct Function<'a> {
    pub signature: FunctionSignature,
    pub args: Box<[SymbolID]>,
    /// Root of the body tree
    pub body: SyntaxNode<'a>,
    pub is_marked_const: bool,
    pub has_side_effects: bool
}
```

It's necessary to identify which operations have side effects.
Operations with side effects cannot be removed by the optimizer.
see function_parser.rs for more info

Type inference may be done here as well.
Once the type inference passes are done, all the types are cemented, meaning any Unspecified data type becomes an explitit type.
Symbols that are marked as DataType::Unspecified can assume any type.
A type hint may be given by the literal value that is assigned to the symbol.
for example
```rust
// n: Unspecified(i32)
let n = 0;
// x: Unspecified(i32) because of operand n
let x = n + 1;

// The type of y is explicit, so it must overwrite the inferred type of x
// The new explicit type of x is registered in the symbol table.
// The next type inference pass will update the type of x
let y: u32 = x - 1;
```
literal values don't always have an inherent type. They have a suggested type, though.
For example, the literal number `43` may have a suggested type of `i32`

```rust

fn inc(a: u32) -> u32 {
    a + 1 // literal node 1 is inferred to be of type u32 because of a
}

// x: Unspecified(i32) -> u32
let x = 0;
// y: Unspecified(i32) -> u32
let y = 4;
// z: Unspecified(i32) -> u32, x and y are marked as u32
let z = x + y;
// res: u32, z is marked as u32
let res = inc(z);
// a: u32
let a: u32 = 3;
// b: Unspecified(i32) -> u32
let b = 1;
// c: u32, b is marked as u32 because a is u32
let c = a + b;
// d: u32 because c and z are u32
let d = c + z;
```
Type inference is performed in two linear passes:
- one forward pass, from the first statement to the last
- one reverse pass, from the last statement to the first
If types are found to not be the same, record the error

Type checks are to be performed at this stage, before any constant folding happens and after type inference.

Ensure functions marked const are actually constant.

if the optimization is enabled, calculate the constantness of every function
by analyzing its code tree. Functions that are found to be constant are marked as constant.


Optimizations available at this stage:
- constant folding (evaluate constant expressions and replace immutable symbols with their known literal value)
constant folding must be performed after the syntax trees are built because we have to ensure
that all symbols are declared before trying to access their eventual known literal value.
Constant functions with known arguments may be executed at this stage, including functions that were marked as const by the compiler

# intermediate code generation

Reduce the language operations to a small set of high-level instructions.
IR operators are a form of three address code.

Instead of using Tns, IRVar would be a better choice.

```rust
struct IRID(usize);

struct IRVar<'a> {
    id: IRID,
    var_type: IRVarType<'a>,
    data_type: Rc<DataType> // It's ok to store the data type here because data types are static
}

enum IRVarType<'a> {
    Temp,
    Symbol { symbol_id: SymbolID<'a> }
}
```

Temporary varaibles are used to "store" the result of expressions that have no logical store.
for example, the expression
```rust
let x = 1 + 2 + 3;
```
has will be transformed in the following tree:

assign:
 - x
 - add:
    - add:
        - 1
        - 2
    - 3
the result of the two add operations must be stored somewhere.
It's here that temporary ir variables come in handy
they provide an abstract store for intermediate operations that don't have a
logical store (symbols).

Having a reference to actual logical symbols will be needed
when generating the concrete bytecode because it's necessary to know
where to store the resulting values of operations.
Variables usually are assigned a location on the stack via an offset to the stack frame.
The compiler, however, may optimize out some varaibles or decide to store them in registers instead.
Handling registers, however, will be handled by the bytecode optimizer and is no concern for the ir generator.

the ir code is composed of a list of nodes.
```rust
struct IRNode {
    op: IROperator,
    has_side_effects: bool
}
```

optimizations available for ir:
- remove unread temporary variables and operations.
Reverse iteration over the nodes and remove those that are never read
Operations may have side effects. Operations with side effects won't be removed.
Side effects are determined prior to the ir code generation, when analyzing the syntax tree.


# program flow analysis

divide the ir code in basic blocks based on the ir instructions.
Labels introduce new basic blocks
Control flow instructions terminate the basic block

Connect the basic block into a graph. A flow graph is generated for every function.

possible optimizations at this stage:
- remove unreachable basic blocks
if a block doesn't have any refs and is not the first block of the function, it can be safely removed

- inline small basic blocks on unconditional jumps
When unconditionally jumping to a basic block, if the target block is small enough (small number of instructions)
we can inline the jump target and remove the jump instruction.

- known function inlining
When calling a known function (which is a special case of unconditional jumping), if the function is small enough,
we can inline the function and remove the call instruction.


# native bytecode generation

first, generate the static data section.
for every static variable in the symbol table, if it's been used (marked as read),
generate it in-place in the bytecode and map its static id to its address in the binary.
Ideally, immutable static values are unique to avoid duplication, which would result in larger binary sizes.

Generate the text section.
For every function, generate its bytecode
References to unnamed local static values are registered and will be generated later.
Register all unresolved labels and fill them in after the text section has been fully generated.
Any unresolved label will result in an unrecovarable error.

Temporary variables will be stored on the stack. However, registers may also be used as an optimization.


The compiler will then search for the main function and instruct the vm to start execution from there.

Possible optimizations at the bytecode generation stage:
- Avoid stack frame initialization
Determine if a function performs any call to other functions. If a function doesn't perform any further function call, there's no need to increment
the stack pointer in the prologue, since local stack variables are accessed through an offset from the stack frame base.
As a result, there's also no need to pop the stack frame when returning.

No optimizations can be performed after label address resolution, as any change to the bytecode may fuck up the addresses.
