
The new compiler will be remade from the start while keeping in mind the overall structure and requirements

This new compiler will have support for multiple modules from the start.
The module manager owns the source code

source code is parsed as a whole string, and not line-by-line.
the line-by-line approach may be used by a text preprocessor, though

This compiler may include compiler directives, if it sounds feasable

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

- initialize the module manager and load the source code

- maybe perform a prepass with the text preprocessor??

- tokenize the source code into language-specific tokens
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
    pub string: &'a str
}
```

Also, include statement separator tokens like `;` tokens.
statement separators are needed to know where to search for high-priority tokens and not go beyond statements

A vector of the lines of source code is also constructed during tokenization.
`Vec<&'a str> -> Box<[&'a str]>`
This data structure will be used to find the source code lines by index when printing errors to the console

- Construct an abstract syntax tree from the tokens based on the priority
Syntax errors should be caught here
tokens are converted into syntax nodes. Syntax nodes are also double linked list nodes and tree nodes.

```rust
struct SyntaxNode<'a> {
    pub source: Rc<SourceToken<'a>>,
    pub next: *mut SyntaxNode<'a>,
    pub prev: *mut SyntaxNode<'a>,
    pub value: SyntaxNodeValue<'a>,
    /// The data type this expression node evaluates to.
    /// This type is to be considered uncertain before and during the type resolution phase.
    /// Symbols should access their type from the symbol table and update this field accordingly.
    pub inferred_type: Rc<DataType>
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

```rust
struct SymbolTable<'a> {
    statics: Vec<?>,
    constants: Vec<?>,
    scopes: Vec<Scope> // Indexed by ScopeID
}

struct Symbol<'a> {
    pub source: Rc<SourceToken<'a>>,
    pub is_public: bool,
    pub data_type: Rc<DataType>,
    pub is_read: bool,
    pub symbol_type: SymbolValue
}

enum SymbolType {
    Mutable,
    Immutable { value: Option<LiteralValue> },
    Constant { value: LiteralValue },
    Function { ?? },
    Static { mutable: bool, init_value: LiteralValue },

    UninitializedConstant,
    UninitializedStatic { mutable: bool },
}

struct Scope<'a> {
    symbols: HashMap<&'a str, Symbol<'a>>>,
    parent: Option<ScopeID>,
    types: HashMap<&'a str, TypeDef<'a>>, // Custom types
    children: Vec<ScopeID>
}

struct SymbolID<'a> {
    name: &'a str,
    scope_id: ScopeID
}

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
    pub is_marked_const: bool
}
```

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

- intermediate code
