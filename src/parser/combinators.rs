//! This module contains all the functions needed for parsing.

// This module contains marcos that can't be documented, so we'll allow missing docs here.
#![allow(missing_docs)]
// Allow redundant closure because of nom.
#![allow(clippy::redundant_closure)]

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use nom::branch::*;
use nom::bytes::complete::*;
use nom::combinator::*;
use nom::multi::*;
use nom::sequence::*;
use nom::IResult;

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
/// ```
/// # use spandex::parser::ast::Ast;
/// # use spandex::parser::Span;
/// # use spandex::parser::combinators::parse_bold;
/// let input = Span::new("*Hello*");
/// let parse = parse_bold(input).unwrap().1;
/// assert_eq!(parse, Ast::Bold(vec![Ast::Text(String::from("Hello"))]));
/// ```
pub fn parse_bold(input: Span) -> IResult<Span, Ast> {
    let (input, _) = tag("*")(input)?;
    let (input, content) = take_until("*")(input)?;
    let (input, _) = tag("*")(input)?;
    let (_, content) = parse_group(content)?;
    Ok((input, Ast::Bold(content)))
}

/// Parses some italic content.
/// ```
/// # use spandex::parser::ast::Ast;
/// # use spandex::parser::Span;
/// # use spandex::parser::combinators::parse_italic;
/// let input = Span::new("/Hello/");
/// let parse = parse_italic(input).unwrap().1;
/// assert_eq!(parse, Ast::Italic(vec![Ast::Text(String::from("Hello"))]));
/// ```
pub fn parse_italic(input: Span) -> IResult<Span, Ast> {
    let (input, _) = tag("/")(input)?;
    let (input, content) = take_until("/")(input)?;
    let (input, _) = tag("/")(input)?;
    let (_, content) = parse_group(content)?;
    Ok((input, Ast::Italic(content)))
}

/// Parses some math inline math.
/// ```
/// # use spandex::parser::ast::Ast;
/// # use spandex::parser::Span;
/// # use spandex::parser::combinators::parse_inline_math;
/// let input = Span::new("$x = 9$");
/// let parse = parse_inline_math(input).unwrap().1;
/// assert_eq!(parse, Ast::InlineMath(String::from("x = 9")));
/// ```
pub fn parse_inline_math(input: Span) -> IResult<Span, Ast> {
    let (input, _) = tag("$")(input)?;
    let (input, content) = take_until("$")(input)?;
    let (input, _) = tag("$")(input)?;
    Ok((input, Ast::InlineMath(content.fragment.into())))
}

/// Parses a styled element.
pub fn parse_styled(input: Span) -> IResult<Span, Ast> {
    alt((parse_bold, parse_italic, parse_inline_math))(input)
}

/// Parses a comment.
pub fn parse_comment(input: Span) -> IResult<Span, Ast> {
    map(
        preceded(
            tag("||"),
            alt((terminated(take_until("\n"), tag("\n")), rest)),
        ),
        { |_| Ast::Newline },
    )(input)
}

/// Parses some multiline inline content.
pub fn parse_any(input: Span) -> IResult<Span, Ast> {
    alt((
        map(tag("**"), |x| warning(x, WarningType::ConsecutiveStars)),
        parse_comment,
        parse_styled,
        map(tag("*"), |x| error(x, ErrorType::UnmatchedStar)),
        map(tag("/"), |x| error(x, ErrorType::UnmatchedSlash)),
        map(tag("$"), |x| error(x, ErrorType::UnmatchedDollar)),
        map(tag("|"), |_| Ast::Text(String::from("|"))),
        map(take_till1(should_stop), |x: Span| {
            Ast::Text(ligature(x.fragment))
        }),
    ))(input)
}

/// Parses some text content.
/// ```
/// # use spandex::parser::ast::Ast;
/// # use spandex::parser::Span;
/// # use spandex::parser::combinators::parse_group;
/// let input = Span::new("*Hello* to /you/");
/// let parsed = parse_group(input).unwrap().1;
/// assert_eq!(parsed, vec![
///     Ast::Bold(vec![Ast::Text(String::from("Hello"))]),
///     Ast::Text(String::from(" to ")),
///     Ast::Italic(vec![Ast::Text(String::from("you"))]),
/// ]);
/// ```
pub fn parse_group(input: Span) -> IResult<Span, Vec<Ast>> {
    many0(parse_any)(input)
}

/// Parses a paragraph of text content.
/// ```
/// # use spandex::parser::ast::Ast;
/// # use spandex::parser::Span;
/// # use spandex::parser::combinators::parse_paragraph;
/// let input = Span::new("*Hello* to /you/");
/// let parsed = parse_paragraph(input).unwrap().1;
/// assert_eq!(parsed, Ast::Paragraph(vec![
///     Ast::Bold(vec![Ast::Text(String::from("Hello"))]),
///     Ast::Text(String::from(" to ")),
///     Ast::Italic(vec![Ast::Text(String::from("you"))]),
/// ]));
/// ```
pub fn parse_paragraph(input: Span) -> IResult<Span, Ast> {
    map(parse_group, Ast::Paragraph)(input)
}

////////////////////////////////////////////////////////////////////////////////
// For titles
////////////////////////////////////////////////////////////////////////////////

/// Parses a title on a single line.
/// ```
/// # use spandex::parser::ast::Ast;
/// # use spandex::parser::Span;
/// # use spandex::parser::combinators::parse_line;
/// let input = Span::new("This is my title");
/// let parsed = parse_line(input).unwrap().1;
/// assert_eq!(parsed, vec![Ast::Text(String::from("This is my title"))]);
/// ```
pub fn parse_line(input: Span) -> IResult<Span, Vec<Ast>> {
    alt((
        map(
            preceded(preceded(take_until("\n"), tag("\n")), tag("")),
            |x: Span| vec![error(x, ErrorType::MultipleLinesTitle)],
        ),
        many0(parse_any),
    ))(input)
}

/// Parses the hashes from the level of a title.
/// ```
/// # use spandex::parser::ast::Ast;
/// # use spandex::parser::Span;
/// # use spandex::parser::combinators::parse_title_level;
/// let input = Span::new("# This is my title");
/// let level = parse_title_level(input).unwrap().1;
/// assert_eq!(level, 0);
/// let input = Span::new("### This is my subtitle");
/// let level = parse_title_level(input).unwrap().1;
/// assert_eq!(level, 2);
/// ```
pub fn parse_title_level(input: Span) -> IResult<Span, usize> {
    map(
        terminated(
            preceded(tag("#"), take_while(|x| x == '#')),
            take_while(char::is_whitespace),
        ),
        |x: Span| x.fragment.len(),
    )(input)
}

/// Parses a whole title.
/// ```
/// # use spandex::parser::ast::Ast;
/// # use spandex::parser::Span;
/// # use spandex::parser::combinators::parse_title;
/// let input = Span::new("# This is my title");
/// let title = parse_title(input).unwrap().1;
/// assert_eq!(title, Ast::Title { level: 0, children: vec![
///     Ast::Text(String::from("This is my title"))]
/// });
/// ```
pub fn parse_title(input: Span) -> IResult<Span, Ast> {
    let (input, level) = parse_title_level(input)?;
    let (input, _) = take_while(char::is_whitespace)(input)?;
    let content = parse_line(input)?.1;
    Ok((
        input,
        Ast::Title {
            level: level as u8,
            children: content,
        },
    ))
}

////////////////////////////////////////////////////////////////////////////////
// For main
////////////////////////////////////////////////////////////////////////////////

/// Gets a bloc of content.
/// ```
/// # use spandex::parser::ast::Ast;
/// # use spandex::parser::Span;
/// # use spandex::parser::combinators::get_bloc;
/// let input = Span::new("First paragraph\n\nSecond paragraph");
/// let (input, bloc) = get_bloc(input).unwrap();
/// assert_eq!(bloc.fragment, "First paragraph");
/// let (input, bloc) = get_bloc(input).unwrap();
/// assert_eq!(bloc.fragment, "Second paragraph");
/// ```
pub fn get_bloc(input: Span) -> IResult<Span, Span> {
    alt((terminated(take_until("\n\n"), many0(tag("\n"))), rest))(input)
}

/// Parses a bloc of content.
/// ```
/// # use spandex::parser::ast::Ast;
/// # use spandex::parser::Span;
/// # use spandex::parser::combinators::parse_bloc_content;
/// let input = Span::new("First paragraph");
/// let (_, bloc) = parse_bloc_content(input).unwrap();
/// assert_eq!(bloc, Ast::Paragraph(vec![Ast::Text(String::from("First paragraph"))]));
/// ```
pub fn parse_bloc_content(input: Span) -> IResult<Span, Ast> {
    alt((parse_title, parse_paragraph))(input)
}

/// Parses a whole dex file.
pub fn parse_content(input: &str) -> IResult<Span, Vec<Ast>> {
    let mut input = Span::new(input.trim_end());
    let mut content = vec![];

    loop {
        let (new_input, bloc) = get_bloc(input)?;
        input = new_input;
        let parsed = parse_bloc_content(bloc)?;
        content.push(parsed.1);

        if input.fragment.is_empty() {
            break;
        }
    }

    Ok((input, content))
}

/// Parses a whole dex file from a name.
pub fn parse<P: AsRef<Path>>(path: P) -> Result<Parsed, Error> {
    let path = path.as_ref();
    let mut file = File::open(&path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let elements = match parse_content(&content) {
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
