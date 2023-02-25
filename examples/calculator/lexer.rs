use crate::Syntax;

use Syntax::{CloseParen, Div, Error, Minus, Mul, Number, OpenParen, Plus, Pow, Whitespace};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Token {
    pub(crate) len: usize,
    pub(crate) syntax: Syntax,
}

pub(crate) struct Lexer<'a> {
    source: &'a str,
    cursor: usize,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        Self { source, cursor: 0 }
    }

    /// Peek the next character of input.
    fn peek(&self) -> Option<char> {
        self.source.get(self.cursor..)?.chars().next()
    }

    /// Step over the next character.
    fn step(&mut self) {
        if let Some(c) = self.peek() {
            self.cursor += c.len_utf8();
        }
    }

    /// Consume input until we hit non-numerics.
    fn consume_while(&mut self, cond: fn(char) -> bool) {
        while let Some(c) = self.peek() {
            if !cond(c) {
                break;
            }

            self.step();
        }
    }

    /// Get the next token.
    pub(crate) fn next(&mut self) -> Option<Token> {
        let c = self.peek()?;
        let start = self.cursor;

        let syntax = match c {
            c if c.is_whitespace() => {
                self.step();
                self.consume_while(char::is_whitespace);
                Whitespace
            }
            '+' => {
                self.step();
                Plus
            }
            '-' => {
                self.step();
                Minus
            }
            '*' => {
                self.step();
                Mul
            }
            '/' => {
                self.step();
                Div
            }
            '^' => {
                self.step();
                Pow
            }
            '(' => {
                self.step();
                OpenParen
            }
            ')' => {
                self.step();
                CloseParen
            }
            '0'..='9' => {
                self.step();
                self.consume_while(|c| c.is_ascii_digit());
                Number
            }
            _ => {
                self.consume_while(|c| !c.is_whitespace());
                Error
            }
        };

        let len = self.cursor.saturating_sub(start);
        Some(Token { len, syntax })
    }
}
