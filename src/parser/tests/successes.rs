//! This module contains the tests that should success and checks that the ast is correct.

use std::error::Error;
use std::path::PathBuf;

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

#[test]
fn test_can_parse_multi_item_unordered_list() -> Result<(), Box<dyn Error>> {
    let p = parse_content("- Item 1\n- Item 2");
    assert!(p.is_ok());

    let (_, ast) = p.unwrap();

    let expected_ast = 
        vec![
            Ast::UnorderedList(
                vec![
                    Ast::UnorderedListItem(
                        vec![Ast::Text("Item 1".into())]
                    ),
                    Ast::UnorderedListItem(
                        vec![Ast::Text("Item 2".into())]
                    ),
                ]
            )
        ];

    assert_eq!(expected_ast, ast);

    Ok(())
}
