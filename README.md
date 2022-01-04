# syntree

[<img alt="github" src="https://img.shields.io/badge/github-udoprog/syntree-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/syntree)
[<img alt="crates.io" src="https://img.shields.io/crates/v/syntree.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/syntree)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-syntree-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/syntree)
[<img alt="build status" src="https://img.shields.io/github/workflow/status/udoprog/syntree/CI/main?style=for-the-badge" height="20">](https://github.com/udoprog/syntree/actions?query=branch%3Amain)

A memory efficient syntax tree.

This crate provides a tree structure which always is contiguously stored and
manipulated in memory. It provides similar APIs as [`rowan`] and is intended
to be an efficient replacement for it (read more below).

<br>

## Usage

Add `syntree` to your crate:

```toml
syntree = "0.2.0"
```

<br>

If you want a complete sample for how `syntree` can be used for parsing, see
the [calculator example].

### Syntax trees

At the root, `syntree` provides a way to model an [abstract syntax tree]
(AST). The nodes of the trees are typically modelled by variants in an enum,
but could in theory [consist of anything you like].

We distinguish between nodes which are elements that can have zero or more
children, and tokens which are terminating elements. Each token has a span
associated with it. This span is intended to indicate *where* in a source
this token was identified so that it can be referenced later.

This is the primary difference between `syntree` and [`rowan`]. *We don't
store the original source* in the syntax tree. Instead, the user of the
library is responsible for providing it.

The following is a simple example of how we can build a syntax tree with
fake spans that do not reference anything in particular. And when we report
*tokens*, we only include the *length* of the span reported. This ensures
that in order to make `syntree`s behave correctly, the whole source must be
reported.

```rust
use syntree::{Span, TreeBuilder};

#[derive(Debug, Clone, Copy)]
enum Syntax {
    OPERATION,
    NUMBER,
    PLUS,
}

use Syntax::*;

let mut b = TreeBuilder::new();

b.start_node(OPERATION);

b.start_node(NUMBER);
b.token(NUMBER, 4);
b.end_node()?;

b.start_node(PLUS);
b.token(PLUS, 1);
b.end_node()?;

b.start_node(NUMBER);
b.token(NUMBER, 5);
b.end_node()?;

b.end_node()?;

let tree = b.build()?;

assert_eq!(tree.span(), Span::new(0, 10));
```

[abstract syntax tree]: https://en.wikipedia.org/wiki/Abstract_syntax_tree
[`rowan`]: https://docs.rs/rowan/latest/rowan/
[Span]: https://docs.rs/syntree/latest/syntree/struct.Span.html
[calculator example]: https://github.com/udoprog/syntree/blob/main/examples/calculator.rs
[consist of anything you like]: https://github.com/udoprog/syntree/blob/main/examples/iterator.rs

License: MIT/Apache-2.0
