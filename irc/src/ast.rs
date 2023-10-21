use rust_vm_lib::ir::IRCode;

use crate::token::{Token, TokenKind, TokenList};


//fn find_highest_priority<'a>(tokens: &'a TokenList) -> Option<&'a Token<'a>> {

    // let mut highest_priority: Option<&'a Token> = None;

    // for token in tokens.iter() {

    //     // Don't search past the end of the statement or into a new scope
    //     if matches!(token.value, TokenKind::Semicolon | TokenKind::ScopeOpen) {
    //         break;
    //     }

    //     if let Some(hpt) = highest_priority {
    //         if token.priority > hpt.priority {
    //             highest_priority = Some(token);
    //         }
    //     }
    // }

    // highest_priority
//}


// fn find_next_statement<'a>(tokens: &TokenList) -> Option<usize> {

//     for (index, token) in tokens.iter().enumerate() {
//         if matches!(token.value, TokenKind::Semicolon | TokenKind::ScopeOpen) {
//             return Some(index);
//         }
//     }

//     None
// }


pub fn parse_token_syntax(tokens: &mut TokenList, source: &IRCode) {

    loop {



    }

}

