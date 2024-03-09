use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use crate::irc::{IRCode, LabelID};


/// A basic block is a sequence of instructions that always get executed together.
/// 
/// A basic block ends when the control flow changes.
/// 
/// A basic block starts at labels or after control flow instructions.
pub struct BasicBlock {
    /// The IR code of this basic block
    pub code: IRCode,
    /// List of basic blocks that can be directly reached from this basic block
    next: Vec<Rc<RefCell<BasicBlock>>>,
    /// List of basic blocks that can directly reach this basic block
    refs: Vec<Rc<RefCell<BasicBlock>>>,
    /// Number of times this block is referenced by other blocks.
    /// This is different from `refs.len()` because a referencing block may get deleted as an optimization.
    /// Since iterating through the `refs` vector would be inefficient, we keep a mutable count of the references.
    pub ref_count: usize,
}

impl BasicBlock {

    pub fn new(code: IRCode) -> Self {
        Self {
            code,
            next: vec![],
            refs: vec![],
            ref_count: 0,
        }
    }


    pub fn next_blocks(&self) -> &Vec<Rc<RefCell<BasicBlock>>> {
        &self.next
    }


    pub fn push_next(&mut self, next: Rc<RefCell<BasicBlock>>) {
        self.next.push(next);
    }


    pub fn push_ref(&mut self, ref_: Rc<RefCell<BasicBlock>>) {
        self.refs.push(ref_);
        self.ref_count += 1;
    }

}

impl std::fmt::Debug for BasicBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "BasicBlock {{")?;
        writeln!(f, "  next: {}", self.next.len())?;
        writeln!(f, "  refs: {}", self.refs.len())?;
        writeln!(f, "  ref_count: {:?}", self.ref_count)?;
        writeln!(f, "  code:")?;
        
        for ir in self.code.iter() {
            writeln!(f, "    {ir}")?;
        }

        writeln!(f, "}}")
    }
}


/// Maps a label to the basic block it introduces.
/// 
/// This is used to determine which basic block to jump to.
pub type BasicBlockTable = HashMap<LabelID, Rc<RefCell<BasicBlock>>>;

// TODO: make this a struct with references to the original function to allow for generating code and identifying the main function/exporting functionsand identifying the main function, etc.
pub type FunctionGraph = Vec<Rc<RefCell<BasicBlock>>>;

