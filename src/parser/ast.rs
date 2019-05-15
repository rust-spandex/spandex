//! This module contains everything related to the ast.

use std::fmt;

use colored::*;

use crate::parser::error::EmptyError;
use crate::parser::warning::EmptyWarning;

/// The abstract syntax tree representing the parsed file.
#[derive(PartialEq, Eq, Clone)]
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

    /// A warning.
    ///
    /// Warning will be stored in the Ast and we will print them in the main function.
    Warning(EmptyWarning),
}

impl Ast {
    /// Returns all the errors contained in the ast.
    pub fn errors(&self) -> Vec<EmptyError> {
        let mut errors = vec![];

        match self {
            Ast::Error(e) => {
                errors.push(e.clone())
            },

            Ast::Warning(_) => {

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

    /// Returns all the warnings contained in the ast.
    pub fn warnings(&self) -> Vec<EmptyWarning> {
        let mut warnings = vec![];

        match self {
            Ast::Warning(e) => {
                warnings.push(e.clone())
            },

            Ast::Error(_) => {

            },

            Ast::Group(children) => {
                for child in children {
                    warnings.extend(child.warnings());
                }
            },

            Ast::Paragraph(children) => {
                for child in children {
                    warnings.extend(child.warnings());
                }
            },

            Ast::Title { content: ast, .. } => {
                warnings.extend(ast.warnings());
            },

            Ast::Bold(ast) => {
                warnings.extend(ast.warnings());
            },

            Ast::Italic(ast) => {
                warnings.extend(ast.warnings());
            },

            Ast::Text(_) | Ast::Newline | Ast::InlineMath(_) => (),
        }

        warnings
    }

    /// Pretty prints the ast.
    pub fn print_debug(&self, fmt: &mut fmt::Formatter, indent: &str) -> fmt::Result {
        match self {
            Ast::Error(e) => write!(fmt, "{}Error({:?})", indent, e)?,
            Ast::Warning(w) => write!(fmt, "{}Warning({:?})", indent, w)?,
            Ast::Text(text) => write!(fmt, "{}Text({:?})", indent, text)?,
            Ast::Newline => write!(fmt, "{}NewLine", indent)?,
            Ast::InlineMath(math) => write!(fmt, "{}Math({:?})", indent, math)?,

            Ast::Group(children) => {
                write!(fmt, "{}Group:", indent)?;
                let new_indent = format!("{}  ", indent);
                for child in children {
                    child.print_debug(fmt, &new_indent)?;
                }
            },

            Ast::Paragraph(children) => {
                write!(fmt, "{}Group:", indent)?;
                let new_indent = format!("{}  ", indent);
                for child in children {
                    child.print_debug(fmt, &new_indent)?;
                }
            },

            Ast::Title { content, level } => {
                write!(fmt, "{}Title(level={})", indent, level)?;
                content.print_debug(fmt, &format!("{}  ", indent))?;
            },

            Ast::Bold(ast) => {
                write!(fmt, "{}Bold", indent)?;
                ast.print_debug(fmt, &format!("{}  ", indent))?;
            },

            Ast::Italic(ast) => {
                write!(fmt, "{}Italic", indent)?;
                ast.print_debug(fmt, &format!("{}  ", indent))?;
            },

        }

        Ok(())
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
            Ast::Warning(_) => (),
        }
        Ok(())
    }
}

impl fmt::Debug for Ast {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.print_debug(fmt, "")
    }
}
