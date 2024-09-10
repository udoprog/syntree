use syntree::{Node, Span, Tree};
use thiserror::Error;

use crate::Syntax;

use Syntax::{Div, Group, Minus, Mul, Number, Operation, Plus, Pow};

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

fn eval_node<F>(mut node: Node<'_, Syntax, F>, source: &str) -> Result<i64, EvalError<F::Index>>
where
    F: syntree::Flavor,
{
    loop {
        return match node.value() {
            Group => {
                node = node
                    .first()
                    .ok_or(EvalError::new(*node.span(), Missing(Number)))?;
                continue;
            }
            Number => source[node.range()]
                .parse::<i64>()
                .map_err(|_| EvalError::new(*node.span(), BadNumber)),
            Operation => {
                let mut it = node.children().skip_tokens();

                let first = it
                    .next()
                    .ok_or(EvalError::new(*node.span(), Missing(Number)))?;

                let mut base = eval_node(first, source)?;

                while let Some(op) = it.next() {
                    let op = op
                        .first()
                        .ok_or(EvalError::new(*node.span(), ExpectedOperator))?;

                    let (calculate, error): (fn(i64, i64) -> Option<i64>, _) = match op.value() {
                        Plus => (i64::checked_add, Overflow),
                        Minus => (i64::checked_sub, Underflow),
                        Mul => (i64::checked_mul, Overflow),
                        Div => (i64::checked_div, DivideByZero),
                        Pow => (pow, Overflow),
                        what => return Err(EvalError::new(*node.span(), UnexpectedOperator(what))),
                    };

                    let first = it
                        .next()
                        .ok_or(EvalError::new(*node.span(), Missing(Number)))?;
                    let b = eval_node(first, source)?;

                    base = match calculate(base, b) {
                        Some(n) => n,
                        None => return Err(EvalError::new(op.span().join(node.span()), error)),
                    }
                }

                Ok(base)
            }
            what => Err(EvalError::new(*node.span(), Expected(Number, what))),
        };
    }
}

/// Eval a tree emitting all available expressions parsed from it.
pub(crate) fn eval<'a, F>(
    tree: &'a Tree<Syntax, F>,
    source: &'a str,
) -> impl Iterator<Item = Result<i64, EvalError<F::Index>>> + 'a
where
    F: syntree::Flavor,
{
    let mut it = tree.children().skip_tokens();

    std::iter::from_fn(move || {
        let node = it.next()?;
        Some(eval_node(node, source))
    })
}
