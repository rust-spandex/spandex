//! This module contains the tests that should success and checks that the ast is correct.

use std::error::Error;
use std::path::PathBuf;

use crate::parser::{parse, Ast, Span};
use crate::parser::combinators::{parse_unordered_list};

#[test]
fn test_title_1() -> Result<(), Box<dyn Error>> {
    let path = "assets/tests/successes/test-title-1.dex";
    let p = parse(path);

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

#[test]
fn test_two_item_unordered_list() {
    let content = "- Item 1
- Item 2";
    
    let unordered_list = parse_unordered_list(Span::new(content)).unwrap().1;

    let expected = make_simple_ast_unordered_list(&[
        (0, &["Item 1"]),
        (0, &["Item 2"]),
    ]);

    assert_eq!(expected, unordered_list);
}

#[test]
fn test_unordered_list_items_with_line_breaks() {
    let content = "- item1line1
item1line2
- item2";

    let unordered_list = parse_unordered_list(Span::new(content)).unwrap().1;

    let expected = make_simple_ast_unordered_list(&[
        (0, &["item1line1\nitem1line2"]),
        (0, &["item2"]),
    ]);

    assert_eq!(expected, unordered_list);
}

#[test]
// get_block trims whitespace from the end of the string
// So we either have to match any '-' at the beginning
// of a line, or stop trimming whitespace, or disallow
// a blank final list item. I have gone with the last
// option for now
fn test_empty_unordered_list_items() {
    let content = "- 
-
- 
- blah";

    let unordered_list = parse_unordered_list(Span::new(content)).unwrap().1;

    let expected = make_simple_ast_unordered_list(&[
        (0, &["\n-"]),
        (0, &[]),
        (0, &["blah"]),
    ]);

    assert_eq!(expected, unordered_list);
}

#[test]
fn test_nested_unordered_list_same_level() -> Result<(), Box<dyn Error>> {
    test_nested_unordered_list("- Item 1\n- Item 2", 0, 0)
}

#[test]
fn test_nested_unordered_list_windows_same_level() -> Result<(), Box<dyn Error>> {
    test_nested_unordered_list("- Item 1\r\n- Item 2", 0, 0)
}

#[test]
fn test_nested_unordered_list_nested() -> Result<(), Box<dyn Error>> {
    test_nested_unordered_list("- Item 1\n - Item 2", 0, 1)
}

#[test]
fn test_nested_unordered_list_double_nested() -> Result<(), Box<dyn Error>> {
    test_nested_unordered_list(" - Item 1\n  - Item 2", 1, 2)
}

fn test_nested_unordered_list(dex: &str, expected_level_1: u8, expected_level_2: u8) -> Result<(), Box<dyn Error>> {
    let unordered_list = parse_unordered_list(Span::new(dex)).unwrap().1;

    assert_eq!(unordered_list,
        make_simple_ast_unordered_list(&[
            (expected_level_1, &["Item 1"]),
            (expected_level_2, &["Item 2"]),
        ])
    );

    Ok(())
}

#[test]
fn test_nested_unordered_list_no_line_ending() {
    let unordered_list = parse_unordered_list(Span::new(" - Item 1")).unwrap().1;

    assert_eq!(unordered_list,
        make_simple_ast_unordered_list(&[
            (1, &["Item 1"])
        ])
    );
}

/// Helper function to create an unordered list AST hierarchy.
fn make_simple_ast_unordered_list(items: &[(u8, &[&str])]) -> Ast {
    Ast::UnorderedList(
        items
            .iter()
            .map(|&(level, item_txt)| Ast::UnorderedListItem {
                level,
                children: item_txt.iter().map(|&t| text(t)).collect(),
            })
            .collect(),
    )
}

/// Helper function to create a text AST element.
fn text(some_text: &str) -> Ast {
    Ast::Text(some_text.into())
}
