//! Helper utilities for pretty-printing trees.

use std::collections::VecDeque;
use std::fmt::Debug;
use std::io::{Error, Write};

use crate::{Kind, Tree};

/// Pretty-print a tree.
pub fn print<O, T>(o: &mut O, tree: &Tree<T>) -> Result<(), Error>
where
    O: Write,
    T: Debug,
{
    let mut stack = VecDeque::new();
    stack.extend(tree.children().map(|n| (true, 0, n)));

    while let Some((indent, n, node)) = stack.pop_front() {
        let data = node.data();

        if let Kind::Token(span) = node.kind() {
            writeln!(o, "{:indent$}{:?} {}", "", data, span, indent = n)?;
            continue;
        }

        if node.is_empty() {
            writeln!(o, "{:indent$}== {:?}", "", data, indent = n)?;
            continue;
        }

        if indent {
            writeln!(o, "{:indent$}>> {:?}", "", data, indent = n)?;

            stack.push_front((false, n, node));

            for out in node.children().rev() {
                stack.push_front((true, n + 2, out));
            }
        } else {
            writeln!(o, "{:indent$}<< {:?}", "", data, indent = n)?;
        }
    }

    Ok(())
}
