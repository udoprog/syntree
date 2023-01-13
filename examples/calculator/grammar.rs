use crate::parsing::Parser;
use crate::Syntax;
use anyhow::Result;
use syntree::Error;

use self::Syntax::{
    CLOSE_PAREN, DIV, ERROR, GROUP, MINUS, MUL, NUMBER, OPEN_PAREN, OPERATION, OPERATOR, PLUS, POW,
};

fn op(syntax: Syntax) -> Option<(u8, u8)> {
    let prio = match syntax {
        PLUS | MINUS => (1, 2),
        DIV | MUL => (3, 4),
        POW => (7, 8),
        _ => return None,
    };

    Some(prio)
}

fn expr(p: &mut Parser<'_>, min: u8) -> Result<(), Error> {
    // Eat all available whitespace before getting a checkpoint.
    let tok = p.peek()?;

    let c = p.tree.checkpoint()?;

    match tok.syntax {
        OPEN_PAREN => {
            p.token()?;

            let c = p.tree.checkpoint()?;
            expr(p, 0)?;
            p.tree.close_at(&c, GROUP)?;

            if !p.eat(CLOSE_PAREN)? {
                p.bump(ERROR)?;
                return Ok(());
            }
        }
        NUMBER => {
            p.bump(NUMBER)?;
        }
        _ => {
            p.bump(ERROR)?;
            return Ok(());
        }
    }

    let mut operation = false;

    loop {
        let tok = p.peek()?;

        let min = match op(tok.syntax) {
            Some((left, right)) if left >= min => right,
            _ => break,
        };

        p.bump(OPERATOR)?;
        expr(p, min)?;
        operation = true;
    }

    if operation {
        p.tree.close_at(&c, OPERATION)?;
    }

    Ok(())
}

/// Parse the root.
pub(crate) fn root(p: &mut Parser<'_>) -> Result<()> {
    loop {
        expr(p, 0)?;

        if p.is_eof()? {
            break;
        }

        // Simple error recovery where we consume until we find an operator
        // which will be consumed as an expression next.
        let c = p.tree.checkpoint()?;
        p.advance_until(&[NUMBER])?;
        p.tree.close_at(&c, ERROR)?;
    }

    Ok(())
}
