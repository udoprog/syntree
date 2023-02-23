use crate::Syntax;
use syntree::{span, Node, Span, Tree};
use thiserror::Error;

use Syntax::{DIV, GROUP, MINUS, MUL, NUMBER, OPERATION, PLUS, POW};

#[derive(Debug, Error)]
#[error("{kind}")]
#[non_exhaustive]
pub(crate) struct EvalError<I> {
    pub(crate) span: Span<I>,
    pub(crate) kind: EvalErrorKind,
}

impl<I> EvalError<I> {
    const fn new(span: Span<I>, kind: EvalErrorKind) -> Self {
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

use EvalErrorKind::{
    BadNumber, DivideByZero, Expected, ExpectedOperator, Missing, Overflow, Underflow,
    UnexpectedOperator,
};

fn pow(a: i64, b: i64) -> Option<i64> {
    let pow = u32::try_from(b).ok()?;
    a.checked_pow(pow)
}

fn eval_node<I>(mut node: Node<'_, Syntax, I>, source: &str) -> Result<i64, EvalError<I>>
where
    I: span::Index,
{
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
pub(crate) fn eval<'a, I>(
    tree: &'a Tree<Syntax, I>,
    source: &'a str,
) -> impl Iterator<Item = Result<i64, EvalError<I>>> + 'a
where
    I: span::Index,
{
    let mut it = tree.children().skip_tokens();

    std::iter::from_fn(move || {
        let node = it.next()?;
        Some(eval_node(node, source))
    })
}
