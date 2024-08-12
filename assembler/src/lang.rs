use std::collections::HashMap;
use std::rc::Rc;

use rusty_vm_lib::vm::Address;
use rusty_vm_lib::assembly::{AsmInstructionNode, PseudoInstructionNode, SourceToken};

use crate::tokenizer::Token;


#[derive(Debug, Clone)]
/// Represents a macro definition in the assembly code
pub struct FunctionMacroDef<'a> {

    pub source: Rc<SourceToken<'a>>,
    /// Maps the parameter name to its position in the invocation
    pub params: HashMap<&'a str, usize>,
    pub body: Box<[Box<[Token<'a>]>]>,

}


#[derive(Debug, Clone)]
pub struct InlineMacroDef<'a> {

    pub source: Rc<SourceToken<'a>>,
    pub def: Box<[Token<'a>]>

}


#[derive(Debug, Clone)]
pub struct LabelDef<'a> {

    pub source: Rc<SourceToken<'a>>,
    pub value: Option<Address>

}


#[derive(Debug)]
pub struct AsmNode<'a> {

    pub value: AsmNodeValue<'a>,
    
    #[cfg(debug_assertions)]
    #[allow(dead_code)]
    pub source: Rc<SourceToken<'a>>

}


#[derive(Debug)]
pub enum AsmNodeValue<'a> {

    Instruction (AsmInstructionNode<'a>),
    PseudoInstruction (PseudoInstructionNode<'a>),
    Label (&'a str),

}

