//! This module contains the tests that should success and checks that the ast is correct.

use std::error::Error;
use std::path::PathBuf;
use test_case::test_case;

use crate::parser::{parse, parse_content, Ast};

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

#[test]
fn test_two_item_unordered_list() {
    let content = "
- Item 1
- Item 2";
    // Remove the first end of line with &[1..] slice
    let (_, ast) = parse_content(&content[1..]).unwrap();
    let expected_ast = vec![make_simple_ast_unordered_list(&[
        (0, &["Item 1"]),
        (0, &["Item 2"]),
    ])];
    assert_eq!(expected_ast, ast);
}

#[test]
fn test_unordered_list_items_with_line_breaks() {
    let content = "
- item1line1
item1line2
- item2";
    // Remove the first end of line with &[1..] slice
    let (_, ast) = parse_content(&content[1..]).unwrap();
    let expected_ast = vec![make_simple_ast_unordered_list(&[
        (0, &["item1line1\nitem1line2"]),
        (0, &["item2"]),
    ])];
    assert_eq!(expected_ast, ast);
}

#[test]
// get_block trims whitespace from the end of the string
// So we either have to match any '-' at the beginning
// of a line, or stop trimming whitespace, or disallow
// a blank final list item. I have gone with the last
// option for now
fn test_empty_unordered_list_items() {
    let content = "
- 
-
- 
- blah";
    // Remove the first end of line with &[1..] slice
    let (_, ast) = parse_content(&content[1..]).unwrap();
    let expected_ast = vec![make_simple_ast_unordered_list(&[
        (0, &["\n-"]),
        (0, &[]),
        (0, &["blah"]),
    ])];
    assert_eq!(expected_ast, ast);
}

#[test_case("- Item 1\n- Item 2", 0 ; "same level")]
#[test_case("- Item 1\n - Item 2", 0 ; "nested")]
#[test_case(" - Item 1\n  - Item 2", 1 ; "double nested")]
#[test_case("- Item 1\r\n- Item 2", 0 ; "windows, same level")]
#[test_case("- Item 1\r\n - Item 2", 0 ; "windows, nested")]
#[test_case(" - Item 1\r\n  - Item 2", 1 ; "windows, double nested")]
#[test_case(" - Item 1", 1 ; "No line ending")]
fn test_nested_unordered_list(dex: &str, expected_level: u8) -> Result<(), Box<dyn Error>> {
    let p = parse_content(dex);

    let (_, content) = p.unwrap();

    // This seems very clunky, could just export parse_unordered_list_item instead
    assert_eq!(1, content.len());
    match &content[0] {
        Ast::UnorderedList(items) => {
            let first_unordered_list_item = &items[0];

            match first_unordered_list_item {
                Ast::UnorderedListItem { level, children } => {
                    assert_eq!(expected_level, *level);
                    assert_eq!(vec![text("Item 1")], *children);
                }

                _ => {
                    assert!(false);
                }
            }
        }

        _ => {
            assert!(false);
        }
    }

    Ok(())
}

fn text(some_text: &str) -> Ast {
    Ast::Text(some_text.into())
}
