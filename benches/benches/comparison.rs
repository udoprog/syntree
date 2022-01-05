use criterion::{criterion_group, criterion_main, Criterion};
use rand::{Rng, RngCore};
use rowan::{GreenNodeBuilder, SyntaxNode};
use syntree::{Tree, TreeBuilder, TreeBuilderError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
enum SyntaxKind {
    STRING,

    ROOT,
}

use SyntaxKind::*;

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Lang {}

impl rowan::Language for Lang {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= ROOT as u16);
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
    }
    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

fn syntree(strings: &[Box<str>], count: usize) -> Result<Tree<SyntaxKind>, TreeBuilderError> {
    let mut builder = TreeBuilder::new();

    let c = builder.checkpoint();

    for s in strings.iter().cycle().take(count) {
        builder.token(STRING, s.len());
    }

    builder.close_at(c, ROOT)?;
    builder.build()
}

fn rowan(strings: &[Box<str>], count: usize) -> SyntaxNode<Lang> {
    let mut builder = GreenNodeBuilder::new();

    let c = builder.checkpoint();

    for s in strings.iter().cycle().take(count) {
        builder.token(STRING.into(), s);
    }

    builder.start_node_at(c, ROOT.into());
    builder.finish_node();
    SyntaxNode::new_root(builder.finish())
}

fn rowan_benchmark(c: &mut Criterion) {
    let sources = generate_random(100, 5, 20);

    c.bench_function("syntree", |b| {
        b.iter(|| syntree(&sources, 1000000).expect("failed to build tree"))
    });
    c.bench_function("rowan", |b| b.iter(|| rowan(&sources, 1000000)));
}

fn generate_random(count: usize, min: usize, max: usize) -> Vec<Box<str>> {
    let mut rng = rand::thread_rng();
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

criterion_group!(benches, rowan_benchmark);
criterion_main!(benches);
