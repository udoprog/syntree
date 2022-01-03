//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/syntree-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/syntree)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/syntree.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/syntree)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-syntree-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/syntree)
//! [<img alt="build status" src="https://img.shields.io/github/workflow/status/udoprog/syntree/CI/main?style=for-the-badge" height="20">](https://github.com/udoprog/syntree/actions?query=branch%3Amain)
//!
//! A memory efficient syntax tree.
//!
//! ```
//! use syntree::{Span, TreeBuilder};
//!
//! #[derive(Debug, Clone, Copy)]
//! enum Syntax {
//!     Root,
//!     Operation,
//!     Number,
//!     Plus,
//! }
//!
//! # fn main() -> anyhow::Result<()> {
//! let mut b = TreeBuilder::new();
//!
//! b.start_node(Syntax::Root);
//! b.start_node(Syntax::Operation);
//!
//! b.start_node(Syntax::Number);
//! b.token(Syntax::Number, Span::new(0, 4));
//! b.end_node()?;
//!
//! b.start_node(Syntax::Plus);
//! b.token(Syntax::Plus, Span::new(4, 5));
//! b.end_node()?;
//!
//! b.start_node(Syntax::Number);
//! b.token(Syntax::Number, Span::new(5, 10));
//! b.end_node()?;
//!
//! b.end_node()?;
//! b.end_node()?;
//!
//! let tree = b.build()?;
//!
//! assert_eq!(tree.span(), Some(Span::new(0, 10)));
//! # Ok(()) }
//! ```

#![deny(missing_docs)]

mod builder;
pub use self::builder::TreeBuilder;

mod convert;

mod span;
pub use self::span::Span;

mod tree;
pub use self::tree::{Kind, Tree};

/// The identifier of a node as returned by functions such as
/// [TreeBuilder::start_node] or [TreeBuilder::token].
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Id(u32);
