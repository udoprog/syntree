//! Helper utilities for pretty-printing trees.

use std::collections::VecDeque;
use std::fmt::Debug;
use std::io::{Error, Write};

use crate::{Kind, Span, Tree};

/// Pretty-print a tree.
pub fn print<O, T>(o: &mut O, tree: &Tree<T>) -> Result<(), Error>
where
    O: Write,
    T: Debug,
{
    print_with_callback(o, tree, |o, data, span, indent| {
        writeln!(o, "{:indent$}{:?} {}", "", data, span, indent = indent)
    })
}

/// Pretty-print a tree with the source spans printed.
pub fn print_with_source<O, T>(o: &mut O, tree: &Tree<T>, source: &str) -> Result<(), Error>
where
    O: Write,
    T: Debug,
{
    print_with_callback(o, tree, |o, data, span, indent| {
        if let Some(source) = source.get(span.range()) {
            writeln!(
                o,
                "{:indent$}{:?}@{span} {:?}",
                "",
                data,
                source,
                span = span,
                indent = indent
            )
        } else {
            writeln!(
                o,
                "{:indent$}{:?}@{span}",
                "",
                data,
                span = span,
                indent = indent
            )
        }
    })
}

fn print_with_callback<O, T>(
    o: &mut O,
    tree: &Tree<T>,
    token: impl Fn(&mut O, &T, Span, usize) -> Result<(), Error>,
) -> Result<(), Error>
where
    O: Write,
    T: Debug,
{
    let mut stack = VecDeque::new();
    stack.extend(tree.children_with_tokens().map(|n| (true, 0, n)));

    while let Some((indent, n, node)) = stack.pop_front() {
        let data = node.data();

        if let Kind::Token(span) = node.kind() {
            token(o, data, span, n)?;
            continue;
        }

        if node.is_empty() {
            writeln!(o, "{:indent$}== {:?}", "", data, indent = n)?;
            continue;
        }

        if indent {
            writeln!(o, "{:indent$}>> {:?}", "", data, indent = n)?;

            stack.push_front((false, n, node));

            for out in node.children_with_tokens().rev() {
                stack.push_front((true, n + 2, out));
            }
        } else {
            writeln!(o, "{:indent$}<< {:?}", "", data, indent = n)?;
        }
    }

    Ok(())
}
