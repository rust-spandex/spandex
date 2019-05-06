//! This crate contains the parser for spandex.

pub mod utils;
pub mod parser;

#[cfg(test)]
mod tests;

use std::fmt;
use std::fs::File;
use std::io::Read;
use std::error::Error;
use std::path::{Path, PathBuf};

use colored::*;

use nom::types::CompleteStr;
use nom_locate::LocatedSpan;

use crate::parser::utils::{next_new_line, previous_new_line, replicate};

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

/// A trait that serves the purpose to add the position method to span.
pub trait ToPosition {

    /// Convers self to a position.
    fn position(&self) -> Position;

}

impl<'a> ToPosition for Span<'a> {
    fn position(&self) -> Position {
        Position {
            line: self.line,
            column: self.get_utf8_column(),
            offset: self.offset,
        }
    }
}

/// The different types errors that can occur while parsing.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ErrorType {
    /// A star for bold content is unmatched.
    UnmatchedStar,

    /// A slash for italic content is unmatched.
    UnmatchedSlash,

    /// A dollar for a inlinemath is unmatched.
    UnmatchedDollar,

    /// A title is on multiple lines.
    MultipleLinesTitle,
}

impl ErrorType {
    /// Returns the title of the error.
    pub fn title(self) -> &'static str {
        match self {
            ErrorType::UnmatchedStar => "unmatched *",
            ErrorType::UnmatchedSlash => "unmactched /",
            ErrorType::UnmatchedDollar => "unmactched $",
            ErrorType::MultipleLinesTitle => "titles must be followed by an empty line"
        }
    }

    /// Returns the detail of the error.
    pub fn detail(self) -> &'static str {
        match self {
            ErrorType::UnmatchedStar => "bold content starts here but never ends",
            ErrorType::UnmatchedSlash => "italic content starts here but never ends",
            ErrorType::UnmatchedDollar => "inline inlinemath starts here but never ends",
            ErrorType::MultipleLinesTitle => "expected empty line here",
        }
    }
}

/// An error that occured during the parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmptyError {
    /// The position of the error.
    pub position: Position,

    /// The type of the error.
    pub ty: ErrorType,
}

/// A struct that can contain many errors.
#[derive(Debug)]
pub struct UnnammedErrors {
    /// The content that produced the errors.
    pub content: String,

    /// The errors that were produced.
    pub errors: Vec<EmptyError>,
}

/// A struct that contains many errors that references a file.
#[derive(Debug)]
pub struct Errors {
    /// The path to the corresponding file.
    pub path: PathBuf,

    /// The content that produced the errors.
    pub content: String,

    /// The errors that were produced.
    pub errors: Vec<EmptyError>,

}

impl fmt::Display for Errors {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {

        for error in &self.errors {

            let start = previous_new_line(&self.content, error.position.offset);
            let end = next_new_line(&self.content, error.position.offset);

            let line = error.position.line;
            let column = error.position.column;

            let line_number = format!("{} ", line);
            let space = replicate(' ', line_number.len() - 1);
            let margin = replicate(' ', column);
            let hats = replicate('^', 1);

            writeln!(fmt, "{}{}", "error: ".bold().red(), error.ty.title().bold())?;

            writeln!(fmt, "{}{} {}:{}:{}",
                     space,
                     "-->".bold().blue(),
                     self.path.display(),
                     line,
                     column)?;

            writeln!(fmt, "{} {}", space, "|".blue().bold())?;
            writeln!(fmt, "{} {}", &format!("{}|", line_number).blue().bold(), &self.content[start .. end])?;
            writeln!(fmt, "{} {}{}{} {}", space, "|".blue().bold(), margin, hats.bold().red(), error.ty.detail().bold().red())?;
            writeln!(fmt, "{} {}", space, "|".blue().bold())?;

        }

        Ok(())
    }
}

impl Error for Errors { }

/// The abstract syntax tree representing the parsed file.
#[derive(Debug, PartialEq, Eq)]
pub enum Ast {
    /// A title.
    Title {
        /// The level of the title.
        level: u8,

        /// The content of the title.
        content: Box<Ast>,
    },

    /// Some bold content.
    Bold(Box<Ast>),

    /// Some italic content.
    Italic(Box<Ast>),

    /// A math inlinemath.
    InlineMath(String),

    /// Some text.
    Text(String),

    /// A paragraph.
    ///
    /// It contains many elements but must be rendered on a single paragraph.
    Paragraph(Vec<Ast>),

    /// A group of content.
    Group(Vec<Ast>),

    /// An empty line.
    Newline,

    /// An error.
    ///
    /// Error will be stored in the abstract syntax tree so we can keep parsing what's parsable and
    /// print many errors instead of crashing immediately.
    Error(EmptyError),
}

/// An ast that was successfully parsed.
#[derive(Debug)]
pub struct Parsed {
    /// The original content of the parsed string.
    pub content: String,

    /// The parsed ast.
    pub ast: Ast,
}

impl Ast {
    /// Returns all the errors contained in the ast.
    pub fn errors(&self) -> Vec<EmptyError> {
        let mut errors = vec![];

        match self {
            Ast::Error(e) => {
                errors.push(e.clone())
            },

            Ast::Group(children) => {
                for child in children {
                    errors.extend(child.errors());
                }
            },

            Ast::Paragraph(children) => {
                for child in children {
                    errors.extend(child.errors());
                }
            },

            Ast::Title { content: ast, .. } => {
                errors.extend(ast.errors());
            },

            Ast::Bold(ast) => {
                errors.extend(ast.errors());
            },

            Ast::Italic(ast) => {
                errors.extend(ast.errors());
            },

            Ast::Text(_) | Ast::Newline | Ast::InlineMath(_) => (),
        }

        errors
    }
}

impl fmt::Display for Ast {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Ast::Title { level, content } => {
                for _ in 0 .. *level {
                    write!(fmt, "{}", "#".bold())?;
                }
                writeln!(fmt, "{}", &format!(" {}", content).bold())?;
            },

            Ast::Bold(subast) => write!(fmt, "{}", &format!("{}", subast).red())?,
            Ast::Italic(subast) => write!(fmt, "{}", &format!("{}", subast).blue())?,
            Ast::InlineMath(content) => write!(fmt, "${}$", content)?,
            Ast::Text(content) => write!(fmt, "{}", content)?,
            Ast::Group(children) => {
                for child in children {
                    write!(fmt, "{}", child)?;
                }
            },
            Ast::Paragraph(children) => {
                for child in children {
                    write!(fmt, "{}", child)?;
                }
            },

            Ast::Error(_) => writeln!(fmt, "?")?,
            Ast::Newline => writeln!(fmt)?,
        }
        Ok(())
    }
}

/// Parses a dex file.
pub fn parse<'a, P: AsRef<Path>>(path: P) -> Result<Parsed, Errors> {

    let path = path.as_ref();
    let mut file = File::open(&path).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    let ast = match parser::parse(Span::new(CompleteStr(&content))) {
        Ok((_, ast)) => {
            ast
        },
        Err(_) => panic!(),
    };

    let errors = ast.errors();

    if errors.is_empty() {
        Ok(Parsed {
            content,
            ast,
        })
    } else {
        Err(Errors {
            path: PathBuf::from(&path),
            content,
            errors,
        })
    }

}
