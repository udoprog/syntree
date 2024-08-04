/// A simple calculator only capable of addition and subtraction.
mod eval;
mod grammar;
mod lexer;
mod parsing;

use std::io::Write;

use anyhow::Result;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFile;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
enum Syntax {
    Number,
    Plus,
    Minus,
    Div,
    Mul,
    Pow,
    Whitespace,
    Operator,
    OpenParen,
    CloseParen,

    // An operation.
    Operation,
    // Precedence group.
    Group,
    // Enf of file.
    Eof,
    Error,
}

fn main() -> Result<()> {
    let source = std::env::args().skip(1).collect::<String>();

    let mut p = parsing::Parser::new(&source);
    grammar::root(&mut p)?;

    let tree = p.tree.build()?;

    let mut o = StandardStream::stderr(ColorChoice::Always);
    let config = codespan_reporting::term::Config::default();

    for result in eval::eval(&tree, &source) {
        match result {
            Ok(output) => {
                writeln!(o, "Output = {output}")?;
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
    }

    writeln!(o, "# Tree:")?;
    syntree::print::print_with_source(o.lock(), &tree, &source)?;
    Ok(())
}
