use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::irc::{FunctionIR, IRCode, IROperator, LabelID};


/// A basic block is a sequence of instructions that always get executed together.
/// 
/// A basic block ends when the control flow changes.
/// 
/// A basic block starts at labels or after control flow instructions.
pub struct BasicBlock {
    /// The IR code of this basic block
    pub code: IRCode,
    /// List of basic blocks that can be directly reached from this basic block
    pub next: Vec<Rc<RefCell<BasicBlock>>>,
    /// List of basic blocks that can directly reach this basic block
    pub refs: Vec<Rc<RefCell<BasicBlock>>>,
}

impl BasicBlock {

    pub fn new(code: IRCode) -> Self {
        Self {
            code,
            next: vec![],
            refs: vec![],
        }
    }

}


/// Maps a label to the basic block it introduces.
/// 
/// This is used to determine which basic block to jump to.
pub type BasicBlockTable = HashMap<LabelID, Rc<RefCell<BasicBlock>>>;


fn divide_basic_blocks(ir_function: FunctionIR, bb_table: &mut BasicBlockTable) -> Vec<Rc<RefCell<BasicBlock>>> {
    
    let mut basic_blocks: Vec<Rc<RefCell<BasicBlock>>> = Vec::new();

    let mut function_code = ir_function.code.take();

    let mut node_ptr = unsafe { function_code.head() };
    while let Some(node) = unsafe { node_ptr.as_mut() } {

        // Split the list at control flow changes (jumps and labels)

        let (first_half, second_half) = match &node.data.op {

            IROperator::Add { .. } |
            IROperator::Sub { .. } |
            IROperator::Mul { .. } |
            IROperator::Div { .. } |
            IROperator::Mod { .. } |
            IROperator::Assign { .. } |
            IROperator::Deref { .. } |
            IROperator::DerefAssign { .. } |
            IROperator::Ref { .. } |
            IROperator::Greater { .. } |
            IROperator::Less { .. } |
            IROperator::GreaterEqual { .. } |
            IROperator::LessEqual { .. } |
            IROperator::Equal { .. } |
            IROperator::NotEqual { .. } |
            IROperator::BitShiftLeft { .. } |
            IROperator::BitShiftRight { .. } |
            IROperator::BitNot { .. } |
            IROperator::BitAnd { .. } |
            IROperator::BitOr { .. } |
            IROperator::BitXor { .. } |
            IROperator::Copy { .. } |
            IROperator::DerefCopy { .. } |
            IROperator::PushScope { .. } |
            IROperator::PopScope { .. } |
            IROperator::Nop => {
                // No control flow change
                node_ptr = unsafe { node.next() };
                continue;
            },
        
            IROperator::JumpIf { .. } |
            IROperator::Call { .. } |
            IROperator::Return |
            IROperator::JumpIfNot { .. } |
            IROperator::Jump { .. }
                => unsafe { function_code.split_after(node_ptr) },

            IROperator::Label { .. }
                => unsafe { function_code.split_before(node_ptr) },
        };

        function_code = second_half;

        // Don't bother adding an empty block
        if !first_half.is_empty() {

            let basic_block = Rc::new(RefCell::new(BasicBlock::new(first_half)));
        
            // If the basic block is introduced by a label, record it in the table to allow jumps to this block
            if let IROperator::Label { label } = unsafe { basic_block.borrow().code.head().as_ref() }.unwrap().data.op {
                bb_table.insert(label.0, basic_block.clone());
            }

            basic_blocks.push(basic_block);
        }

        node_ptr = unsafe { node.next() };
    }

    // Add the last block, if it wasn't added yet
    if !function_code.is_empty() {

        let basic_block = Rc::new(RefCell::new(BasicBlock::new(function_code)));
        
        if let IROperator::Label { label } = unsafe { basic_block.borrow().code.head().as_ref() }.unwrap().data.op {
            bb_table.insert(label.0, basic_block.clone());
        }

        basic_blocks.push(basic_block);
    }

    basic_blocks
}


fn analyze_function_graph(function_graph: Vec<Rc<RefCell<BasicBlock>>>, bb_table: &BasicBlockTable) {

    todo!()

}


pub fn flow_graph(ir_code: Vec<FunctionIR>) {

    let mut bb_table = BasicBlockTable::new();

    // First, we need to divide the basic blocks of each function
    let function_blocks: Vec<Vec<Rc<RefCell<BasicBlock>>>> = ir_code.into_iter()
    .map(|ir_function| {
        divide_basic_blocks(ir_function, &mut bb_table)
    }).collect();

    // Now we can analyze the relationships between the basic blocks
    for basic_blocks in function_blocks {
        analyze_function_graph(basic_blocks, &bb_table);
    }

}

