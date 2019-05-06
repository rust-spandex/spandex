//! This module contains all the functions needed for parsing.

// This module contains marcos that can't be documented, so we'll allow missing docs here.
#![allow(missing_docs)]

use nom::rest;

use crate::parser::{Span, Ast, ToPosition, EmptyError, ErrorType};

/// Returns true if the character passed as parameter changes the type of parsing we're going to do.
pub fn should_stop(c: char) -> bool {
    c == '*' || c == '/' || c == '$'
}

/// Creates an error.
pub fn error(span: Span, ty: ErrorType) -> Ast {
    Ast::Error(EmptyError {
        position: span.position(),
        ty,
    })
}

/// Parses some bold content.
named!(pub parse_bold<Span, Ast>,
    map!(
        map_res!(preceded!(tag!("*"), take_until_and_consume!("*")), parse_paragraph),
        { |(_,x)| Ast::Bold(Box::new(x)) }
    )
);

/// Parses some italic content.
named!(pub parse_italic<Span, Ast>,
    map!(
        map_res!(preceded!(tag!("/"), take_until_and_consume!("/")), parse_paragraph),
        { |(_,x)| Ast::Italic(Box::new(x)) }
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

/// Parses some multiline inline content.
named!(pub parse_any<Span, Ast>,
    alt!(
        parse_styled
        | map!(tag!("*"), |x| error(x, ErrorType::UnmatchedStar))
        | map!(tag!("/"), |x| error(x, ErrorType::UnmatchedSlash))
        | map!(tag!("$"), |x| error(x, ErrorType::UnmatchedDollar))
        | take_till!(should_stop) => { |x: Span| { Ast::Text(x.to_string()) } }
    )
);

/// Parses a paragraph of text content.
named!(pub parse_paragraph<Span, Ast>,
    map!(many0!(parse_any), |x| Ast::Group(x))
);

////////////////////////////////////////////////////////////////////////////////
// For titles
////////////////////////////////////////////////////////////////////////////////

/// Parses a paragraph on a single line.
named!(pub parse_line<Span, Ast>,
    alt!(
        map!(preceded!(take_until_and_consume!("\n"), take!(0)), |x| { error(x, ErrorType::MultipleLinesTitle) })
        | map!(many0!(parse_any), |x| Ast::Group(x))
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
                level: level as u8,
                content: Box::new(content)
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
        parse_title | parse_paragraph | map!(call!(rest), |_| Ast::Text("toto".into()))
    )
);

/// Parses a whole dex file.
named!(pub parse<Span, Ast>,
    do_parse!(
        title: many1!(map_res!(call!(get_bloc), parse_bloc_content)) >> ({
            Ast::Group(title.into_iter().map(|x| x.1).collect())
        })
    )
);

