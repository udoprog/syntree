use crate::lexer::{Lexer, Token};
use crate::Syntax;
use anyhow::Result;
use syntree::{TreeBuilder, TreeError};
use Syntax::{EOF, WHITESPACE};

/// Parser and lexer baked into one.
pub(crate) struct Parser<'a> {
    lexer: Lexer<'a>,
    pub(crate) tree: TreeBuilder<Syntax>,
    // One token of lookahead.
    buf: Option<Token>,
}

impl<'a> Parser<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        Self {
            lexer: Lexer::new(source),
            tree: TreeBuilder::new(),
            buf: None,
        }
    }

    /// Peek the next token.
    pub fn peek(&mut self) -> Result<Token, TreeError> {
        loop {
            // Fill up buffer.
            self.fill()?;

            if let Some(tok) = self.buf {
                return Ok(tok);
            }

            return Ok(Token {
                len: 0,
                syntax: EOF,
            });
        }
    }

    /// Test if the parser is currently at EOF.
    pub(crate) fn is_eof(&mut self) -> Result<bool, TreeError> {
        Ok(self.peek()?.syntax == EOF)
    }

    /// Try to eat the given sequence of syntax as the given node `what`.
    pub(crate) fn eat(&mut self, what: Syntax) -> Result<bool, TreeError> {
        if self.peek()?.syntax != what {
            return Ok(false);
        }

        let tok = self.step()?;
        self.tree.token(tok.syntax, tok.len)?;
        Ok(true)
    }

    /// Consume a token.
    pub(crate) fn token(&mut self) -> Result<(), TreeError> {
        let tok = self.step()?;
        self.tree.token(tok.syntax, tok.len)?;
        Ok(())
    }

    /// Bump the current input as the given syntax.
    pub(crate) fn bump(&mut self, what: Syntax) -> Result<(), TreeError> {
        let tok = self.step()?;
        self.tree.open(what)?;
        self.tree.token(tok.syntax, tok.len)?;
        self.tree.close()?;
        Ok(())
    }

    /// Advance until one of `any` matches.
    pub(crate) fn advance_until(&mut self, any: &[Syntax]) -> Result<(), TreeError> {
        // Consume until we see another Number (or EOF) for some primitive
        // error recovery.
        loop {
            let t = self.peek()?;

            if t.syntax == EOF || any.iter().any(|s| *s == t.syntax) {
                break;
            }

            self.tree.token(t.syntax, t.len)?;
            self.step()?;
        }

        Ok(())
    }

    /// Consume the head token.
    pub(crate) fn step(&mut self) -> Result<Token, TreeError> {
        let tok = self.peek()?;
        self.buf.take();
        Ok(tok)
    }

    fn fill(&mut self) -> Result<(), TreeError> {
        while self.buf.is_none() {
            let tok = match self.lexer.next() {
                Some(tok) => tok,
                None => break,
            };

            // Consume whitespace transparently.
            if matches!(tok.syntax, WHITESPACE) {
                self.tree.token(tok.syntax, tok.len)?;
                continue;
            }

            self.buf = Some(tok);
        }

        Ok(())
    }
}
