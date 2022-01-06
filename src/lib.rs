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
//! syntree = "0.10.0"
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
//! NUMBER@0..3
//!   NUMBER@0..3 "128"
//! WHITESPACE@3..4 " "
//! OPERATOR@4..5
//!   PLUS@4..5 "+"
//! WHITESPACE@5..6 " "
//! NUMBER@6..8
//!   NUMBER@6..8 "64"
//! ```
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
//! Note that below, [`syntree::tree!`] is only a helper which simplifies
//! building trees for examples. It corresponds exactly to performing the
//! corresponding [`open`] and [`close`] calls on [`TreeBuilder`].
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
//! let mut b = TreeBuilder::new();
//!
//! b.open(NUMBER);
//! b.token(LIT, 1);
//! b.token(LIT, 3);
//!
//! b.open(NESTED);
//! b.token(LIT, 1);
//! b.close()?;
//!
//! b.close()?;
//!
//! let tree = b.build()?;
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
//! the use of a [handwritten pratt parser]. See the [calculator example] for a
//! complete use case.
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
//! In [Rune] this became apparent once we started [expanding macros]. Because
//! macros expand to things which do not reference source locations so we need
//! some other way to include what the tokens represent in the syntax trees.
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
//! instead of panicking.
//!
//! While on the surface this might seem like a minor difference in opinion on
//! whether programming mistakes should be errors or not, in my experience
//! parsers tend to be part of a crate in a larger project and errors are
//! triggered by edge cases in user-provided input.
//!
//! So let's say that [Rune] is embedded in [OxidizeBot]. There's a piece of
//! code in a user-provided script which triggers a bug in the rune compiler
//! that causes an illegal tree to be constructed. If tree construction simply
//! panics, the whole bot will go down. But if we instead propagate an error
//! this would have to be handled instead in however [OxidizeBot] uses [Rune].
//! For that instance it's simply more appropriate to log the error and unload
//! the script (and hopefully receive a bug report by the user) than to crash
//! the bot.
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
//! [`Tree`]: https://docs.rs/syntree/latest/syntree/struct.Tree.html
//! [`TreeBuilder`]: https://docs.rs/syntree/latest/syntree/struct.TreeBuilder.html
//! [`TreeError`]: https://docs.rs/syntree/latest/syntree/struct.TreeError.html
//! [abstract syntax trees]: https://en.wikipedia.org/wiki/Abstract_syntax_tree
//! [calculator example]: https://github.com/udoprog/syntree/blob/main/examples/calculator.rs
//! [could be whatever you want]: https://github.com/udoprog/syntree/blob/main/examples/iterator.rs
//! [expanding macros]: https://github.com/rune-rs/rune/blob/main/crates/rune-modules/src/core.rs#L36
//! [handwritten pratt parser]: https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
//! [kind-str]: https://github.com/rune-rs/rune/blob/e97a32e/crates/rune/src/ast/generated.rs#L4359
//! [Rune]: https://github.com/rune-rs/rune
//! [OxidizeBot]: https://github.com/udoprog/OxidizeBot
//! [synthetic_strings]: https://github.com/udoprog/syntree/blob/main/examples/synthetic_strings.rs

#![deny(missing_docs)]

#[macro_use]
mod macros;

mod non_max;

mod builder;

pub use self::builder::{Id, TreeBuilder};

mod error;
pub use self::error::TreeError;

mod nodes;
pub use self::nodes::Nodes;

pub mod print;

mod node;
pub use self::node::Node;

mod span;
pub use self::span::Span;

mod tree;
pub use self::tree::{Kind, Tree};

mod walk;
pub use self::walk::{Walk, WithDepths};

mod walk_events;
pub use self::walk_events::{Event, WalkEvents};

mod skip_tokens;
pub use self::skip_tokens::SkipTokens;

mod links;
