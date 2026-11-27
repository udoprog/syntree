//! Building a tree with and without pre-sizing the builder.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use syntree::{Builder, Error, FlavorDefault, Tree};

fn build(
    mut builder: Builder<u16, FlavorDefault>,
    tokens: usize,
) -> Result<Tree<u16, FlavorDefault>, Error> {
    builder.open(0)?;

    for _ in 0..tokens {
        builder.token(1, 1)?;
    }

    builder.close()?;
    builder.build()
}

fn setup(c: &mut Criterion) {
    let mut group = c.benchmark_group("capacity");

    for tokens in [1_000, 100_000] {
        group.bench_with_input(BenchmarkId::new("new", tokens), &tokens, |b, &tokens| {
            b.iter(|| build(Builder::new(), tokens).unwrap())
        });

        group.bench_with_input(
            BenchmarkId::new("with_capacities", tokens),
            &tokens,
            |b, &tokens| {
                b.iter(|| {
                    let builder = Builder::with_capacities(tokens + 1, tokens).unwrap();
                    build(builder, tokens).unwrap()
                })
            },
        );
    }
}

criterion_group!(benches, setup);
criterion_main!(benches);
