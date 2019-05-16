//! This module contains everything related to parsing errors.

use std::error::Error;
use std::fmt;
use std::path::PathBuf;

use colored::*;

use crate::parser::utils::{next_new_line, previous_new_line, replicate};
use crate::parser::Position;

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
            ErrorType::MultipleLinesTitle => "titles must be followed by an empty line",
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

    /// Returns an optional note.
    pub fn note(self) -> Option<&'static str> {
        match self {
            ErrorType::UnmatchedStar => None,
            ErrorType::UnmatchedSlash => None,
            ErrorType::UnmatchedDollar => None,
            ErrorType::MultipleLinesTitle => None,
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

            writeln!(
                fmt,
                "{}{} {}:{}:{}",
                space,
                "-->".bold().blue(),
                self.path.display(),
                line,
                column
            )?;

            writeln!(fmt, "{} {}", space, "|".blue().bold())?;
            writeln!(
                fmt,
                "{} {}",
                &format!("{}|", line_number).blue().bold(),
                &self.content[start..end]
            )?;
            writeln!(
                fmt,
                "{} {}{}{} {}",
                space,
                "|".blue().bold(),
                margin,
                hats.bold().red(),
                error.ty.detail().bold().red()
            )?;
            writeln!(fmt, "{} {}", space, "|".blue().bold())?;
            if let Some(note) = error.ty.note() {
                writeln!(
                    fmt,
                    "{} {} {}{}",
                    space,
                    "=".blue().bold(),
                    "note: ".bold(),
                    note
                )?;
            }
        }

        Ok(())
    }
}

impl Error for Errors {}
