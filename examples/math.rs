//! Example converted from https://github.com/rust-analyzer/rowan/blob/master/examples/math.rs

use anyhow::Result;
use std::iter::Peekable;
use syntree::{print, Builder, Error, Tree};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
enum Syntax {
    ADD,
    SUB,
    MUL,
    DIV,

    WHITESPACE,

    NUMBER,
    ERROR,
    OPERATION,
    ROOT,
}

use Syntax::{ADD, DIV, ERROR, MUL, NUMBER, OPERATION, ROOT, SUB, WHITESPACE};

struct Parser<I: Iterator<Item = (Syntax, usize)>> {
    builder: Builder<Syntax>,
    iter: Peekable<I>,
}

impl<I: Iterator<Item = (Syntax, usize)>> Parser<I> {
    fn peek(&mut self) -> Result<Option<Syntax>, Error> {
        while self.iter.peek().map_or(false, |&(t, _)| t == WHITESPACE) {
            self.bump()?;
        }
        Ok(self.iter.peek().map(|&(t, _)| t))
    }

    fn bump(&mut self) -> Result<(), Error> {
        if let Some((token, len)) = self.iter.next() {
            self.builder.token(token, len)?;
        }

        Ok(())
    }

    fn parse_val(&mut self) -> Result<(), Error> {
        match self.peek()? {
            Some(NUMBER) => {
                self.bump()?;
            }
            _ => {
                self.builder.open(ERROR)?;
                self.bump()?;
                self.builder.close()?;
            }
        }

        Ok(())
    }

    fn handle_operation(
        &mut self,
        tokens: &[Syntax],
        next: fn(&mut Self) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let c = self.builder.checkpoint()?;
        next(self)?;

        while self.peek()?.map_or(false, |t| tokens.contains(&t)) {
            self.bump()?;
            next(self)?;
            self.builder.close_at(&c, OPERATION)?;
        }

        Ok(())
    }

    fn parse_mul(&mut self) -> Result<(), Error> {
        self.handle_operation(&[MUL, DIV], Self::parse_val)
    }

    fn parse_add(&mut self) -> Result<(), Error> {
        self.handle_operation(&[ADD, SUB], Self::parse_mul)
    }

    fn parse(mut self) -> Result<Tree<Syntax>, Error> {
        self.builder.open(ROOT)?;
        self.parse_add()?;
        self.builder.close()?;
        self.builder.build()
    }
}

fn lexer(source: &str) -> impl Iterator<Item = (Syntax, usize)> + '_ {
    let mut it = source.char_indices().peekable();
    let len = source.len();

    return std::iter::from_fn(move || {
        let (start, c) = it.next()?;

        let syntax = match c {
            c if c.is_whitespace() => {
                eat(&mut it, char::is_whitespace);
                WHITESPACE
            }
            '+' => ADD,
            '-' => SUB,
            '/' => DIV,
            '*' => MUL,
            '0'..='9' => {
                eat(&mut it, |c| matches!(c, '0'..='9' | '.'));
                NUMBER
            }
            _ => {
                eat(&mut it, |c| !c.is_whitespace());
                ERROR
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

    let iter = lexer(&source);

    let parser = Parser {
        builder: Builder::new(),
        iter: iter.peekable(),
    };

    let tree = parser.parse()?;

    print::print_with_source(std::io::stdout(), &tree, &source)?;
    Ok(())
}
