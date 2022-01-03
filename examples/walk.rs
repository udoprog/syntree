use std::collections::VecDeque;

use anyhow::Result;
use syntree::{Kind, Span, TreeBuilder};

#[derive(Debug, Clone, Copy)]
enum Syntax {
    Root,
    Operation,
    Number,
    Plus,
}

fn main() -> Result<()> {
    let mut b = TreeBuilder::new();

    b.start_node(Syntax::Root);
    b.start_node(Syntax::Operation);

    b.start_node(Syntax::Number);
    b.token(Syntax::Number, Span::new(0, 4));
    b.end_node()?;

    b.start_node(Syntax::Plus);
    b.token(Syntax::Plus, Span::new(4, 5));
    b.end_node()?;

    b.start_node(Syntax::Number);
    b.token(Syntax::Number, Span::new(5, 10));
    b.end_node()?;

    b.end_node()?;
    b.end_node()?;

    let tree = b.build()?;

    let mut stack = VecDeque::new();
    stack.extend(tree.children().map(|n| (true, 0, n)));

    while let Some((indent, n, node)) = stack.pop_front() {
        let data = node.data();

        if let Kind::Token(span) = node.kind() {
            println!("{:indent$}{:?} {}", "", data, span, indent = n);
            continue;
        }

        if node.is_empty() {
            println!("{:indent$}== {:?}", "", data, indent = n);
            continue;
        }

        if indent {
            println!("{:indent$}>> {:?}", "", data, indent = n);

            stack.push_front((false, n, node));

            for out in node.children().rev() {
                stack.push_front((true, n + 2, out));
            }
        } else {
            println!("{:indent$}<< {:?}", "", data, indent = n);
        }
    }

    Ok(())
}
