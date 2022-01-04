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
    syntree::print::print_with_source(&mut o.lock(), &tree, &source)?;
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
    use std::collections::VecDeque;
    use syntree::TreeBuilder;

    #[derive(Debug, Clone, Copy)]
    pub(crate) struct Token {
        pub(crate) len: usize,
        pub(crate) syntax: Syntax,
    }

    struct Lexer<'a> {
        source: &'a str,
        pos: usize,
    }

    impl<'a> Lexer<'a> {
        fn new(source: &'a str) -> Self {
            Self { source, pos: 0 }
        }

        /// Step to the next character.
        fn step(&mut self) {
            if let Some(c) = self.source[self.pos..].chars().next() {
                self.pos += c.len_utf8();
            }
        }

        /// Peek the next character of input.
        fn peek(&self) -> Option<(char, usize)> {
            let c = self.source[self.pos..].chars().next()?;
            Some((c, self.pos))
        }

        /// Consume input until we hit non-numerics.
        fn consume_number(&mut self) {
            while let Some(('0'..='9', _)) = self.peek() {
                self.step();
            }
        }

        /// Consume input until we hit non-numerics.
        fn consume_whitespace(&mut self) {
            while let Some((c, _)) = self.peek() {
                if !c.is_whitespace() {
                    break;
                }

                self.step();
            }
        }

        /// Get the next token.
        fn next(&mut self) -> Option<Token> {
            while let Some((c, start)) = self.peek() {
                let syntax = if c.is_whitespace() {
                    self.consume_whitespace();
                    Syntax::WHITESPACE
                } else {
                    match c {
                        '+' => {
                            self.step();
                            Syntax::PLUS
                        }
                        '-' => {
                            self.step();
                            Syntax::MINUS
                        }
                        '0'..='9' => {
                            self.step();
                            self.consume_number();
                            Syntax::NUMBER
                        }
                        _ => {
                            self.step();
                            Syntax::ERROR
                        }
                    }
                };

                return Some(Token {
                    len: self.pos.checked_sub(start).expect("length underflow"),
                    syntax,
                });
            }

            None
        }
    }

    /// Parser and lexer baked into one.
    pub(crate) struct Parser<'a> {
        lexer: Lexer<'a>,
        pub(crate) tree: TreeBuilder<Syntax>,
        buf: VecDeque<Token>,
    }

    impl<'a> Parser<'a> {
        pub(crate) fn new(source: &'a str) -> Self {
            Self {
                lexer: Lexer::new(source),
                tree: TreeBuilder::new(),
                buf: VecDeque::new(),
            }
        }

        /// Test if the parser is currently at EOF.
        pub(crate) fn is_eof(&mut self) -> bool {
            self.nth(0).syntax == Syntax::EOF
        }

        /// Peek the next token.
        pub(crate) fn peek(&mut self) -> Token {
            self.nth(0)
        }

        /// Try to eat the given sequence of syntax as the given node `what`.
        pub(crate) fn eat(&mut self, what: Syntax, expected: &[Syntax]) -> bool {
            // Ensure we consume leading whitespace before we take the checkpoint.
            self.fill(0);

            let c = self.tree.checkpoint();

            for (n, syntax) in expected.iter().copied().enumerate() {
                if self.nth(n).syntax != syntax {
                    return false;
                }
            }

            for _ in 0..expected.len() {
                let tok = self.nth(0);
                self.tree.token(tok.syntax, tok.len);
                self.step();
            }

            self.tree.insert_node_at(c, what);
            true
        }

        /// Bump the current input as the given syntax.
        pub(crate) fn bump_node(&mut self, what: Syntax) -> Result<()> {
            self.tree.start_node(what);
            let tok = self.nth(0);
            self.step();
            self.tree.token(tok.syntax, tok.len);
            self.tree.end_node()?;
            Ok(())
        }

        pub(crate) fn advance_until(&mut self, what: &[Syntax]) {
            // Consume until we see another Number (or EOF) for some primitive error
            // recovery.
            loop {
                let tok = self.nth(0);

                if tok.syntax == Syntax::EOF || what.iter().any(|s| *s == tok.syntax) {
                    break;
                }

                self.tree.token(tok.syntax, tok.len);
                self.step();
            }
        }

        /// Consume the head token.
        fn step(&mut self) {
            self.fill(0);
            self.buf.pop_front();
        }

        /// Access the token at the nth position.
        fn nth(&mut self, pos: usize) -> Token {
            loop {
                // Fill up buffer.
                self.fill(pos);

                if let Some(tok) = self.buf.get(pos) {
                    return *tok;
                }

                return Token {
                    len: 0,
                    syntax: Syntax::EOF,
                };
            }
        }

        fn fill(&mut self, pos: usize) {
            while self.buf.len() <= pos {
                let tok = match self.lexer.next() {
                    Some(tok) => tok,
                    None => break,
                };

                // Consume whitespace transparently.
                if matches!(tok.syntax, Syntax::WHITESPACE) {
                    self.tree.token(tok.syntax, tok.len);
                    continue;
                }

                self.buf.push_back(tok);
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
            if !p.eat(NUMBER, &[NUMBER]) {
                p.bump_node(ERROR)?;
                continue;
            }

            // Consume subsequent operators followed by numbers.
            while !p.is_eof() {
                let tok = p.peek();

                match tok.syntax {
                    PLUS => {
                        p.bump_node(OPERATOR)?;
                    }
                    MINUS => {
                        p.bump_node(OPERATOR)?;
                    }
                    _ => {
                        // Simple error recovery where we consume until we find
                        // an operator.
                        let c = p.tree.checkpoint();
                        p.advance_until(&[PLUS, MINUS]);
                        p.tree.insert_node_at(c, ERROR);
                        continue;
                    }
                }

                if !p.eat(NUMBER, &[NUMBER]) {
                    p.bump_node(ERROR)?;
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
        fn new(span: Span, kind: EvalErrorKind) -> Self {
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
        ($span:expr, $item:expr, $expect:expr, $pat:pat) => {
            if let Some(n) = $item {
                if !matches!(n.data(), $pat) {
                    return Err(EvalError::new(n.span(), Expected($expect, *n.data())));
                }

                n
            } else {
                return Err(EvalError::new($span, Missing($expect)));
            }
        };
    }

    pub(crate) fn eval(tree: &Tree<Syntax>, source: &str) -> Result<i64, EvalError> {
        use EvalErrorKind::*;

        let mut it = tree.children();
        let tree_span = tree.span();

        let initial = expect!(tree_span, it.next(), NUMBER, NUMBER);
        let span = initial.span();
        let mut n = source[span.range()]
            .parse::<i64>()
            .map_err(|_| EvalError::new(span, BadNumber))?;

        while let Some(node) = it.next() {
            let span = node.span();

            if *node.data() != OPERATOR {
                return Err(EvalError::new(span, Expected(OPERATOR, *node.data())));
            }

            let op: fn(i64, i64) -> i64 = match node.first().map(|n| *n.data()) {
                Some(PLUS) => i64::wrapping_add,
                Some(MINUS) => i64::wrapping_sub,
                Some(what) => return Err(EvalError::new(node.span(), UnexpectedOperator(what))),
                None => return Err(EvalError::new(span, ExpectedOperator)),
            };

            let arg = expect!(tree_span, it.next(), NUMBER, NUMBER);
            let span = arg.span();
            let arg = source[arg.span().range()]
                .parse::<i64>()
                .map_err(|_| EvalError::new(span, BadNumber))?;

            n = op(n, arg);
        }

        Ok(n)
    }
}
