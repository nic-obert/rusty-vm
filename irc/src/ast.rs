use rust_vm_lib::ir::IRCode;

use crate::token::Token;



pub struct SyntaxNode {


}


pub struct AST {

    root: SyntaxNode

}


impl AST {

    pub fn build(tokens: &[Token], source: &IRCode) -> Self {
        todo!()
    }

}

