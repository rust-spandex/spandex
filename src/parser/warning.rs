//! This module contains everything related to parse warnings.

use std::fmt;
use std::path::PathBuf;

use colored::*;

use crate::parser::utils::{next_new_line, previous_new_line, replicate};
use crate::parser::Position;

/// The different types of warning that can occur.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WarningType {
    /// Two consecutive stars only seperated by whitespaces.
    ConsecutiveStars,
}

impl WarningType {
    /// Returns the title of the warning.
    pub fn title(self) -> &'static str {
        match self {
            WarningType::ConsecutiveStars => "empty bold section",
        }
    }

    /// Returns the defail of the warning.
    pub fn detail(self) -> &'static str {
        match self {
            WarningType::ConsecutiveStars => "this will be ignored",
        }
    }

    /// Returns a potential note.
    pub fn note(self) -> Option<&'static str> {
        match self {
            WarningType::ConsecutiveStars => {
                Some("to use bold, you should use single stars, e.g. '*this is bold*'")
            }
        }
    }
}

/// An warning that occured during the parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmptyWarning {
    /// The position of the warning.
    pub position: Position,

    /// The type of the warning.
    pub ty: WarningType,
}

/// A struct that contains many warnings that references a file.
#[derive(Debug)]
pub struct Warnings {
    /// The path to the corresponding file.
    pub path: PathBuf,

    /// The content that produced the warnings.
    pub content: String,

    /// The warnings produced.
    pub warnings: Vec<EmptyWarning>,
}

impl fmt::Display for Warnings {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for warning in &self.warnings {
            let start = previous_new_line(&self.content, warning.position.offset);
            let end = next_new_line(&self.content, warning.position.offset);

            let line = warning.position.line;
            let column = warning.position.column;

            let line_number = format!("{} ", line);
            let space = replicate(' ', line_number.len() - 1);
            let margin = replicate(' ', column);
            let hats = replicate('^', 1);

            writeln!(
                fmt,
                "{}{}",
                "warning: ".bold().yellow(),
                warning.ty.title().bold()
            )?;

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
                hats.bold().yellow(),
                warning.ty.detail().bold().yellow()
            )?;
            writeln!(fmt, "{} {}", space, "|".blue().bold())?;

            if let Some(note) = warning.ty.note() {
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
