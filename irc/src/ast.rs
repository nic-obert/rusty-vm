use rust_vm_lib::ir::IRCode;

use crate::token::{Token, TokenKind};


fn find_highest_priority<'a>(tokens: &'a [Token]) -> Option<(usize, &'a Token<'a>)> {

    let mut highest_priority: Option<(usize, &'a Token)> = None;

    for (index, token) in tokens.iter().enumerate() {

        // Don't search past the end of the statement or into a new scope
        if matches!(token.value, TokenKind::Semicolon | TokenKind::ScopeOpen) {
            break;
        }

        if let Some((_hpi, hpt)) = highest_priority {
            if token.priority > hpt.priority {
                highest_priority = Some((index, token));
            }
        }
    }

    highest_priority
}


fn find_next_statement(tokens: &[Token]) -> Option<usize> {

    for (index, token) in tokens.iter().enumerate() {
        if matches!(token.value, TokenKind::Semicolon | TokenKind::ScopeOpen) {
            return Some(index);
        }
    }

    None
}


fn parse_scope<'a>(tokens: &mut Vec<Token<'a>>, source: &'a IRCode) {
    
    loop {

        if let Some(hp) = find_highest_priority(&tokens) {
            
        } else {
            // The current statement is finished, pass on to the next
            if let Some(start) = find_next_statement(&tokens) {
                //parse_scope(&mut tokens[start..], source);
            } else {
                // There's nothing more to parse
                break;
            }   
        }

    }

}


pub fn parse_syntax_tree<'a>(mut tokens: Vec<Token<'a>>, source: &'a IRCode) -> Vec<Token<'a>> {
    parse_scope(&mut tokens, source);
    tokens
}

