//! This module contains all the functions needed for parsing.

// This module contains marcos that can't be documented, so we'll allow missing docs here.
#![allow(missing_docs)]
// Allow redundant closure because of nom.
#![allow(clippy::redundant_closure)]

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use nom::branch::alt;
use nom::bytes::complete::{tag, take_till1, take_until};
use nom::character::complete::{char, line_ending, not_line_ending, space0};
use nom::combinator::{map, map_res, opt, rest, verify};
use nom::multi::{fold_many0, many0, many1_count};
use nom::sequence::delimited;
use nom::{IResult, Slice};

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
    let (input, content) = in_between("*", input)?;
    let (_, content) = parse_group(content)?;
    Ok((input, Ast::Bold(content)))
}

fn in_between<'a>(pattern: &str, input: Span<'a>) -> IResult<Span<'a>, Span<'a>> {
    delimited(tag(pattern), take_until(pattern), tag(pattern))(input)
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
    let (input, content) = in_between("/", input)?;
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
    let (input, content) = in_between("$", input)?;
    Ok((input, Ast::InlineMath(content.fragment().to_string())))
}

/// Parses a delimited element.
pub fn parse_delimited(input: Span) -> IResult<Span, Ast> {
    alt((parse_bold, parse_italic, parse_inline_math))(input)
}

fn parse_delimited_unmatch_error(input: Span) -> IResult<Span, Ast> {
    alt((
        map(tag("*"), |x| error(x, ErrorType::UnmatchedStar)),
        map(tag("/"), |x| error(x, ErrorType::UnmatchedSlash)),
        map(tag("$"), |x| error(x, ErrorType::UnmatchedDollar)),
    ))(input)
}

/// Parses a comment.
/// ```
/// # use spandex::parser::ast::Ast;
/// # use spandex::parser::Span;
/// # use spandex::parser::combinators::parse_comment;
/// let input = Span::new("|| comment");
/// let parse = parse_comment(input).unwrap().1;
/// assert_eq!(parse, Ast::Newline);
/// ```
pub fn parse_comment(input: Span) -> IResult<Span, Ast> {
    let (input, _) = tag("||")(input)?;
    let (input, _) = not_line_ending(input)?;
    let (input, _) = opt(line_ending)(input)?;
    Ok((input, Ast::Newline))
}

/// Parses some multiline inline content.
pub fn parse_any(input: Span) -> IResult<Span, Ast> {
    alt((
        map(tag("**"), |x| warning(x, WarningType::ConsecutiveStars)),
        parse_comment,
        parse_delimited,
        parse_delimited_unmatch_error,
        map(tag("|"), |_| Ast::Text(String::from("|"))),
        map(take_till1(should_stop), |x: Span| {
            Ast::Text(ligature(x.fragment()))
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
/// # use spandex::parser::combinators::parse_single_line;
/// let input = Span::new("This is my title");
/// let parsed = parse_single_line(input).unwrap().1;
/// assert_eq!(parsed, vec![Ast::Text(String::from("This is my title"))]);
/// ```
pub fn parse_single_line(input: Span) -> IResult<Span, Vec<Ast>> {
    alt((parse_two_lines_error, parse_group))(input)
}

fn parse_two_lines_error(input: Span) -> IResult<Span, Vec<Ast>> {
    let (input, _) = not_line_ending(input)?;
    let (input, _) = line_ending(input)?;
    let (input, span) = not_line_ending(input)?;
    Ok((input, vec![error(span, ErrorType::MultipleLinesTitle)]))
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
    map(many1_count(char('#')), |nb_hashes| nb_hashes - 1)(input)
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
    let (input, _) = space0(input)?;
    let (input, content) = parse_single_line(input)?;
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

/// Gets a block of content.
/// ```
/// # use spandex::parser::ast::Ast;
/// # use spandex::parser::Span;
/// # use spandex::parser::combinators::get_block;
/// let input = Span::new("First paragraph\n\nSecond paragraph");
/// let (input, block) = get_block(input).unwrap();
/// assert_eq!(block.fragment(), &"First paragraph");
/// let (input, block) = get_block(input).unwrap();
/// assert_eq!(block.fragment(), &"Second paragraph");
/// ```
pub fn get_block(input: Span) -> IResult<Span, Span> {
    let take_until_double_line_ending = |i| {
        alt((
            take_until("\r\n\r\n"),
            take_until("\r\n\n"),
            take_until("\n\n"),
        ))(i)
    };
    let at_least_1_char = verify(rest, |s: &Span| !s.fragment().is_empty());

    let (input, span) = alt((take_until_double_line_ending, at_least_1_char))(input)?;
    let len_after_trimmed = span.fragment().trim_end().len();
    let (input, _) = many0(line_ending)(input)?;
    Ok((input, span.slice(..len_after_trimmed)))
}

/// Parses a block of content.
/// ```
/// # use spandex::parser::ast::Ast;
/// # use spandex::parser::Span;
/// # use spandex::parser::combinators::parse_block_content;
/// let input = Span::new("First paragraph");
/// let (_, block) = parse_block_content(input).unwrap();
/// assert_eq!(block, Ast::Paragraph(vec![Ast::Text(String::from("First paragraph"))]));
/// ```
pub fn parse_block_content(input: Span) -> IResult<Span, Ast> {
    alt((parse_title, parse_paragraph))(input)
}

/// Parses a whole dex file.
pub fn parse_content(input: &str) -> IResult<Span, Vec<Ast>> {
    let parse_block = map_res(get_block, parse_block_content);
    fold_many0(parse_block, Vec::new, |mut content: Vec<_>, (_, block)| {
        content.push(block);
        content
    })(Span::new(input))
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
