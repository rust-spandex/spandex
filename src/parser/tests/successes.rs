//! This module contains the tests that should success and checks that the ast is correct.

use crate::parser::{parse, Ast};
use std::error::Error;

#[test]
fn test_title_1() -> Result<(), Box<Error>> {
    let p = parse("assets/tests/successes/test-title-1.dex");
    assert!(p.is_ok());

    let ast = p.unwrap().ast;

    let expected_ast = Ast::Group(vec![Ast::Title {
        level: 0,
        content: Box::new(Ast::Group(vec![Ast::Text("A title".into())])),
    }]);

    assert_eq!(expected_ast, ast);

    Ok(())
}

#[test]
fn test_title_2() -> Result<(), Box<Error>> {
    let p = parse("assets/tests/successes/test-title-2.dex");
    assert!(p.is_ok());

    let ast = p.unwrap().ast;

    let expected_ast = Ast::Group(vec![Ast::Title {
        level: 1,
        content: Box::new(Ast::Group(vec![Ast::Text("A subtitle".into())])),
    }]);

    assert_eq!(expected_ast, ast);

    Ok(())
}

#[test]
fn test_titles() -> Result<(), Box<Error>> {
    let p = parse("assets/tests/successes/test-titles.dex");
    assert!(p.is_ok());

    let ast = p.unwrap().ast;

    let expected_ast = Ast::Group(vec![
        Ast::Title {
            level: 0,
            content: Box::new(Ast::Group(vec![Ast::Text("A title".into())])),
        },
        Ast::Title {
            level: 1,
            content: Box::new(Ast::Group(vec![Ast::Text("With its subtitle".into())])),
        },
    ]);

    assert_eq!(expected_ast, ast);

    Ok(())
}
