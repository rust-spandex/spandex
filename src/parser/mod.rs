//! This crate contains the parser for spandex.

pub mod ast;
pub mod error;
pub mod parser;
pub mod utils;
pub mod warning;

#[cfg(test)]
mod tests;

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use nom::types::CompleteStr;
use nom_locate::LocatedSpan;

use crate::parser::ast::Ast;
use crate::parser::error::Errors;
use crate::parser::warning::Warnings;

/// This type will allow us to know where we are while we're parsing the content.
pub type Span<'a> = LocatedSpan<CompleteStr<'a>>;

/// A position is a span but without the reference to the complete str.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Position {
    /// The line number of the position.
    pub line: u32,

    /// The column number of the position.
    pub column: usize,

    /// The offset from the beginning of the string.
    pub offset: usize,
}

/// Returns the position of a span.
pub fn position<'a>(span: &Span<'a>) -> Position {
    Position {
        line: span.line,
        column: span.get_utf8_column(),
        offset: span.offset,
    }
}

/// An ast that was successfully parsed.
#[derive(Debug)]
pub struct Parsed {
    /// The parsed ast.
    pub ast: Ast,

    /// The warnings that were produced.
    pub warnings: Warnings,
}

/// Parses a dex file.
pub fn parse<'a, P: AsRef<Path>>(path: P) -> Result<Parsed, Errors> {
    let path = path.as_ref();
    let mut file = File::open(&path).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    let ast = match parser::parse(Span::new(CompleteStr(&content))) {
        Ok((_, ast)) => ast,
        Err(_) => panic!(),
    };

    let errors = ast.errors();
    let warnings = ast.warnings();

    if errors.is_empty() {
        Ok(Parsed {
            ast,
            warnings: Warnings {
                path: PathBuf::from(&path),
                warnings,
                content,
            },
        })
    } else {
        Err(Errors {
            path: PathBuf::from(&path),
            content,
            errors,
        })
    }
}
