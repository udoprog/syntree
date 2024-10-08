//! Helper utilities for pretty-printing trees.

#![cfg(feature = "std")]
#![cfg_attr(docsrs, doc(cfg(feature = "std")))]

use std::fmt;
use std::io::{Error, Write};

use crate::flavor::Flavor;
use crate::span::Span;
use crate::tree::Tree;

/// Pretty-print a tree without a source.
///
/// This will replace all source references with `+`. If you have a source
/// available you can use [`print_with_source`] instead.
///
/// # Examples
///
/// ```
/// #[derive(Debug, Clone, Copy)]
/// enum Syntax {
///     NUMBER,
///     WHITESPACE,
///     OPERATOR,
///     PLUS,
/// }
///
/// use Syntax::*;
///
/// let tree = syntree::tree! {
///     NUMBER => {
///         (NUMBER, 3),
///     },
///     (WHITESPACE, 1),
///     OPERATOR => {
///         (PLUS, 1)
///     },
///     (WHITESPACE, 1),
///     NUMBER => {
///         (NUMBER, 2),
///     },
/// };
///
/// let mut s = Vec::new();
/// syntree::print::print(&mut s, &tree)?;
/// # let s = String::from_utf8(s)?;
/// # assert_eq!(s, "NUMBER@0..3\n  NUMBER@0..3 +\nWHITESPACE@3..4 +\nOPERATOR@4..5\n  PLUS@4..5 +\nWHITESPACE@5..6 +\nNUMBER@6..8\n  NUMBER@6..8 +\n");
/// # Ok::<_, Box<dyn core::error::Error>>(())
/// ```
///
/// This would write:
///
/// ```text
/// NUMBER@0..3
///   NUMBER@0..3 +
/// WHITESPACE@3..4 +
/// OPERATOR@4..5
///   PLUS@4..5 +
/// WHITESPACE@5..6 +
/// NUMBER@6..8
///   NUMBER@6..8 +
/// ```
pub fn print<O, T, F>(o: O, tree: &Tree<T, F>) -> Result<(), Error>
where
    O: Write,
    T: Copy + fmt::Debug,
    F: Flavor<Index: fmt::Display>,
{
    print_with_lookup(o, tree, |_| None)
}

/// Pretty-print a tree with the source spans printed.
///
/// # Examples
///
/// ```
/// #[derive(Debug, Clone, Copy)]
/// enum Syntax {
///     NUMBER,
///     WHITESPACE,
///     OPERATOR,
///     PLUS,
/// }
///
/// use Syntax::*;
///
/// let source = "128 + 64";
///
/// let tree = syntree::tree! {
///     NUMBER => {
///         (NUMBER, 3),
///     },
///     (WHITESPACE, 1),
///     OPERATOR => {
///         (PLUS, 1)
///     },
///     (WHITESPACE, 1),
///     NUMBER => {
///         (NUMBER, 2),
///     },
/// };
///
/// let mut s = Vec::new();
/// syntree::print::print_with_source(&mut s, &tree, source)?;
/// # let s = String::from_utf8(s)?;
/// # assert_eq!(s, "NUMBER@0..3\n  NUMBER@0..3 \"128\"\nWHITESPACE@3..4 \" \"\nOPERATOR@4..5\n  PLUS@4..5 \"+\"\nWHITESPACE@5..6 \" \"\nNUMBER@6..8\n  NUMBER@6..8 \"64\"\n");
/// # Ok::<_, Box<dyn core::error::Error>>(())
/// ```
///
/// This would write:
///
/// ```text
/// NUMBER@0..3
///   NUMBER@0..3 "128"
/// WHITESPACE@3..4 " "
/// OPERATOR@4..5
///   PLUS@4..5 "+"
/// WHITESPACE@5..6 " "
/// NUMBER@6..8
///   NUMBER@6..8 "64"
/// ```
pub fn print_with_source<O, T, F>(o: O, tree: &Tree<T, F>, source: &str) -> Result<(), Error>
where
    O: Write,
    T: Copy + fmt::Debug,
    F: Flavor<Index: fmt::Display>,
{
    print_with_lookup(o, tree, |span| source.get(span.range()))
}

fn print_with_lookup<'a, O, T, F>(
    mut o: O,
    tree: &Tree<T, F>,
    source: impl Fn(&Span<F::Index>) -> Option<&'a str>,
) -> Result<(), Error>
where
    O: Write,
    T: Copy + fmt::Debug,
    F: Flavor<Index: fmt::Display>,
{
    for (depth, node) in tree.walk().with_depths() {
        let n = (depth * 2) as usize;
        let data = node.value();
        let span = node.span();

        if node.has_children() {
            writeln!(o, "{:n$}{:?}@{}", "", data, span)?;
        } else if let Some(source) = source(span) {
            writeln!(o, "{:n$}{:?}@{} {:?}", "", data, span, source)?;
        } else {
            writeln!(o, "{:n$}{:?}@{} +", "", data, span)?;
        }
    }

    Ok(())
}
