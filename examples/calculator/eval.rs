use crate::Syntax;
use syntree::{Node, Span, Tree};
use thiserror::Error;

use Syntax::*;

#[derive(Debug, Error)]
#[error("{kind}")]
#[non_exhaustive]
pub(crate) struct EvalError {
    pub(crate) span: Span,
    pub(crate) kind: EvalErrorKind,
}

impl EvalError {
    const fn new(span: Span, kind: EvalErrorKind) -> Self {
        Self { span, kind }
    }
}

#[derive(Debug, Error)]
pub(crate) enum EvalErrorKind {
    #[error("expected {0:?} but was {1:?}")]
    Expected(Syntax, Syntax),

    #[error("expected {0:?}")]
    Missing(Syntax),

    #[error("bad number")]
    BadNumber,

    #[error("{0:?} is not a valid operator")]
    UnexpectedOperator(Syntax),

    #[error("expected an operator")]
    ExpectedOperator,

    #[error("numerical overflow")]
    Overflow,

    #[error("numerical underflow")]
    Underflow,

    #[error("divide by zero")]
    DivideByZero,
}

use EvalErrorKind::*;

fn pow(a: i64, b: i64) -> Option<i64> {
    let pow = u32::try_from(b).ok()?;
    a.checked_pow(pow)
}

fn eval_node(mut node: Node<'_, Syntax>, source: &str) -> Result<i64, EvalError> {
    loop {
        return match *node.value() {
            GROUP => {
                node = node
                    .first()
                    .ok_or(EvalError::new(*node.span(), Missing(NUMBER)))?;
                continue;
            }
            NUMBER => source[node.range()]
                .parse::<i64>()
                .map_err(|_| EvalError::new(*node.span(), BadNumber)),
            OPERATION => {
                let mut it = node.children().skip_tokens();

                let first = it
                    .next()
                    .ok_or(EvalError::new(*node.span(), Missing(NUMBER)))?;

                let mut base = eval_node(first, source)?;

                while let Some(op) = it.next() {
                    let op = op
                        .first()
                        .ok_or(EvalError::new(*node.span(), ExpectedOperator))?;

                    let (calculate, error): (fn(i64, i64) -> Option<i64>, _) = match *op.value() {
                        PLUS => (i64::checked_add, Overflow),
                        MINUS => (i64::checked_sub, Underflow),
                        MUL => (i64::checked_mul, Overflow),
                        DIV => (i64::checked_div, DivideByZero),
                        POW => (pow, Overflow),
                        what => return Err(EvalError::new(*node.span(), UnexpectedOperator(what))),
                    };

                    let first = it
                        .next()
                        .ok_or(EvalError::new(*node.span(), Missing(NUMBER)))?;
                    let b = eval_node(first, source)?;

                    base = match calculate(base, b) {
                        Some(n) => n,
                        None => return Err(EvalError::new(op.span().join(node.span()), error)),
                    }
                }

                Ok(base)
            }
            what => Err(EvalError::new(*node.span(), Expected(NUMBER, what))),
        };
    }
}

/// Eval a tree emitting all available expressions parsed from it.
pub(crate) fn eval<'a>(
    tree: &'a Tree<Syntax>,
    source: &'a str,
) -> impl Iterator<Item = Result<i64, EvalError>> + 'a {
    let mut it = tree.children().skip_tokens();

    std::iter::from_fn(move || {
        let node = it.next()?;
        Some(eval_node(node, source))
    })
}
