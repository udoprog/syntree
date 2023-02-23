use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::{prelude::StdRng, Rng, RngCore, SeedableRng};
use rowan::{GreenNodeBuilder, SyntaxNode};
use syntree::{span, Builder, Error, Tree};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
enum Syntax {
    STRING,
    ENTRY,
    WHITESPACE,
    ROOT,
}

use Syntax::*;

impl From<Syntax> for rowan::SyntaxKind {
    fn from(kind: Syntax) -> Self {
        Self(kind as u16)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Lang {}

impl rowan::Language for Lang {
    type Kind = Syntax;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= ROOT as u16);
        unsafe { std::mem::transmute::<u16, Syntax>(raw.0) }
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

fn syntree_build<I>(strings: &[Box<str>], count: usize) -> Result<Tree<Syntax, I>, Error>
where
    I: span::Index<Length = usize>,
{
    let mut builder = Builder::new_with();

    let c = builder.checkpoint()?;

    for s in strings.iter().cycle().take(count) {
        builder.token(STRING, s.len())?;
    }

    builder.close_at(&c, ROOT)?;
    builder.build()
}

fn rowan_build(strings: &[Box<str>], count: usize) -> SyntaxNode<Lang> {
    let mut builder = GreenNodeBuilder::new();

    let c = builder.checkpoint();

    for s in strings.iter().cycle().take(count) {
        builder.token(STRING.into(), s);
    }

    builder.start_node_at(c, ROOT.into());
    builder.finish_node();
    SyntaxNode::new_root(builder.finish())
}

fn rowan_tree(n: usize, strings: &[Box<str>]) -> SyntaxNode<Lang> {
    let mut builder = GreenNodeBuilder::new();

    let c = builder.checkpoint();

    for (_, s) in (0..n).zip(strings.iter().cycle()) {
        builder.start_node(ENTRY.into());
        builder.token(STRING.into(), &**s);
        builder.finish_node();
        builder.token(WHITESPACE.into(), " ");
    }

    builder.start_node_at(c, ROOT.into());
    builder.finish_node();
    SyntaxNode::new_root(builder.finish())
}

fn syntree_tree<I>(n: usize, strings: &[Box<str>]) -> Result<Tree<Syntax, I>, Error>
where
    I: span::Index<Length = usize>,
{
    let mut builder = Builder::<_, I>::new_with();

    let c = builder.checkpoint()?;

    for (_, s) in (0..n).zip(strings.iter().cycle()) {
        builder.open(ENTRY)?;
        builder.token(STRING.into(), s.len())?;
        builder.close()?;
        builder.token(WHITESPACE.into(), 1)?;
    }

    builder.close_at(&c, ROOT)?;
    builder.build()
}

fn setup(c: &mut Criterion) {
    let strings = generate_random(100, 5, 20);

    let sizes = [1024, 2048, 4096, 8192, 16384];

    {
        let mut group = c.benchmark_group("building");

        for size in sizes {
            group.bench_with_input(BenchmarkId::new("syntree-u32", size), &size, |b, size| {
                b.iter(|| syntree_build::<u32>(&strings, *size).expect("failed to build tree"))
            });

            group.bench_with_input(BenchmarkId::new("syntree-usize", size), &size, |b, size| {
                b.iter(|| syntree_build::<usize>(&strings, *size).expect("failed to build tree"))
            });

            group.bench_with_input(BenchmarkId::new("rowan", size), &size, |b, size| {
                b.iter(|| rowan_build(&strings, *size))
            });
        }
    }

    {
        let mut group = c.benchmark_group("children_full");

        for size in sizes {
            group.bench_with_input(BenchmarkId::new("syntree-u32", size), &size, |b, size| {
                let syntree = syntree_tree::<u32>(*size, &strings).unwrap();
                let root = syntree.first().unwrap();
                b.iter(|| root.children().count())
            });

            group.bench_with_input(BenchmarkId::new("syntree-usize", size), &size, |b, size| {
                let syntree = syntree_tree::<usize>(*size, &strings).unwrap();
                let root = syntree.first().unwrap();
                b.iter(|| root.children().count())
            });

            group.bench_with_input(BenchmarkId::new("rowan", size), &size, |b, size| {
                let rowan = rowan_tree(*size, &strings);
                b.iter(|| rowan.children_with_tokens().count())
            });
        }
    }

    {
        let mut group = c.benchmark_group("children_nodes_only");

        for size in sizes {
            group.bench_with_input(BenchmarkId::new("syntree-u32", size), &size, |b, size| {
                let syntree = syntree_tree::<u32>(*size, &strings).unwrap();
                let root = syntree.first().unwrap();
                b.iter(|| root.children().skip_tokens().count())
            });

            group.bench_with_input(BenchmarkId::new("syntree-usize", size), &size, |b, size| {
                let syntree = syntree_tree::<usize>(*size, &strings).unwrap();
                let root = syntree.first().unwrap();
                b.iter(|| root.children().skip_tokens().count())
            });

            group.bench_with_input(BenchmarkId::new("rowan", size), &size, |b, size| {
                let rowan = rowan_tree(*size, &strings);
                b.iter(|| rowan.children().count())
            });
        }
    }
}

criterion_group!(benches, setup);
criterion_main!(benches);

fn generate_random(count: usize, min: usize, max: usize) -> Vec<Box<str>> {
    let mut rng = StdRng::seed_from_u64(0x12345678);
    let mut s = String::with_capacity(16);

    let mut output = Vec::with_capacity(count);

    for _ in 0..count {
        let target = (rng.gen::<usize>() % (max - min)) + min;
        assert!(target >= min && target < max, "target was {}", target);

        while s.len() < target {
            s.extend(char::from_u32(rng.next_u32()).filter(|c| !c.is_whitespace()));
        }

        output.push(s.as_str().into());
    }

    output
}
