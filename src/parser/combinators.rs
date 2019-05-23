//! This module contains all the functions needed for parsing.

// This module contains marcos that can't be documented, so we'll allow missing docs here.
#![allow(missing_docs)]
// Allow redundant closure because of nom.
#![allow(clippy::redundant_closure)]

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use nom::types::CompleteStr;
use nom::*;

use crate::layout::paragraphs::ligatures::ligature;
use crate::parser::ast::Ast;
use crate::parser::error::{EmptyError, ErrorType, Errors};
use crate::parser::warning::{EmptyWarning, WarningType, Warnings};
use crate::parser::{position, Error, Parsed, Span};

/// Returns true if the character passed as parameter changes the type of parsing we're going to do.
pub fn should_stop(c: char) -> bool {
    c == '*' || c == '/' || c == '$' || c == '|'
}

/// Creates an error.
pub fn error(span: Span, ty: ErrorType) -> Ast {
    Ast::Error(EmptyError {
        position: position(&span),
        ty,
    })
}

/// Creates a warning.
pub fn warning(span: Span, ty: WarningType) -> Ast {
    Ast::Warning(EmptyWarning {
        position: position(&span),
        ty,
    })
}

/// Parses some bold content.
named!(pub parse_bold<Span, Ast>,
    map!(
        map_res!(preceded!(tag!("*"), take_until_and_consume!("*")), parse_group),
        { |(_,x)| Ast::Bold(x) }
    )
);

/// Parses some italic content.
named!(pub parse_italic<Span, Ast>,
    map!(
        map_res!(preceded!(tag!("/"), take_until_and_consume!("/")), parse_group),
        { |(_,x)| Ast::Italic(x) }
    )
);

/// Parses some math inline math.
named!(pub parse_inline_math<Span, Ast>,
    map!(preceded!(tag!("$"), take_until_and_consume!("$")), { |x: Span| Ast::InlineMath(x.fragment.0.into())} )
);

/// Parses a styled element.
named!(pub parse_styled<Span, Ast>,
    alt!(
        parse_bold | parse_italic | parse_inline_math
    )
);

/// Parses a comment.
named!(pub parse_comment<Span, Ast>,
    map!(preceded!(tag!("||"), alt!(take_until_and_consume!("\n") | call!(rest))), { |_|  Ast::Newline })
);

/// Parses some multiline inline content.
named!(pub parse_any<Span, Ast>,
    alt!(
        tag!("**") => { |x| warning(x, WarningType::ConsecutiveStars) }
        | parse_comment
        | parse_styled
        | tag!("*") => { |x| error(x, ErrorType::UnmatchedStar) }
        | tag!("/") => { |x| error(x, ErrorType::UnmatchedSlash) }
        | tag!("$") => { |x| error(x, ErrorType::UnmatchedDollar) }
        | tag!("|") => { |_| { Ast::Text(String::from("|")) } }
        | take_till!(should_stop) => { |x: Span| { Ast::Text(ligature(x.fragment.0)) } }
    )
);

/// Parses a list item.
named!(pub parse_list_item<Span, Ast>,
    map!(
        preceded!(
            tag!("  - "),
            map_res!(alt!(
                terminated!(take_until_and_consume!("\n"), take_until!("  - "))
                | rest
            ), parse_group)
        ),
        |x| Ast::Paragraph(x.1)
    )
);

/// Parses a list.
named!(pub parse_list<Span, Ast>,
    map!(many1!(parse_list_item), |x| Ast::List(x))
);

/// Parses some text content.
named!(pub parse_group<Span, Vec<Ast>>,
    many0!(parse_any)
);

/// Parses a paragraph of text content.
named!(pub parse_paragraph<Span, Ast>,
    map!(many0!(parse_any), Ast::Paragraph)
);

////////////////////////////////////////////////////////////////////////////////
// For titles
////////////////////////////////////////////////////////////////////////////////

/// Parses a paragraph on a single line.
named!(pub parse_line<Span, Vec<Ast>>,
    alt!(
        map!(preceded!(take_until_and_consume!("\n"), take!(0)), |x| { vec![error(x, ErrorType::MultipleLinesTitle)] })
        | many0!(parse_any)
    )
);

/// Parses the hashes from the level of a title.
named!(pub parse_title_level<Span, usize>,
    map!(
        terminated!(preceded!(tag!("#"), take_while!(|x| x == '#')), take_while!(char::is_whitespace)),
        |x| x.fragment.0.len() + 1
    )
);

/// Parses a whole title.
named!(pub parse_title<Span, Ast>,
    do_parse!(
        level: parse_title_level >>
        content: parse_line >> ({
            Ast::Title {
                level: (level - 1) as u8,
                children: content
            }
        })
    )
);

////////////////////////////////////////////////////////////////////////////////
// For main
////////////////////////////////////////////////////////////////////////////////

/// Gets a bloc of content.
named!(pub get_bloc<Span, Span>,
    alt!(
        terminated!(take_until_and_consume!("\n\n"), many0!(tag!("\n")))
        | terminated!(take_until_and_consume!("\n"), eof!())
        | call!(rest)
    )
);

/// Parses a bloc of content.
named!(pub parse_bloc_content<Span, Ast>,
    alt!(
        parse_title
        | parse_list
        | parse_paragraph
    )
);

/// Parses a whole dex file.
named!(pub parse_content<Span, Vec<Ast>>,
    do_parse!(
        content: many1!(map_res!(call!(get_bloc), parse_bloc_content)) >> ({
            content.into_iter().map(|x| x.1).collect()
        })
    )
);

/// Parses a whole dex file from a name.
pub fn parse<P: AsRef<Path>>(path: P) -> Result<Parsed, Error> {
    let path = path.as_ref();
    let mut file = File::open(&path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let elements = match parse_content(Span::new(CompleteStr(&content))) {
        Ok((_, elements)) => elements,
        Err(_) => unreachable!(),
    };

    let ast = Ast::File(PathBuf::from(path), elements);

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
        Err(Error::DexError(Errors {
            path: PathBuf::from(&path),
            content,
            errors,
        }))
    }
}
