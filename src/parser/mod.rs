//! This crate contains the parser for spandex.

pub mod ast;
pub mod combinators;
pub mod error;
pub mod utils;
pub mod warning;

#[cfg(test)]
mod tests;

use nom_locate::LocatedSpan;

use crate::parser::ast::Ast;
use crate::parser::warning::Warnings;
use crate::Error;

/// This type will allow us to know where we are while we're parsing the content.
pub type Span<'a> = LocatedSpan<&'a str>;

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
pub fn position(span: &Span) -> Position {
    Position {
        line: span.location_line(),
        column: span.get_utf8_column(),
        offset: span.location_offset(),
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

pub use combinators::parse;
