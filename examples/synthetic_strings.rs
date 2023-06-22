//! Example that showcases why it might be useful to smuggle more data into the
//! syntax tree.

use std::collections::HashMap;
use std::iter::Peekable;

use anyhow::Result;
use syntree::{print, Builder};

#[derive(Debug, Clone, Copy)]
enum Syntax {
    /// A string referenced somewhere else using the provided ID.
    Synthetic(usize),
    /// A literal string from the source.
    Literal,
    /// Whitespace.
    Whitespace,
    /// A lexer error.
    Error,
}

use Syntax::*;

#[derive(Default)]
struct Storage {
    lookup: HashMap<String, usize>,
    storage: Vec<String>,
}

impl Storage {
    /// Insert a string into synthetic storage.
    fn insert(&mut self, name: &str, value: &str) -> usize {
        let id = self.storage.len();
        self.storage.push(value.to_string());
        self.lookup.insert(name.to_owned(), id);
        id
    }

    /// Lookup a string from synthetic storage by name.
    fn lookup(&self, name: &str) -> Option<usize> {
        self.lookup.get(name).copied()
    }

    fn get(&self, id: usize) -> Option<&str> {
        Some(self.storage.get(id)?.as_str())
    }
}

fn lexer<'a>(source: &'a str, storage: &'a Storage) -> impl Iterator<Item = (Syntax, usize)> + 'a {
    let mut it = source.char_indices().peekable();
    let len = source.len();

    return std::iter::from_fn(move || {
        let (start, c) = it.next()?;

        let syntax = match c {
            c if c.is_whitespace() => {
                eat(&mut it, char::is_whitespace);
                Whitespace
            }
            '$' => {
                eat(&mut it, |c| c.is_ascii_lowercase());
                let end = it.peek().map_or(len, |(n, _)| *n);
                let id = &source[(start + 1)..end];

                match storage.lookup(id) {
                    Some(id) => Synthetic(id),
                    None => Error,
                }
            }
            'A'..='Z' | 'a'..='z' => {
                eat(&mut it, |c| c.is_ascii_alphabetic());
                Literal
            }
            _ => {
                eat(&mut it, |c| !c.is_whitespace());
                Error
            }
        };

        let end = it.peek().map_or(len, |(n, _)| *n);
        Some((syntax, end.saturating_sub(start)))
    });

    /// Consume all available whitespace.
    fn eat(it: &mut Peekable<impl Iterator<Item = (usize, char)>>, cond: fn(char) -> bool) {
        while it.peek().filter(|&(_, c)| cond(*c)).is_some() {
            it.next();
        }
    }
}

fn main() -> Result<()> {
    let source = std::env::args().skip(1).collect::<String>();

    let mut storage = Storage::default();
    storage.insert("world", "Earth");

    let lexer = lexer(&source, &storage);

    let mut tree = Builder::new();

    for (syntax, len) in lexer {
        tree.token(syntax, len)?;
    }

    let tree = tree.build()?;

    println!("Tree:");

    print::print_with_source(std::io::stdout(), &tree, &source)?;

    println!("Eval:");

    let mut count = 0usize;

    for node in tree.children() {
        let string = match *node.value() {
            Synthetic(id) => match storage.get(id) {
                Some(string) => string,
                None => {
                    println!("{} = {} (not found)", count, &source[node.range()]);
                    count += 1;
                    continue;
                }
            },
            Literal => &source[node.range()],
            Whitespace => continue,
            Error => {
                println!("Error: {}", &source[node.range()]);
                continue;
            }
        };

        println!("{count} = {string:?}");
        count += 1;
    }

    Ok(())
}
