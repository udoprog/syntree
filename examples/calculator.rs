/// A simple calculator only capable of addition and subtraction.
use anyhow::Result;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFile;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use std::io::Write;

fn main() -> Result<()> {
    let source = std::env::args().skip(1).collect::<String>();

    let mut p = parsing::Parser::new(&source);
    grammar::root(&mut p)?;

    let tree = p.tree.build()?;

    let mut o = StandardStream::stderr(ColorChoice::Always);
    let config = codespan_reporting::term::Config::default();

    match eval::eval(&tree, &source) {
        Ok(output) => {
            writeln!(o, "Output = {}", output)?;
        }
        Err(e) => {
            let file = SimpleFile::new("<cli>", &source);

            let diagnostic = Diagnostic::error()
                .with_message("parse error")
                .with_labels(vec![
                    Label::primary((), e.span.range()).with_message(e.to_string())
                ]);

            term::emit(&mut o.lock(), &config, &file, &diagnostic)?;
        }
    }

    writeln!(o, "# Tree:")?;
    syntree::print::print_with_source(o.lock(), &tree, &source)?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Syntax {
    NUMBER,
    PLUS,
    MINUS,
    ERROR,
    WHITESPACE,
    EOF,

    OPERATOR,
}

mod parsing {
    use crate::Syntax;
    use anyhow::Result;
    use syntree::{TreeBuilder, TreeError};
    use Syntax::*;

    #[derive(Debug, Clone, Copy)]
    pub(crate) struct Token {
        pub(crate) len: usize,
        pub(crate) syntax: Syntax,
    }

    struct Lexer<'a> {
        source: &'a str,
        cursor: usize,
    }

    impl<'a> Lexer<'a> {
        fn new(source: &'a str) -> Self {
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
        fn next(&mut self) -> Option<Token> {
            let c = self.peek()?;
            let start = self.cursor;

            let syntax = match c {
                c if c.is_whitespace() => {
                    self.step();
                    self.consume_while(char::is_whitespace);
                    WHITESPACE
                }
                '+' => {
                    self.step();
                    PLUS
                }
                '-' => {
                    self.step();
                    MINUS
                }
                '0'..='9' => {
                    self.step();
                    self.consume_while(|c| matches!(c, '0'..='9'));
                    NUMBER
                }
                _ => {
                    self.consume_while(|c| !c.is_whitespace());
                    ERROR
                }
            };

            let len = self.cursor.saturating_sub(start);
            Some(Token { len, syntax })
        }
    }

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
        pub fn peek(&mut self) -> Token {
            loop {
                // Fill up buffer.
                self.fill();

                if let Some(tok) = self.buf {
                    return tok;
                }

                return Token {
                    len: 0,
                    syntax: EOF,
                };
            }
        }

        /// Test if the parser is currently at EOF.
        pub(crate) fn is_eof(&mut self) -> bool {
            self.peek().syntax == EOF
        }

        /// Try to eat the given sequence of syntax as the given node `what`.
        pub(crate) fn eat(&mut self, what: Syntax) -> Result<bool, TreeError> {
            if self.peek().syntax != what {
                return Ok(false);
            }

            let tok = self.step();
            self.tree.open(what);
            self.tree.token(tok.syntax, tok.len);
            self.tree.close()?;
            Ok(true)
        }

        /// Bump the current input as the given syntax.
        pub(crate) fn bump(&mut self, what: Syntax) -> Result<()> {
            let tok = self.step();
            self.tree.open(what);
            self.tree.token(tok.syntax, tok.len);
            self.tree.close()?;
            Ok(())
        }

        /// Advance until one of `any` matches.
        pub(crate) fn advance_until(&mut self, any: &[Syntax]) {
            // Consume until we see another Number (or EOF) for some primitive
            // error recovery.
            loop {
                let t = self.peek();

                if t.syntax == EOF || any.iter().any(|s| *s == t.syntax) {
                    break;
                }

                self.tree.token(t.syntax, t.len);
                self.step();
            }
        }

        /// Consume the head token.
        fn step(&mut self) -> Token {
            let tok = self.peek();
            self.buf.take();
            tok
        }

        fn fill(&mut self) {
            while self.buf.is_none() {
                let tok = match self.lexer.next() {
                    Some(tok) => tok,
                    None => break,
                };

                // Consume whitespace transparently.
                if matches!(tok.syntax, WHITESPACE) {
                    self.tree.token(tok.syntax, tok.len);
                    continue;
                }

                self.buf = Some(tok);
            }
        }
    }
}

mod grammar {
    use self::Syntax::*;
    use crate::parsing::Parser;
    use crate::Syntax;
    use anyhow::Result;

    /// Parse the root.
    pub(crate) fn root(p: &mut Parser<'_>) -> Result<()> {
        while !p.is_eof() {
            // Consume first number.
            if !p.eat(NUMBER)? {
                p.bump(ERROR)?;
                continue;
            }

            // Consume subsequent operators followed by numbers.
            while !p.is_eof() {
                let tok = p.peek();

                match tok.syntax {
                    PLUS => {
                        p.bump(OPERATOR)?;
                    }
                    MINUS => {
                        p.bump(OPERATOR)?;
                    }
                    _ => {
                        // Simple error recovery where we consume until we find
                        // an operator.
                        let c = p.tree.checkpoint();
                        p.advance_until(&[PLUS, MINUS]);
                        p.tree.close_at(c, ERROR)?;
                        continue;
                    }
                }

                if !p.eat(NUMBER)? {
                    p.bump(ERROR)?;
                    continue;
                }
            }
        }

        Ok(())
    }
}

mod eval {
    use crate::Syntax;
    use syntree::{Span, Tree};
    use thiserror::Error;
    use Syntax::*;

    #[derive(Debug, Error)]
    #[error("{kind}")]
    #[non_exhaustive]
    pub(crate) struct EvalError {
        pub(crate) span: Span,
        pub(crate) kind: EvalErrorKind,
    }

    impl EvalError {
        const fn new(span: Span, kind: EvalErrorKind) -> Self {
            Self { span, kind }
        }
    }

    #[derive(Debug, Error)]
    pub(crate) enum EvalErrorKind {
        #[error("expected {0:?} but was {1:?}")]
        Expected(Syntax, Syntax),

        #[error("expected {0:?}")]
        Missing(Syntax),

        #[error("bad number")]
        BadNumber,

        #[error("{0:?} is not a valid operator")]
        UnexpectedOperator(Syntax),

        #[error("expected an operator")]
        ExpectedOperator,
    }

    macro_rules! expect {
        ($span:expr, $item:expr, $expect:expr) => {
            if let Some(n) = $item {
                if *n.value() != $expect {
                    return Err(EvalError::new(n.span(), Expected($expect, *n.value())));
                }

                n
            } else {
                return Err(EvalError::new($span, Missing($expect)));
            }
        };
    }

    pub(crate) fn eval(tree: &Tree<Syntax>, source: &str) -> Result<i64, EvalError> {
        use EvalErrorKind::*;

        let mut it = tree.children().skip_tokens();
        let full_span = Span::new(0, source.len());

        let first = expect!(full_span, it.next(), NUMBER);

        let mut n = source[first.span().range()]
            .parse::<i64>()
            .map_err(|_| EvalError::new(first.span(), BadNumber))?;

        while let Some(node) = it.next() {
            if *node.value() != OPERATOR {
                return Err(EvalError::new(
                    node.span(),
                    Expected(OPERATOR, *node.value()),
                ));
            }

            let op = node
                .first()
                .ok_or(EvalError::new(node.span(), ExpectedOperator))?;

            let op: fn(i64, i64) -> i64 = match *op.value() {
                PLUS => i64::wrapping_add,
                MINUS => i64::wrapping_sub,
                what => return Err(EvalError::new(node.span(), UnexpectedOperator(what))),
            };

            let rem = Span::new(node.span().end, source.len());

            let arg = expect!(rem, it.next(), NUMBER);

            let arg = source[arg.span().range()]
                .parse::<i64>()
                .map_err(|_| EvalError::new(arg.span(), BadNumber))?;

            n = op(n, arg);
        }

        Ok(n)
    }
}
