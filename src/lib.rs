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
//! syntree = "0.11.1"
//! ```
//!
//! If you want a complete sample for how `syntree` can be used for parsing, see
//! the [calculator example][calculator].
//!
//! <br>
//!
//! ## Enabling `syntree_compact`
//!
//! We support a configuration option to reduce the size of the tree in memory.
//! It changes the tree from using `usize` as indexes to use `u32` which saves 4
//! bytes per reference on 64-bit platforms.
//!
//! This can be enabled by setting `--cfg syntree_compact` while building and
//! might improve performance due to allowing nodes to fit neatly on individual
//! cache lines.
//!
//! ```sh
//! RUSTFLAGS="--cfg syntree_compact" cargo build
//! ```
//!
//! <br>
//!
//! ## Syntax trees
//!
//! This crate provides a way to efficiently model [abstract syntax trees]. The
//! nodes of the tree are typically represented by variants in an enum, but
//! [could be whatever you want][any-syntax].
//!
//! Each tree consists of *nodes* and *tokens*. Siblings are intermediary
//! elements in the tree which encapsulate zero or more other nodes or tokens,
//! while tokens are leaf elements representing exact source locations.
//!
//! An example tree for the simple expression `256 / 2 + 64 * 2` could be
//! represented like this:
//!
//! ```text
//! OPERATION@0..16
//!   NUMBER@0..3
//!     NUMBER@0..3 "256"
//!   WHITESPACE@3..4 " "
//!   OPERATOR@4..5
//!     DIV@4..5 "/"
//!   WHITESPACE@5..6 " "
//!   NUMBER@6..7
//!     NUMBER@6..7 "2"
//!   WHITESPACE@7..8 " "
//!   OPERATOR@8..9
//!     PLUS@8..9 "+"
//!   WHITESPACE@9..10 " "
//!   OPERATION@10..16
//!     NUMBER@10..12
//!       NUMBER@10..12 "64"
//!     WHITESPACE@12..13 " "
//!     OPERATOR@13..14
//!       MUL@13..14 "*"
//!     WHITESPACE@14..15 " "
//!     NUMBER@15..16
//!       NUMBER@15..16 "2"
//! ```
//!
//! > Try it for yourself with:
//! >
//! > ```sh
//! > cargo run --example calculator -- "256 / 2 + 64 * 2"
//! > ```
//!
//! The primary difference between `syntree` and [`rowan`] is that *we don't
//! store the original source* in the syntax tree. Instead, the user of the
//! library is responsible for providing it as necessary. Like when calling
//! [`print_with_source`].
//!
//! The API for constructing a syntax tree is provided through [`TreeBuilder`]
//! which provides streaming builder methods. Internally the builder is
//! represented as a contiguous slab of memory. Once a tree is built the
//! structure of the tree can be queried through the [`Tree`] type.
//!
//! Note that [`syntree::tree!`] is only a helper which simplifies building
//! trees for examples. It corresponds exactly to performing [`open`],
//! [`close`], and [`token`] calls on [`TreeBuilder`] as specified.
//!
//! ```
//! use syntree::{Span, TreeBuilder};
//!
//! #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//! enum Syntax {
//!     NUMBER,
//!     LIT,
//!     NESTED,
//! }
//!
//! use Syntax::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut tree = TreeBuilder::new();
//!
//! tree.open(NUMBER)?;
//! tree.token(LIT, 1)?;
//! tree.token(LIT, 3)?;
//!
//! tree.open(NESTED)?;
//! tree.token(LIT, 1)?;
//! tree.close()?;
//!
//! tree.close()?;
//!
//! let tree = tree.build()?;
//!
//! let expected = syntree::tree! {
//!     NUMBER => {
//!         (LIT, 1),
//!         (LIT, 3),
//!         NESTED => {
//!             (LIT, 1)
//!         }
//!     }
//! };
//!
//! assert_eq!(tree, expected);
//!
//! let number = tree.first().ok_or("missing number")?;
//! assert_eq!(number.span(), Span::new(0, 5));
//! # Ok(()) }
//! ```
//!
//! Note how the resulting [`Span`] for `NUMBER` corresponds to the full span of
//! its `LIT` children. Including the ones within `NESTED`.
//!
//! Trees are usually constructed by parsing an input. This library encourages
//! the use of a [handwritten pratt parser][pratt]. See the [calculator
//! example][calculator] for a complete use case.
//!
//! <br>
//!
//! ## Why not `rowan`?
//!
//! I love [`rowan`]. It's the reason why I started this project. But this crate
//! still exists for a few philosophical differences that would be hard to
//! reconcile directly in `rowan`.
//!
//! `rowan` only supports adding types which in some way can be coerced into an
//! `repr(u16)` as part of the syntax tree. I think this decision is reasonable,
//! but it precludes you from designing trees which contain anything else other
//! than source references without having to perform some form of indirect
//! lookup on the side. This is something I need in order to move [Rune] to
//! lossless syntax trees like (see [the representation of `Kind::Str`
//! enum][kind-str]).
//!
//! So consider the following tokens:
//!
//! ```
//! #[derive(Debug, Clone, Copy)]
//! enum Syntax {
//!     /// A string referenced somewhere else using the provided ID.
//!     SYNTHETIC(Option<usize>),
//!     /// A literal string from the source.
//!     LITERAL,
//!     /// Whitespace.
//!     WHITESPACE,
//!     /// A lexer error.
//!     ERROR,
//! }
//! ```
//!
//! You can see the [full `synthetic_strings` example][synthetic_strings] for
//! how this might be used. But not only can the `SYNTHETIC` token correspond to
//! some source location (as it should because it was expanded from one!). It
//! also directly represents that it's *not* a literal string referencing a
//! source location.
//!
//! In [Rune] this became apparent once we started [expanding
//! macros][rune-macros]. Because macros expand to things which do not reference
//! source locations so we need some other way to include what the tokens
//! represent in the syntax trees.
//!
//! You can try a *very* simple lex-time variable expander in the
//! [`synthetic_strings` example][synthetic_strings]:
//!
//! ```sh
//! cargo run --example synthetic_strings -- "Hello $world"
//! ```
//!
//! Which would output:
//!
//! ```text
//! Tree:
//! LITERAL@0..5 "Hello"
//! WHITESPACE@5..6 " "
//! SYNTHETIC(Some(0))@6..12 "$world"
//! Eval:
//! 0 = "Hello"
//! 1 = "Earth"
//! ```
//!
//! So in essense `syntree` doesn't believe you need to store strings in the
//! tree itself. Even if you want to deduplicate string storage. All of that can
//! be done on the side and encoded into the syntax tree as you wish.
//!
//! <br>
//!
//! ### Errors instead of panics
//!
//! Another point where this crate differs is that we rely on propagating a
//! [`TreeError`] during tree construction if the API is used incorrectly
//! *instead* of panicking.
//!
//! While on the surface this might seem like a minor difference in opinion on
//! whether programming mistakes should be errors or not. In my experience
//! parsers tend to be part of a crate in a larger project. And errors triggered
//! by edge cases in user-provided input that once encountered can usually be
//! avoided.
//!
//! So let's say [Rune] is embedded in [OxidizeBot] and there's a piece of code
//! in a user-provided script which triggers a bug in the rune compiler. Which
//! in turn causes an illegal tree to be constructed. If tree construction
//! simply panics, the whole bot will go down. But if we instead propagate an
//! error this would have to be handled in [OxidizeBot] which could panic if it
//! wanted to. In this instance it's simply more appropriate to log the error
//! and unload the script (and hopefully receive a bug report!) than to crash
//! the bot.
//!
//! Rust has great diagnostics for indicating that results should be handled,
//! and while it is [more awkward to deal with results][syntree-math] than [to
//! simply panic][rowan-math] I believe that the end result is more robust
//! software.
//!
//! <br>
//!
//! ## Performance and memory use
//!
//! The only goal in terms of performance is to be as performant as `rowan`. And
//! cursory testing shows `syntree` to be a bit faster on synthetic workloads
//! which can probably be solely attributed to storing and manipulating the
//! entire tree in a single contiguous memory region. This might or might not
//! change in the future, depending on if the needs for `syntree` changes. While
//! performance is important, it *is not* a primary focus.
//!
//! I also expect (but haven't verified) that `syntree` could end up having a
//! similarly low memory profile as `rowan` which performs node deduplication.
//! The one caveat is that it depends on how the original source is stored and
//! queried. Something which `rowan` solves for you, but `syntree` leaves as an
//! exercise to the reader.
//!
//! [`close`]: https://docs.rs/syntree/latest/syntree/struct.TreeBuilder.html#method.close
//! [`open`]: https://docs.rs/syntree/latest/syntree/struct.TreeBuilder.html#method.open
//! [`print_with_source`]: https://docs.rs/syntree/latest/syntree/print/fn.print_with_source.html
//! [`rowan`]: https://docs.rs/rowan/latest/rowan/
//! [`Span`]: https://docs.rs/syntree/latest/syntree/struct.Span.html
//! [`syntree::tree!`]: https://docs.rs/syntree/latest/syntree/macro.tree.html
//! [`token`]: https://docs.rs/syntree/latest/syntree/struct.TreeBuilder.html#method.token
//! [`Tree`]: https://docs.rs/syntree/latest/syntree/struct.Tree.html
//! [`TreeBuilder`]: https://docs.rs/syntree/latest/syntree/struct.TreeBuilder.html
//! [`TreeError`]: https://docs.rs/syntree/latest/syntree/enum.TreeError.html
//! [abstract syntax trees]: https://en.wikipedia.org/wiki/Abstract_syntax_tree
//! [any-syntax]: https://github.com/udoprog/syntree/blob/main/examples/iterator.rs
//! [calculator]: https://github.com/udoprog/syntree/blob/main/examples/calculator
//! [kind-str]: https://github.com/rune-rs/rune/blob/e97a32e/crates/rune/src/ast/generated.rs#L4359
//! [OxidizeBot]: https://github.com/udoprog/OxidizeBot
//! [pratt]: https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
//! [rowan-math]: https://github.com/rust-analyzer/rowan/blob/master/examples/math.rs
//! [rune-macros]: https://github.com/rune-rs/rune/blob/main/crates/rune-modules/src/core.rs#L36
//! [Rune]: https://github.com/rune-rs/rune
//! [synthetic_strings]: https://github.com/udoprog/syntree/blob/main/examples/synthetic_strings.rs
//! [syntree-math]: https://github.com/udoprog/syntree/blob/main/examples/math.rs

#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[macro_use]
mod macros;
mod ancestors;
mod builder;
mod change_set;
mod children;
mod error;
mod links;
mod node;
mod non_max;
pub mod print;
mod siblings;
mod skip_tokens;
mod span;
mod tree;
mod walk;
mod walk_events;

pub use self::ancestors::Ancestors;
pub use self::builder::{Checkpoint, Id, TreeBuilder};
pub use self::change_set::ChangeSet;
pub use self::children::Children;
pub use self::error::TreeError;
pub use self::node::Node;
pub use self::siblings::Siblings;
pub use self::skip_tokens::SkipTokens;
pub use self::span::Span;
pub use self::tree::{Kind, Tree};
pub use self::walk::{Walk, WithDepths};
pub use self::walk_events::{Event, WalkEvents};
