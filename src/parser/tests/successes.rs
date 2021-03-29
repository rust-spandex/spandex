//! This module contains the tests that should success and checks that the ast is correct.

use std::error::Error;
use std::path::PathBuf;
use test_case::test_case;

// tempoary for scratching
use nom::branch::alt;
use nom::bytes::complete::{tag, take_till1, take_until, take_while, take};
use nom::character::complete::{char, line_ending, not_line_ending, space0, none_of};
use nom::combinator::{map, map_res, opt, rest, verify};
use nom::multi::{fold_many0, many0, many1, many1_count, many_till,many0_count};
use nom::sequence::{delimited, preceded, terminated};
use nom::IResult;
use crate::parser::{position, Parsed, Span};



use crate::parser::{parse, parse_content, Ast};

#[test]
fn test_title_1() -> Result<(), Box<dyn Error>> {
    let path = "assets/tests/successes/test-title-1.dex";
    let p = parse(path);
    assert!(p.is_ok());

    let ast = p.unwrap().ast;

    let expected_ast = Ast::File(
        PathBuf::from(path),
        vec![Ast::Title {
            level: 0,
            children: vec![Ast::Text("A title".into())],
        }],
    );

    assert_eq!(expected_ast, ast);

    Ok(())
}

#[test]
fn test_title_2() -> Result<(), Box<dyn Error>> {
    let path = "assets/tests/successes/test-title-2.dex";
    let p = parse(path);
    assert!(p.is_ok());

    let ast = p.unwrap().ast;

    let expected_ast = Ast::File(
        PathBuf::from(path),
        vec![Ast::Title {
            level: 1,
            children: vec![Ast::Text("A subtitle".into())],
        }],
    );

    assert_eq!(expected_ast, ast);

    Ok(())
}

#[test]
fn test_titles() -> Result<(), Box<dyn Error>> {
    let path = "assets/tests/successes/test-titles.dex";
    let p = parse(path);
    assert!(p.is_ok());

    let ast = p.unwrap().ast;

    let expected_ast = Ast::File(
        PathBuf::from(path),
        vec![
            Ast::Title {
                level: 0,
                children: vec![Ast::Text("A title".into())],
            },
            Ast::Title {
                level: 1,
                children: vec![Ast::Text("With its subtitle".into())],
            },
        ],
    );

    assert_eq!(expected_ast, ast);

    Ok(())
}

#[test_case("- Item 1\n- Item 2" ; "windows line ending")]
#[test_case("- Item 1\r\n- Item 2" ; "linux line ending")]
fn test_two_item_unordered_list(dex: &str) -> Result<(), Box<dyn Error>> {
    let p = parse_content(dex);
    assert!(p.is_ok());

    let (_, ast) = p.unwrap();

    let expected_ast = vec![
        Ast::UnorderedList(vec![
            Ast::UnorderedListItem{ 
                level: 0, 
                children: vec![text("Item 1")]
            },
            Ast::UnorderedListItem{ 
                level: 0, 
                children: vec![text("Item 2")]
            }
        ])
    ];

    assert_eq!(expected_ast, ast);

    Ok(())
}

#[test]
fn test_unordered_list_items_with_line_breaks() -> Result<(), Box<dyn Error>> {
    let p = parse_content("- item1line1\nitem1line2\n- item2");
    assert!(p.is_ok());

    let (_, ast) = p.unwrap();

    let expected_ast = vec![
        Ast::UnorderedList(vec![
            Ast::UnorderedListItem { 
                level: 0, 
                children: vec![text("item1line1\nitem1line2")] 
            },
            Ast::UnorderedListItem { 
                level: 0, 
                children: vec![text("item2")]
            }
        ])
    ];

    assert_eq!(expected_ast, ast);

    Ok(())
}

#[test]
// get_block trims whitespace from the end of the string
// So we either have to match any '-' at the beginning
// of a line, or stop trimming whitespace, or disallow
// a blank final list item. I have gone with the last
// option for now
fn test_empty_unordered_list_items() -> Result<(), Box<dyn Error>> {
    let p = parse_content("- \n- \n- blah");
    assert!(p.is_ok());

    let (_, ast) = p.unwrap();

    let expected_ast = vec![Ast::UnorderedList(vec![
        Ast::UnorderedListItem{ level:0, children: vec![] },
        Ast::UnorderedListItem{ level:0, children: vec![] },
        Ast::UnorderedListItem{ level:0, children: vec![text("blah")] },
    ])];

    assert_eq!(expected_ast, ast);

    Ok(())
}


pub fn is_space(c: char) -> bool {
    c == ' '
}



pub fn parse_unordered_list_item(input: Span) -> IResult<Span, String> {
    let (input, level ) = terminated(many0_count(char(' ')), tag("- "))(input)?;
    
    let (input, (content, terminator)) = 
            many_till(
                none_of(""),
                delimited(
                    line_ending,
                    take_while(is_space),
                    tag("-")
                )
            )
        (input)?;
    
    //let content = content.iter().map(|s|s.fragment).collect::<Vec<&str>>().join("");
    let content: String = content.into_iter().collect();

    Ok((input, content))
}

#[test_case("- Item 1\n- Item 2" ; "same level")]
#[test_case("- Item 1\n - Item 2" ; "nested")]
#[test_case(" - Item 1\n  - Item 2" ; "double nested")]
#[test_case("- Item 1\r\n- Item 2" ; "linux, same level")]
#[test_case("- Item 1\r\n - Item 2" ; "linux, nested")]
#[test_case(" - Item 1\r\n  - Item 2" ; "linux, double nested")]
fn test_nested_unordered_list(dex: &str) -> Result<(), Box<dyn Error>> {
    let p = parse_unordered_list_item(Span::new(dex.into()));
    assert!(p.is_ok());

    let (_, content) = p.unwrap();

    assert_eq!("Item 1", content);

    Ok(())
}

fn text(some_text: &str) -> Ast {
    Ast::Text(some_text.into())
}
