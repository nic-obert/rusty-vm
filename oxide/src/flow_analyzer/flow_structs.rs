use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use crate::{irc::{FunctionLabels, IRCode, LabelID}, lang::data_types::DataType};


/// A basic block is a sequence of instructions that always get executed together.
///
/// A basic block ends when the control flow changes.
///
/// A basic block starts at labels or after control flow instructions.
pub struct BasicBlock {
    /// The IR code of this basic block
    pub code: IRCode,
    /// List of basic blocks that can be directly reached from this basic block.
    /// These include the block that comes right after self and eventual blocks that can be jumped to
    next: Vec<Rc<RefCell<BasicBlock>>>,
    /// List of basic blocks that can directly reach this basic block.
    /// These include the previous block and eventual blocks that jump to this block's start label.
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

/// Represents a function's code as a graph where every node is a basic block
#[derive(Debug)]
pub struct FunctionGraph<'a> {

    /// The function name as it appears in the source code
    pub name: &'a str,

    /// The code of the function, split into basic blocks
    pub code_blocks: Vec<Rc<RefCell<BasicBlock>>>,

    /// Important function labels. There are used when calling the function and when returning
    pub labels: FunctionLabels,

    /// The function signature. This is used to correctly implement the calling convention.
    pub signature: Rc<DataType>,

}
