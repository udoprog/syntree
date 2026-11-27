//! A checkpoint-heavy parse, shaped like a precedence-climbing expression
//! parser: every precedence level takes a checkpoint, and most are never
//! closed.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use syntree::{Builder, Error};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Syntax {
    Root,
    Binary,
    Number,
    Op,
}

/// Precedence levels between an expression and its operands.
const LEVELS: usize = 8;

/// Build `count` statements of `a + b + c + d`, taking a checkpoint at every
/// precedence level of every operand.
fn build(count: usize) -> Result<syntree::Tree<Syntax, syntree::FlavorDefault>, Error> {
    let mut b = Builder::new();
    b.open(Syntax::Root)?;

    for _ in 0..count {
        let expr = b.checkpoint()?;

        for operand in 0..4 {
            if operand > 0 {
                b.token(Syntax::Op, 1)?;
            }

            for _ in 0..LEVELS {
                b.checkpoint()?;
            }

            let number = b.checkpoint()?;
            b.token(Syntax::Number, 1)?;
            b.close_at(number, Syntax::Number)?;

            if operand > 0 {
                b.close_at(expr, Syntax::Binary)?;
            }
        }
    }

    b.close()?;
    b.build()
}

fn setup(c: &mut Criterion) {
    let mut group = c.benchmark_group("checkpoints");

    for count in [16, 256, 4096] {
        group.bench_with_input(
            BenchmarkId::new("expressions", count),
            &count,
            |b, &count| {
                b.iter(|| build(count).unwrap());
            },
        );
    }
}

criterion_group!(benches, setup);
criterion_main!(benches);
