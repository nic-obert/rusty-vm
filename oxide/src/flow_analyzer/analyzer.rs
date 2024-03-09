use std::rc::Rc;
use std::mem;
use std::cell::RefCell;

use crate::irc::{FunctionIR, IROperator};
use crate::cli_parser::OptimizationFlags;

use super::{BasicBlock, BasicBlockTable, FunctionGraph};


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
        // Note that an empty block cannot be jumped to because jumping to a block requires that the block contains (starts with) a label.
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


fn connect_function_graph(function_graph: &FunctionGraph, bb_table: &BasicBlockTable) {
    /*
        Set the next and refs fields of each basic block.
        Iterate over the basic blocks and update the parameters based on the last instruction of the block.
    */

    let mut graph_iter = function_graph.iter().peekable();
    while let Some(basic_block_ref) = graph_iter.next() {
        
        let mut basic_block = basic_block_ref.borrow_mut();
        
        assert!(!basic_block.code.is_empty(), "Empty basic blocks should not be allowed. This is a bug.");

        match &unsafe { basic_block.code.tail().as_ref() }.unwrap().data.op {

            // These instructions don't change the control flow
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
            IROperator::Nop
                => { },

            IROperator::Jump { target } => {

                let target_ref = bb_table.get(&target.0).unwrap();

                basic_block.push_next(target_ref.clone());

                let mut target = if Rc::ptr_eq(target_ref, basic_block_ref) {
                    // The block jumps to itself. To satisfy the borrow checker, we don't borrow the block again.
                    basic_block
                } else {
                    // If the block jumps to another block, it's safe to borrow the block as mutable.
                    target_ref.borrow_mut()
                };

                target.push_ref(basic_block_ref.clone());
            },

            IROperator::JumpIfNot { target, .. } |
            IROperator::JumpIf { target, .. } => {

                let target_ref = bb_table.get(&target.0).unwrap();

                basic_block.push_next(target_ref.clone());

                // If the jump condition isn't met, the next block is the one that will be executed.
                if let Some(&next_block) = graph_iter.peek() {
                    basic_block.push_next(next_block.clone());
                    next_block.borrow_mut().push_ref(basic_block_ref.clone());
                }

                let mut target = if Rc::ptr_eq(target_ref, basic_block_ref) {
                    // The block jumps to itself. To satisfy the borrow checker, we don't borrow the block again.
                    basic_block
                } else {
                    // If the block jumps to another block, it's safe to borrow the block as mutable.
                    target_ref.borrow_mut()
                };

                target.push_ref(basic_block_ref.clone());
            },

            IROperator::Call { return_label, .. } => {

                let return_target_ref = bb_table.get(&return_label.0).unwrap();
                // The next block will be execute after the function call returns.
                return_target_ref.borrow_mut().push_ref(basic_block_ref.clone());

                // We may not know which block will be executed by the function call, so don't do anything with it.
            },

            IROperator::Label { .. } => {},
            // Return does not know where to jump to
            IROperator::Return => {},
        }
        
    }
}


fn remove_unreachable_blocks(function_graph: &mut FunctionGraph) {
    /*
        Remove all basic blocks that are not reachable from the function entry block.
        A block is considered unreachable if it has no refs.
        Since all jumps withing a functions are to known labels, we can assume that we know all the possible paths.
        A call instruction may jump to an unknown label, but that would pass control to another function.
        The only block that may have no refs is the entry block because it may be called indirectly from another function.

        Perform a linear iteration over the basic blocks, starting from the entry block. 
        A linear iteration is necessary because the graph may contain cycles.
        A top-to-bottom iteration is necessary because the elimination of a block may change the refs of other blocks.
    */
    
    let old_graph = mem::replace(function_graph, Vec::with_capacity(function_graph.len()));
    let mut old_graph_iter = old_graph.into_iter();
    
    function_graph.push(
        old_graph_iter.next().unwrap() // Assume the graph has at least one initial basic block
    );

    for block_ref in old_graph_iter {

        if block_ref.borrow().ref_count == 0 {
            // The block is unreachable. Remove it and decrement the ref_count of its next blocks
            for referenced_block in block_ref.borrow().next_blocks() {
                // Since the block has no refs, we can assume that the next block is not the block itself, so it's safe to borrow_mut
                referenced_block.borrow_mut().ref_count -= 1;
            }
            
        } else {
            // The block is reachable, so keep it
            function_graph.push(block_ref);
        }
    }

}


pub fn flow_graph(ir_code: Vec<FunctionIR>, optimization_flags: &OptimizationFlags) {

    let mut bb_table = BasicBlockTable::new();

    // First, we need to divide the basic blocks of each function
    let mut function_blocks: Vec<FunctionGraph> = ir_code.into_iter()
    .map(|ir_function| {
        divide_basic_blocks(ir_function, &mut bb_table)
    }).collect();

    // Now we can analyze the relationships between the basic blocks
    for basic_blocks in function_blocks.iter() {
        connect_function_graph(basic_blocks, &bb_table);
    }

    if optimization_flags.remove_useless_code {
        for basic_blocks in function_blocks.iter_mut() {
            remove_unreachable_blocks(basic_blocks);
        }
    }

}

/*
    Possible optimizations:

    When unconditionally jumping to a basic block, if the target block is small enough (small number of instructions)
    we can inline the jump target and remove the jump instruction.

    When calling a function (which is a special case of unconditional jumping), if the function is small enough,
    we can inline the function and remove the call instruction.

    
*/

