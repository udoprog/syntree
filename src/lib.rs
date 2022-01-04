//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/syntree-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/syntree)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/syntree.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/syntree)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-syntree-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/syntree)
//! [<img alt="build status" src="https://img.shields.io/github/workflow/status/udoprog/syntree/CI/main?style=for-the-badge" height="20">](https://github.com/udoprog/syntree/actions?query=branch%3Amain)
//!
//! A memory efficient syntax tree.
//!
//! This crate provides a tree structure which always is contiguously stored and
//! manipulated in memory. It provides similar APIs as [`rowan`] and is intended
//! to be an efficient replacement for it (read more below).
//!
//! <br>
//!
//! # Usage
//!
//! Add `syntree` to your crate:
//!
//! ```toml
//! syntree = "0.4.3"
//! ```
//!
//! If you want a complete sample for how `syntree` can be used for parsing, see
//! the [calculator example].
//!
//! <br>
//!
//! ## Syntax trees
//!
//! This crate provides a way to efficiently model [abstract syntax trees]. The
//! nodes of the tree are typically represented by variants in an enum, but
//! [could be whatever you want].
//!
//! Each tree consists of *nodes* and *tokens*. Nodes are intermediary elements
//! in the tree which encapsulate zero or more other nodes or tokens, while
//! tokens are leaf elements representing exact source locations.
//!
//! An example tree for the simple expression `128 + 64` could be represented
//! like this:
//!
//! > Try it for yourself with:
//! >
//! > ```sh
//! > cargo run --example calculator -- "128 + 64"
//! > ```
//!
//! ```text
//! >> NUMBER
//!   NUMBER@0..3 "128"
//! << NUMBER
//! WHITESPACE@3..4 " "
//! >> OPERATOR
//!   PLUS@4..5 "+"
//! << OPERATOR
//! WHITESPACE@5..6 " "
//! >> NUMBER
//!   NUMBER@6..8 "64"
//! << NUMBER
//! ```
//!
//! The primary difference between `syntree` and [`rowan`] is that *we don't
//! store the original source* in the syntax tree. Instead, the user of the
//! library is responsible for providing it as necessary. Like when calling
//! [print_with_source].
//!
//! The API for constructing a syntax tree is provided through [TreeBuilder]
//! which provides streaming builder methods. Internally the builder is
//! represented as a contiguous slab of memory. Once a tree is built the
//! structure of the tree can be queried through the [Tree] type.
//!
//! ```
//! use syntree::{Span, TreeBuilder};
//!
//! #[derive(Debug, Clone, Copy)]
//! enum Syntax {
//!     OPERATION,
//!     NUMBER,
//!     PLUS,
//!     MINUS,
//!     WHITESPACE,
//! }
//!
//! use Syntax::*;
//!
//! # fn main() -> anyhow::Result<()> {
//! let mut b = TreeBuilder::new();
//!
//! b.open(OPERATION);
//!
//! b.open(OPERATION);
//!
//! b.open(NUMBER);
//! b.token(NUMBER, 4);
//! b.close()?;
//!
//! b.token(WHITESPACE, 1);
//!
//! b.open(PLUS);
//! b.token(PLUS, 1);
//! b.close()?;
//!
//! b.token(WHITESPACE, 1);
//!
//! b.open(NUMBER);
//! b.token(NUMBER, 5);
//! b.close()?;
//!
//! b.close()?;
//!
//! b.token(WHITESPACE, 1);
//!
//! b.open(MINUS);
//! b.token(MINUS, 1);
//! b.close()?;
//!
//! b.token(WHITESPACE, 1);
//!
//! b.open(NUMBER);
//! b.token(NUMBER, 3);
//! b.close()?;
//!
//! b.close()?;
//!
//! let tree = b.build()?;
//!
//! assert_eq!(tree.span().range(), 0..18);
//! assert_eq!(tree.children().count(), 1);
//! # Ok(()) }
//! ```
//!
//! [abstract syntax trees]: https://en.wikipedia.org/wiki/Abstract_syntax_tree
//! [`rowan`]: https://docs.rs/rowan/latest/rowan/
//! [Span]: https://docs.rs/syntree/latest/syntree/struct.Span.html
//! [calculator example]: https://github.com/udoprog/syntree/blob/main/examples/calculator.rs
//! [could be whatever you want]: https://github.com/udoprog/syntree/blob/main/examples/iterator.rs
//! [print_with_source]: https://docs.rs/syntree/latest/syntree/print/fn.print_with_source.html
//! [TreeBuilder]: https://docs.rs/syntree/latest/syntree/struct.TreeBuilder.html
//! [Tree]: https://docs.rs/syntree/latest/syntree/struct.Tree.html

#![deny(missing_docs)]

#[macro_use]
mod macros;

mod non_max;

mod builder;

pub use self::builder::{BuildError, CloseError, Id, TreeBuilder};

mod convert;

pub mod print;

mod node;
pub use self::node::Node;

mod span;
pub use self::span::Span;

mod tree;
pub use self::tree::{Children, Kind, Tree, Walk, WithoutTokens};
