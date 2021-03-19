//! This module contains everything related to the ast.

use std::fmt;
use std::path::PathBuf;

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
        children: Vec<Ast>,
    },

    /// Some bold content.
    Bold(Vec<Ast>),

    /// Some italic content.
    Italic(Vec<Ast>),

    /// A math inlinemath.
    InlineMath(String),

    /// Some text.
    Text(String),

    /// A paragraph.
    ///
    /// It contains many elements but must be rendered on a single paragraph.
    Paragraph(Vec<Ast>),

    /// Content stored in a specific file.
    File(PathBuf, Vec<Ast>),

    UnorderedList(Vec<Ast>), // must be UnorderedListItem
    
    UnorderedListItem(Vec<Ast>),

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
    /// Returns the children of the ast, if any.
    pub fn children(&self) -> Option<&Vec<Ast>> {
        match self {
            Ast::File(_, children)
            | Ast::Paragraph(children)
            | Ast::Title { children, .. }
            | Ast::Bold(children)
            | Ast::Italic(children) 
            | Ast::UnorderedList(children)
            | Ast::UnorderedListItem(children) => Some(children),
            _ => None,
        }
    }

    /// Returns all the errors contained in the ast.
    pub fn errors(&self) -> Vec<EmptyError> {
        let mut errors = vec![];

        if let Ast::Error(e) = self {
            errors.push(e.clone());
        }

        if let Some(children) = self.children() {
            for child in children {
                errors.extend(child.errors());
            }
        }

        errors
    }

    /// Returns all the errors contained in the ast.
    pub fn warnings(&self) -> Vec<EmptyWarning> {
        let mut warnings = vec![];

        if let Ast::Warning(e) = self {
            warnings.push(e.clone());
        }

        if let Some(children) = self.children() {
            for child in children {
                warnings.extend(child.warnings());
            }
        }

        warnings
    }

    /// Pretty prints the ast.
    pub fn print_debug(
        &self,
        fmt: &mut fmt::Formatter,
        indent: &str,
        last_child: bool,
    ) -> fmt::Result {
        let delimiter1 = if indent.is_empty() {
            "─"
        } else if last_child {
            "└"
        } else {
            "├"
        };

        let delimiter2 = match self {
            Ast::Error(_) | Ast::Warning(_) | Ast::Text(_) | Ast::Newline | Ast::InlineMath(_) => {
                "──"
            }
            _ => "─┬",
        };

        let new_indent = format!("{}{}{} ", indent, delimiter1, delimiter2);

        let indent = if last_child {
            format!("{}  ", indent)
        } else {
            format!("{}│ ", indent)
        };

        match self {
            Ast::Error(e) => writeln!(fmt, "{}{}", new_indent, &format!("Error({:?})", e).red())?,
            Ast::Warning(e) => writeln!(
                fmt,
                "{}{}",
                new_indent,
                &format!("Warning({:?})", e).yellow()
            )?,
            Ast::Text(t) => writeln!(
                fmt,
                "{}{}{}{}",
                new_indent,
                "Text(".green(),
                &format!("{:?}", t).dimmed(),
                ")".green()
            )?,
            Ast::Newline => writeln!(fmt, "{}NewLine", new_indent)?,
            Ast::InlineMath(math) => writeln!(fmt, "{}Math({:?})", new_indent, math)?,
            Ast::File(path, _) => writeln!(
                fmt,
                "{}{}",
                new_indent,
                &format!("File(\"{}\")", path.display()).blue().bold()
            )?,
            Ast::Paragraph(_) => writeln!(fmt, "{}{}", new_indent, "Paragraph".blue().bold())?,

            Ast::Title { level, .. } => writeln!(
                fmt,
                "{}{}",
                new_indent,
                &format!("Title(level={})", level).magenta().bold()
            )?,

            Ast::Bold(_) => writeln!(fmt, "{}{}", new_indent, "Bold".cyan().bold())?,

            Ast::Italic(_) => writeln!(fmt, "{}{}", new_indent, "Italic".cyan().bold())?,

            Ast::UnorderedList(_) => writeln!(fmt, "{}{}", new_indent, "UnorderedList".blue().bold())?,
         
            Ast::UnorderedListItem(_) => writeln!(fmt, "{}{}", new_indent, "UnorderedListItem".blue().bold())?,
        }

        if let Some(children) = self.children() {
            let len = children.len();
            for (index, child) in children.iter().enumerate() {
                child.print_debug(fmt, &indent, index == len - 1)?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for Ast {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Ast::Title { level, .. } => {
                for _ in 0..*level {
                    write!(fmt, "{}", "#".bold())?;
                }
            }

            Ast::InlineMath(content) => write!(fmt, "${}$", content)?,
            Ast::Text(content) => write!(fmt, "{}", content)?,
            _ => (),
        }

        if let Some(children) = self.children() {
            for child in children {
                write!(fmt, "{}", child)?;
            }
        }

        Ok(())
    }
}

impl fmt::Debug for Ast {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.print_debug(fmt, "", true)
    }
}
