//! Example converted from https://github.com/rust-analyzer/rowan/blob/master/examples/math.rs

use std::iter::Peekable;
use syntree::{print, Tree, TreeBuilder};

type BoxedError = Box<dyn std::error::Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
enum SyntaxKind {
    WHITESPACE = 0,

    ADD,
    SUB,
    MUL,
    DIV,

    NUMBER,
    ERROR,
    OPERATION,
    ROOT,
}

use SyntaxKind::*;

struct Parser<I: Iterator<Item = (SyntaxKind, usize)>> {
    builder: TreeBuilder<SyntaxKind>,
    iter: Peekable<I>,
}

impl<I: Iterator<Item = (SyntaxKind, usize)>> Parser<I> {
    fn peek(&mut self) -> Option<SyntaxKind> {
        while self
            .iter
            .peek()
            .map(|&(t, _)| t == WHITESPACE)
            .unwrap_or(false)
        {
            self.bump();
        }
        self.iter.peek().map(|&(t, _)| t)
    }

    fn bump(&mut self) {
        if let Some((token, len)) = self.iter.next() {
            self.builder.token(token.into(), len);
        }
    }

    fn parse_val(&mut self) -> Result<(), BoxedError> {
        match self.peek() {
            Some(NUMBER) => self.bump(),
            _ => {
                self.builder.open(ERROR);
                self.bump();
                self.builder.close()?;
            }
        }

        Ok(())
    }

    fn handle_operation(
        &mut self,
        tokens: &[SyntaxKind],
        next: fn(&mut Self) -> Result<(), BoxedError>,
    ) -> Result<(), BoxedError> {
        let checkpoint = self.builder.checkpoint();
        next(self)?;
        while self.peek().map(|t| tokens.contains(&t)).unwrap_or(false) {
            self.bump();
            next(self)?;
            self.builder.close_at(checkpoint, OPERATION);
        }

        Ok(())
    }

    fn parse_mul(&mut self) -> Result<(), BoxedError> {
        self.handle_operation(&[MUL, DIV], Self::parse_val)
    }

    fn parse_add(&mut self) -> Result<(), BoxedError> {
        self.handle_operation(&[ADD, SUB], Self::parse_mul)
    }

    fn parse(mut self) -> Result<Tree<SyntaxKind>, BoxedError> {
        self.builder.open(ROOT);
        self.parse_add()?;
        self.builder.close()?;
        Ok(self.builder.build()?)
    }
}

fn lexer(source: &str) -> impl Iterator<Item = (SyntaxKind, usize)> + '_ {
    let mut it = source.char_indices().peekable();
    let len = source.len();

    return std::iter::from_fn(move || {
        let (start, c) = it.next()?;

        if c.is_whitespace() {
            consume_while(&mut it, char::is_whitespace);
            let end = next(&mut it).unwrap_or(len);
            return Some((WHITESPACE, end.saturating_sub(start)));
        }

        let syntax = match c {
            '+' => ADD,
            '-' => SUB,
            '/' => DIV,
            '*' => MUL,
            _ => {
                consume_while(&mut it, |c| !c.is_whitespace());
                NUMBER
            }
        };

        let end = next(&mut it).unwrap_or(len);
        Some((syntax, end.saturating_sub(start)))
    });

    fn next(it: &mut Peekable<impl Iterator<Item = (usize, char)>>) -> Option<usize> {
        it.peek().map(|(n, _)| *n)
    }

    /// Consume all available whitespace.
    fn consume_while(
        it: &mut Peekable<impl Iterator<Item = (usize, char)>>,
        condition: fn(char) -> bool,
    ) {
        while let Some(&(_, c)) = it.peek() {
            if !condition(c) {
                break;
            }

            it.next();
        }
    }
}

fn main() -> Result<(), BoxedError> {
    let source = std::env::args().skip(1).collect::<String>();

    let iter = lexer(&source);

    let parser = Parser {
        builder: TreeBuilder::new(),
        iter: iter.peekable(),
    };

    let tree = parser.parse()?;

    print::print_with_source(std::io::stdout(), &tree, &source)?;
    Ok(())
}
