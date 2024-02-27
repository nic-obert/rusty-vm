use std::fmt::Display;
use std::rc::Rc;

use crate::lang::data_types::{DataType, LiteralValue};
use crate::symbol_table::{ScopeDiscriminant, ScopeID, SymbolTable};
use crate::tokenizer::{SourceToken, TokenParsingList};
use crate::utils::write_indent;


#[derive(Debug)]
pub enum RuntimeOp<'a> {

    MakeArray { elements: Vec<SyntaxNode<'a>> },
    Add { left: Box<SyntaxNode<'a>>, right: Box<SyntaxNode<'a>> },
    Sub { left: Box<SyntaxNode<'a>>, right: Box<SyntaxNode<'a>> },
    Mul { left: Box<SyntaxNode<'a>>, right: Box<SyntaxNode<'a>> },
    Div { left: Box<SyntaxNode<'a>>, right: Box<SyntaxNode<'a>> },
    Mod { left: Box<SyntaxNode<'a>>, right: Box<SyntaxNode<'a>> },
    Assign { left: Box<SyntaxNode<'a>>, right: Box<SyntaxNode<'a>> },
    Deref { mutable: bool, expr: Box<SyntaxNode<'a>> },
    Ref { mutable: bool, expr: Box<SyntaxNode<'a>> },
    Call { callable: Box<SyntaxNode<'a>>, args: Vec<SyntaxNode<'a>> },
    Return(Option<Box<SyntaxNode<'a>>>),
    Equal { left: Box<SyntaxNode<'a>>, right: Box<SyntaxNode<'a>> },
    NotEqual { left: Box<SyntaxNode<'a>>, right: Box<SyntaxNode<'a>> },
    Greater { left: Box<SyntaxNode<'a>>, right: Box<SyntaxNode<'a>> },
    Less { left: Box<SyntaxNode<'a>>, right: Box<SyntaxNode<'a>> },
    GreaterEqual { left: Box<SyntaxNode<'a>>, right: Box<SyntaxNode<'a>> },
    LessEqual { left: Box<SyntaxNode<'a>>, right: Box<SyntaxNode<'a>> },
    LogicalAnd { left: Box<SyntaxNode<'a>>, right: Box<SyntaxNode<'a>> },
    LogicalOr { left: Box<SyntaxNode<'a>>, right: Box<SyntaxNode<'a>> },
    BitShiftLeft { left: Box<SyntaxNode<'a>>, right: Box<SyntaxNode<'a>> },
    BitShiftRight { left: Box<SyntaxNode<'a>>, right: Box<SyntaxNode<'a>> },
    BitwiseOr { left: Box<SyntaxNode<'a>>, right: Box<SyntaxNode<'a>> },
    BitwiseAnd { left: Box<SyntaxNode<'a>>, right: Box<SyntaxNode<'a>> },
    BitwiseXor { left: Box<SyntaxNode<'a>>, right: Box<SyntaxNode<'a>> },
    LogicalNot(Box<SyntaxNode<'a>>),
    BitwiseNot(Box<SyntaxNode<'a>>),
    Break,
    Continue,
    ArrayIndex { array: Box<SyntaxNode<'a>>, index: Box<SyntaxNode<'a>> },

}

impl RuntimeOp<'_> {

    /// Assumes the operation is allowed at compile-time.
    /// Assumes the operands are literals.
    pub fn execute(&self, scope_id: ScopeID, symbol_table: &SymbolTable) -> Result<Rc<LiteralValue>, &'static str> {

        match self {
            RuntimeOp::Add { left, right } => {
                let left_value = left.known_literal_value(scope_id, symbol_table).unwrap();
                let left = left_value.assume_numeric();
                let right_value = right.known_literal_value(scope_id, symbol_table).unwrap();
                let right = right_value.assume_numeric();
                Ok(LiteralValue::Numeric(left.add(right)).into())
            },
            RuntimeOp::Sub { left, right } => {
                let left_value = left.known_literal_value(scope_id, symbol_table).unwrap();
                let left = left_value.assume_numeric();
                let right_value = right.known_literal_value(scope_id, symbol_table).unwrap();
                let right = right_value.assume_numeric();
                Ok(LiteralValue::Numeric(left.sub(right)).into())
            },
            RuntimeOp::Mul { left, right } => {
                let left_value = left.known_literal_value(scope_id, symbol_table).unwrap();
                let left = left_value.assume_numeric();
                let right_value = right.known_literal_value(scope_id, symbol_table).unwrap();
                let right = right_value.assume_numeric();
                Ok(LiteralValue::Numeric(left.mul(right)).into())
            },
            RuntimeOp::Div { left, right } => {
                let left_value = left.known_literal_value(scope_id, symbol_table).unwrap();
                let left = left_value.assume_numeric();
                let right_value = right.known_literal_value(scope_id, symbol_table).unwrap();
                let right = right_value.assume_numeric();
                match left.div(right) {
                    Ok(result) => Ok(LiteralValue::Numeric(result).into()),
                    Err(()) => Err("Division by zero")
                }
            },
            RuntimeOp::Mod { left, right } => {
                let left_value = left.known_literal_value(scope_id, symbol_table).unwrap();
                let left = left_value.assume_numeric();
                let right_value = right.known_literal_value(scope_id, symbol_table).unwrap();
                let right = right_value.assume_numeric();
                match left.modulo(right) {
                    Ok(result) => Ok(LiteralValue::Numeric(result).into()),
                    Err(()) => Err("Division by zero")
                }
            },
            RuntimeOp::Equal { left, right } => {
                let left = left.known_literal_value(scope_id, symbol_table).unwrap();
                let right = right.known_literal_value(scope_id, symbol_table).unwrap();
                Ok(LiteralValue::Bool(left.equal(&right)).into())
            },
            RuntimeOp::NotEqual { left, right } => {
                let left = left.known_literal_value(scope_id, symbol_table).unwrap();
                let right = right.known_literal_value(scope_id, symbol_table).unwrap();
                Ok(LiteralValue::Bool(!left.equal(&right)).into())
            },
            RuntimeOp::Greater { left, right } => {
                let left_value = left.known_literal_value(scope_id, symbol_table).unwrap();
                let left = left_value.assume_numeric();
                let right_value = right.known_literal_value(scope_id, symbol_table).unwrap();
                let right = right_value.assume_numeric();
                Ok(LiteralValue::Bool(left.greater(right)).into())
            },
            RuntimeOp::Less { left, right } => {
                let left_value = left.known_literal_value(scope_id, symbol_table).unwrap();
                let left = left_value.assume_numeric();
                let right_value = right.known_literal_value(scope_id, symbol_table).unwrap();
                let right = right_value.assume_numeric();
                Ok(LiteralValue::Bool(left.less(right)).into())
            },
            RuntimeOp::GreaterEqual { left, right } => {
                let left_value = left.known_literal_value(scope_id, symbol_table).unwrap();
                let left = left_value.assume_numeric();
                let right_value = right.known_literal_value(scope_id, symbol_table).unwrap();
                let right = right_value.assume_numeric();
                Ok(LiteralValue::Bool(left.greater_equal(right)).into())
            },
            RuntimeOp::LessEqual { left, right } => {
                let left_value = left.known_literal_value(scope_id, symbol_table).unwrap();
                let left = left_value.assume_numeric();
                let right_value = right.known_literal_value(scope_id, symbol_table).unwrap();
                let right = right_value.assume_numeric();
                Ok(LiteralValue::Bool(left.less_equal(right)).into())
            },
            RuntimeOp::LogicalAnd { left, right } => {
                let left = left.known_literal_value(scope_id, symbol_table).unwrap().assume_bool();
                let right = right.known_literal_value(scope_id, symbol_table).unwrap().assume_bool();
                Ok(LiteralValue::Bool(left && right).into())
            },
            RuntimeOp::LogicalOr { left, right } => {
                let left = left.known_literal_value(scope_id, symbol_table).unwrap().assume_bool();
                let right = right.known_literal_value(scope_id, symbol_table).unwrap().assume_bool();
                Ok(LiteralValue::Bool(left || right).into())
            },
            RuntimeOp::BitShiftLeft { left, right } => {
                let left_value = left.known_literal_value(scope_id, symbol_table).unwrap();
                let left = left_value.assume_numeric();
                let right_value = right.known_literal_value(scope_id, symbol_table).unwrap();
                let right = right_value.assume_numeric();
                Ok(LiteralValue::Numeric(left.bitshift_left(right)).into())
            },
            RuntimeOp::BitShiftRight { left, right } => {
                let left_value = left.known_literal_value(scope_id, symbol_table).unwrap();
                let left = left_value.assume_numeric();
                let right_value = right.known_literal_value(scope_id, symbol_table).unwrap();
                let right = right_value.assume_numeric();
                Ok(LiteralValue::Numeric(left.bitshift_right(right)).into())
            },
            RuntimeOp::BitwiseOr { left, right } => {
                let left_value = left.known_literal_value(scope_id, symbol_table).unwrap();
                let left = left_value.assume_numeric();
                let right_value = right.known_literal_value(scope_id, symbol_table).unwrap();
                let right = right_value.assume_numeric();
                Ok(LiteralValue::Numeric(left.bitwise_or(right)).into())
            },
            RuntimeOp::BitwiseAnd { left, right } => {
                let left_value = left.known_literal_value(scope_id, symbol_table).unwrap();
                let left = left_value.assume_numeric();
                let right_value = right.known_literal_value(scope_id, symbol_table).unwrap();
                let right = right_value.assume_numeric();
                Ok(LiteralValue::Numeric(left.bitwise_and(right)).into())
            },
            RuntimeOp::BitwiseXor { left, right } => {
                let left_value = left.known_literal_value(scope_id, symbol_table).unwrap();
                let left = left_value.assume_numeric();
                let right_value = right.known_literal_value(scope_id, symbol_table).unwrap();
                let right = right_value.assume_numeric();
                Ok(LiteralValue::Numeric(left.bitwise_xor(right)).into())
            },
            RuntimeOp::LogicalNot(operand) => {
                let operand = operand.known_literal_value(scope_id, symbol_table).unwrap().assume_bool();
                Ok(LiteralValue::Bool(!operand).into())
            },
            RuntimeOp::BitwiseNot(operand) => {
                let operand_value = operand.known_literal_value(scope_id, symbol_table).unwrap();
                let operand = operand_value.assume_numeric();
                Ok(LiteralValue::Numeric(operand.bitwise_not()).into())
            },
            RuntimeOp::ArrayIndex { array, index } => {
                let index = index.known_literal_value(scope_id, symbol_table).unwrap().assume_numeric().assume_uint();
                let array_value = array.known_literal_value(scope_id, symbol_table).unwrap();
                let (_data_type, elements) = array_value.assume_array();
                
                if index as usize >= elements.len() {
                    return Err("Index out of bounds");
                }

                Ok(elements[index as usize].clone())
            },
            
            RuntimeOp::MakeArray { elements: _ } => todo!(),
            
            RuntimeOp::Assign { .. } |
            RuntimeOp::Deref { .. } |
            RuntimeOp::Ref { .. } |
            RuntimeOp::Call { .. } |
            RuntimeOp::Return(_) |
            RuntimeOp::Break |
            RuntimeOp::Continue
                => unreachable!("Operation {:?} is not allowed at compile-time, but the compiler is trying to execute it nonetheless. This is a bug.", self),
        }
    }


    pub const fn name(&self) -> &'static str {

        macro_rules! op_names {
            ($($op:ident),*) => {
                match self {
                    $(RuntimeOp::$op { .. } => stringify!($op),)*
                }
            }
        }

        op_names! {
            MakeArray,
            Add,
            Sub,
            Mul,
            Div,
            Mod,
            Assign,
            Deref,
            Ref,
            Call,
            Return,
            Equal,
            NotEqual,
            Greater,
            Less,
            GreaterEqual,
            LessEqual,
            LogicalAnd,
            LogicalOr,
            BitShiftLeft,
            BitShiftRight,
            BitwiseOr,
            BitwiseAnd,
            BitwiseXor,
            LogicalNot,
            BitwiseNot,
            ArrayIndex,
            Break,
            Continue
        }
    }

    pub fn returns_a_value(&self) -> bool {
        match self {
            RuntimeOp::MakeArray { .. } |
            RuntimeOp::Add { .. } |
            RuntimeOp::Sub { .. } |
            RuntimeOp::Mul { .. } |
            RuntimeOp::Div { .. } |
            RuntimeOp::Mod { .. } |
            RuntimeOp::Deref { .. } |
            RuntimeOp::Call { .. } |
            RuntimeOp::Ref { .. } |
            RuntimeOp::Equal { .. } |
            RuntimeOp::NotEqual { .. } |
            RuntimeOp::Greater { .. } |
            RuntimeOp::Less { .. } |
            RuntimeOp::GreaterEqual { .. } |
            RuntimeOp::LessEqual { .. } |
            RuntimeOp::LogicalAnd { .. } |
            RuntimeOp::LogicalOr { .. } |
            RuntimeOp::BitShiftLeft { .. } |
            RuntimeOp::BitShiftRight { .. } |
            RuntimeOp::BitwiseOr { .. } |
            RuntimeOp::BitwiseAnd { .. } |
            RuntimeOp::BitwiseXor { .. } |
            RuntimeOp::LogicalNot(_) |
            RuntimeOp::BitwiseNot(_) |
            RuntimeOp::ArrayIndex { .. } 
                => true,

            RuntimeOp::Assign { .. } |
            RuntimeOp::Return(_) |
            RuntimeOp::Break |
            RuntimeOp::Continue
                => false,
        }
    }

}


#[derive(Debug)]
pub enum SyntaxNodeValue<'a> {

    RuntimeOp(RuntimeOp<'a>),
    FunctionParams(Vec<FunctionParam<'a>>),
    DataType(Rc<DataType>),
    Function { name: &'a str, signature: Rc<DataType>, body: ScopeBlock<'a> },
    As { target_type: Rc<DataType>, expr: Box<SyntaxNode<'a>> },
    IfChain { if_blocks: Vec<IfBlock<'a>>, else_block: Option<ScopeBlock<'a>> },
    While { condition: Box<SyntaxNode<'a>>, body: ScopeBlock<'a> },
    Loop { body: ScopeBlock<'a> },
    DoWhile { body: ScopeBlock<'a>, condition: Box<SyntaxNode<'a>> },
    Scope(ScopeBlock<'a>),
    Symbol { name: &'a str, scope_discriminant: ScopeDiscriminant },
    Literal(Rc<LiteralValue>),
    
    Const { name: &'a str, data_type: Rc<DataType>, definition: Box<SyntaxNode<'a>> },
    Static { name: &'a str, data_type: Rc<DataType>, definition: Box<SyntaxNode<'a>> },
    TypeDef { name: &'a str, definition: Rc<DataType> },

    /// A placeholder value used to satisfy Rust's no-uninitalized-fields rule
    /// This value should never be used in any other context.
    Placeholder,

}

impl SyntaxNodeValue<'_> {

    pub const fn name(&self) -> &'static str {
        
        match self {
            SyntaxNodeValue::RuntimeOp(op) => op.name(),
            SyntaxNodeValue::FunctionParams(_) => "FunctionParams",
            SyntaxNodeValue::DataType(_) => "DataType",
            SyntaxNodeValue::Function { .. } => "Function",
            SyntaxNodeValue::As { .. } => "As",
            SyntaxNodeValue::IfChain { .. } => "IfChain",
            SyntaxNodeValue::While { .. } => "While",
            SyntaxNodeValue::Loop { .. } => "Loop",
            SyntaxNodeValue::DoWhile { .. } => "DoWhile",
            SyntaxNodeValue::Scope(_) => "Scope",
            SyntaxNodeValue::Symbol { .. } => "Symbol",
            SyntaxNodeValue::Literal(_) => "Literal",
            SyntaxNodeValue::Const { .. } => "Const",
            SyntaxNodeValue::Static { .. } => "Static",
            SyntaxNodeValue::TypeDef { .. } => "TypeDef",
            SyntaxNodeValue::Placeholder => unreachable!(),
        }
    }


    pub fn is_expression(&self) -> bool {
        match self {
            SyntaxNodeValue::RuntimeOp(op) => op.returns_a_value(),
            
            SyntaxNodeValue::Literal(_) |
            SyntaxNodeValue::Scope(_) |
            SyntaxNodeValue::As { .. } |
            SyntaxNodeValue::IfChain { .. } |
            SyntaxNodeValue::Symbol { .. } 
             => true,

            SyntaxNodeValue::DataType(_) |
            SyntaxNodeValue::Function { .. } |
            SyntaxNodeValue::While { .. } |
            SyntaxNodeValue::Loop { .. } |
            SyntaxNodeValue::DoWhile { .. } |
            SyntaxNodeValue::FunctionParams(_) |
            SyntaxNodeValue::Const { .. } |
            SyntaxNodeValue::Static { .. } |
            SyntaxNodeValue::TypeDef { .. } 
             => false,

            SyntaxNodeValue::Placeholder => unreachable!(),
        }
    }

}


#[derive(Debug)]
pub struct SyntaxNode<'a> {

    pub value: SyntaxNodeValue<'a>,

    pub token: Rc<SourceToken<'a>>,

    /// The data type this node evaluates to
    pub data_type: Rc<DataType>,

    /// Whether the node may have side effects.
    /// If this is false, then the node is guaranteed to not have side effects.
    /// Since the compiler must be conservative, this field might be set to true even if the node does not have side effects, if the compiler can't determine that it doesn't.
    pub has_side_effects: bool,

}

impl<'a> SyntaxNode<'a> {

    pub fn new(value: SyntaxNodeValue<'a>, token: Rc<SourceToken<'a>>) -> SyntaxNode<'a> {
        Self {
            value,
            data_type: Rc::new(DataType::Unspecified),
            has_side_effects: false,
            token,
        }
    }


    pub fn extract_value(&mut self) -> SyntaxNodeValue<'a> {
        std::mem::replace(&mut self.value, SyntaxNodeValue::Placeholder)
    }


    /// Return the symbol's literal value, if it is known at compile-time.
    pub fn known_literal_value(&'a self, scope_id: ScopeID, symbol_table: &'a SymbolTable) -> Option<Rc<LiteralValue>> {
        match &self.value {
            SyntaxNodeValue::Literal(value) => Some(value.clone()),
            SyntaxNodeValue::Symbol { name, scope_discriminant } => {
                symbol_table.get_symbol(scope_id, name, *scope_discriminant)
                    .and_then(|symbol| symbol.borrow().get_value().clone())
            },
            _ => None,
        }
    }


    pub fn assume_literal(&self) -> Rc<LiteralValue> {
        match &self.value {
            SyntaxNodeValue::Literal(value) => value.clone(),
            _ => panic!("Expected a literal, but got {:?}", self.value),
        }
    }


    pub fn has_literal_value(&self) -> bool {
        matches!(self.value, SyntaxNodeValue::Literal(_))
    }


    /// Whether the syntax node represents an expression (and thus returns a value).
    pub fn is_expression(&self) -> bool {
        self.value.is_expression()
    }


    pub fn fmt_indented(&self, indent: usize, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // format: "<indent> | <node>"

        write_indent(f, indent)?;
        writeln!(f, "| {:?}: {}", self.value.name(), self.data_type)?;

        // Format the operands
        match &self.value {
            SyntaxNodeValue::RuntimeOp(op) => match op {
                RuntimeOp::Add { left: op1, right: op2 } |
                RuntimeOp::Sub { left: op1, right: op2 } |
                RuntimeOp::Mul { left: op1, right: op2 } |
                RuntimeOp::Div { left: op1, right: op2 } |
                RuntimeOp::Mod { left: op1, right: op2 } |
                RuntimeOp::Assign { left: op1, right: op2 } |
                RuntimeOp::Equal { left: op1, right: op2 } |
                RuntimeOp::NotEqual { left: op1, right: op2 } |
                RuntimeOp::Greater { left: op1, right: op2 } |
                RuntimeOp::Less { left: op1, right: op2 } |
                RuntimeOp::GreaterEqual { left: op1, right: op2 } |
                RuntimeOp::LessEqual { left: op1, right: op2 } |
                RuntimeOp::LogicalAnd { left: op1, right: op2 } |
                RuntimeOp::LogicalOr { left: op1, right: op2 } |
                RuntimeOp::BitShiftLeft { left: op1, right: op2 } |
                RuntimeOp::BitShiftRight { left: op1, right: op2 } |
                RuntimeOp::BitwiseOr { left: op1, right: op2 } |
                RuntimeOp::BitwiseAnd { left: op1, right: op2 } |
                RuntimeOp::BitwiseXor { left: op1, right: op2 } |
                RuntimeOp::ArrayIndex { array: op1, index: op2 }
                => {
                    op1.fmt_indented(indent + 1, f)?;
                    op2.fmt_indented(indent + 1, f)?;
                },
                
                RuntimeOp::Call { callable, args } => {
                    callable.fmt_indented(indent + 1, f)?;
                    for arg in args.iter() {
                        arg.fmt_indented(indent + 1, f)?;
                    }
                
                },

                RuntimeOp::Ref { mutable, expr } |
                RuntimeOp::Deref { mutable, expr }
                => {
                    write_indent(f, indent + 1)?;
                    writeln!(f, "{}", if *mutable { "mut" } else { "immutable" })?;
                    expr.fmt_indented(indent + 1, f)?;
                },
                
                RuntimeOp::MakeArray { elements } => for element in elements.iter() {
                    element.fmt_indented(indent + 1, f)?;
                },
                
                RuntimeOp::BitwiseNot(operand) |
                RuntimeOp::LogicalNot(operand)
                    => operand.fmt_indented(indent + 1, f)?,
                
                RuntimeOp::Return(expr) => if let Some(expr) = expr {
                    expr.fmt_indented(indent + 1, f)?;
                },

                RuntimeOp::Break |
                RuntimeOp::Continue
                    => { /* no operands */ }
            },

            SyntaxNodeValue::FunctionParams(_) => todo!(),
            SyntaxNodeValue::DataType(dt) => {
                write_indent(f, indent + 1)?;
                writeln!(f, "{}", dt.name())?;
            },

            SyntaxNodeValue::Function { name, signature, body } => {
                write_indent(f, indent + 1)?;
                writeln!(f, "fn {name}: {signature}")?;
                body.fmt_indented(indent + 1, f)?;
            },

            SyntaxNodeValue::As { target_type, expr } => {
                write_indent(f, indent + 1)?;
                writeln!(f, "as {}", target_type)?;
                expr.fmt_indented(indent + 1, f)?;
            },

            SyntaxNodeValue::IfChain { if_blocks, else_block } => {
                for if_node in if_blocks.iter() {
                    write_indent(f, indent + 1)?;
                    writeln!(f, "if")?;
                    if_node.condition.fmt_indented(indent + 1, f)?;
                    write_indent(f, indent + 1)?;
                    writeln!(f, "then")?;
                    if_node.body.fmt_indented(indent + 1, f)?;
                }

                if let Some(else_block) = else_block {
                    write_indent(f, indent + 1)?;
                    writeln!(f, "else")?;
                    else_block.fmt_indented(indent + 1, f)?;
                }
            },

            SyntaxNodeValue::While { condition, body } => {
                write_indent(f, indent + 1)?;
                writeln!(f, "while")?;
                condition.fmt_indented(indent + 1, f)?;
                write_indent(f, indent + 1)?;
                writeln!(f, "do")?;
                body.fmt_indented(indent + 1, f)?;
            
            },

            SyntaxNodeValue::Loop { body } => body.fmt_indented(indent + 1, f)?,
            SyntaxNodeValue::DoWhile { body, condition } => {
                write_indent(f, indent + 1)?;
                writeln!(f, "do")?;
                body.fmt_indented(indent + 1, f)?;
                write_indent(f, indent + 1)?;
                writeln!(f, "while")?;
                condition.fmt_indented(indent + 1, f)?;
            
            },

            SyntaxNodeValue::Scope(block) => block.fmt_indented(indent + 1, f)?,
            SyntaxNodeValue::Symbol { name, scope_discriminant } => {
                write_indent(f, indent + 1)?;
                writeln!(f, "{name}: {scope_discriminant}°")?;
            },

            SyntaxNodeValue::Literal(value) => {
                write_indent(f, indent + 1)?;
                writeln!(f, "{value}")?;
            
            },

            SyntaxNodeValue::Const { name, data_type, definition } => {
                write_indent(f, indent + 1)?;
                write!(f, "const {name}: {data_type} =")?;
                definition.fmt_indented(indent + 1, f)?;
            
            },

            SyntaxNodeValue::Static { name, data_type, definition } => {
                write_indent(f, indent + 1)?;
                write!(f, "static {name}: {data_type} =")?;
                definition.fmt_indented(indent + 1, f)?;
            },

            SyntaxNodeValue::TypeDef { name, definition } => {
                write_indent(f, indent + 1)?;
                write!(f, "typedef {name} = {}", definition.name())?;
            },

            SyntaxNodeValue::Placeholder => unreachable!(),
        }

        Ok(())
    }

}


#[derive(Debug)]
pub struct UnparsedScopeBlock<'a> {
    pub statements: Vec<TokenParsingList<'a>>,
    pub scope_id: ScopeID,
}

impl UnparsedScopeBlock<'_> {

    pub fn new(scope_id: ScopeID) -> Self {
        Self {
            statements: Vec::new(),
            scope_id
        }
    }


    pub fn fmt_indented(&self, indent: usize, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write_indent(f, indent)?;
        writeln!(f, "ScopeBlock {{")?;
        for statement in &self.statements {
            statement.fmt_indented(indent + 1, f)?;
            writeln!(f)?;
        }
        writeln!(f)?;
        write_indent(f, indent)?;
        writeln!(f, "}}")?;
        Ok(())
    }

}

impl Display for UnparsedScopeBlock<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_indented(0, f)
    }
}


pub struct ScopeBlock<'a> {
    pub statements: Vec<SyntaxNode<'a>>,
    pub scope_id: ScopeID,
    pub has_side_effects: bool,
}

impl ScopeBlock<'_> {    

    pub fn new(scope_id: ScopeID) -> Self {
        Self {
            statements: Vec::new(),
            scope_id,
            has_side_effects: false,
        }
    }

    pub fn return_type(&self) -> Rc<DataType> {
        if let Some(last_statement) = self.statements.last() {
            last_statement.data_type.clone()
        } else {
            DataType::Void.into()
        }
    }

    pub fn return_value_literal(&self, symbol_table: &SymbolTable ) -> Option<Rc<LiteralValue>> {

        self.statements.last()
            .and_then(|last_statement| last_statement.known_literal_value(self.scope_id, symbol_table)
        )
    }

    pub fn fmt_indented(&self, indent: usize, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for statement in &self.statements {
            statement.fmt_indented(indent, f)?;
        }
        Ok(())
    }

}

impl std::fmt::Debug for ScopeBlock<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for ScopeBlock<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_indented(0, f)
    }
}

impl Default for ScopeBlock<'_> {
    fn default() -> Self {
        Self::new(ScopeID::placeholder())
    }
}


#[derive(Debug)]
pub struct IfBlock<'a> {
    pub condition: SyntaxNode<'a>,
    pub body: ScopeBlock<'a>,
}


pub struct FunctionParam<'a> {
    pub token: Rc<SourceToken<'a>>,
    pub data_type: Rc<DataType>,
    pub mutable: bool,
}

impl FunctionParam<'_> {

    pub fn name(&self) -> &str {
        self.token.string
    }

}

impl std::fmt::Debug for FunctionParam<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}: {}", if self.mutable { "mut " } else { "" }, self.name(), self.data_type)
    }
}

