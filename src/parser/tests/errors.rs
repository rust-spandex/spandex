//! This module contains the tests that should fail and checks that the error messages are correct.

use std::error::Error;
use crate::parser::{parse, ErrorType};

#[test]
fn test_unmatched_star() -> Result<(), Box<Error>> {
    let p = parse("assets/tests/errors/test-unmatched-star.dex");
    assert!(p.is_err());

    let p = p.err().unwrap();
    assert_eq!(p.errors.len(), 1);

    let p = &p.errors[0];
    assert_eq!(p.ty, ErrorType::UnmatchedStar);
    assert_eq!(p.position.line, 1);
    assert_eq!(p.position.column, 10);
    assert_eq!(p.position.offset, 9);
    Ok(())
}

#[test]
fn test_unmatched_slash() -> Result<(), Box<Error>> {
    let p = parse("assets/tests/errors/test-unmatched-slash.dex");
    assert!(p.is_err());

    let p = p.err().unwrap();
    assert_eq!(p.errors.len(), 1);

    let p = &p.errors[0];
    assert_eq!(p.ty, ErrorType::UnmatchedSlash);
    assert_eq!(p.position.line, 1);
    assert_eq!(p.position.column, 7);
    assert_eq!(p.position.offset, 6);
    Ok(())
}

#[test]
fn test_unmatched_dollar() -> Result<(), Box<Error>> {
    let p = parse("assets/tests/errors/test-unmatched-dollar.dex");
    assert!(p.is_err());

    let p = p.err().unwrap();
    assert_eq!(p.errors.len(), 1);

    let p = &p.errors[0];
    assert_eq!(p.ty, ErrorType::UnmatchedDollar);
    assert_eq!(p.position.line, 1);
    assert_eq!(p.position.column, 13);
    assert_eq!(p.position.offset, 12);
    Ok(())
}

#[test]
fn test_mixed_star_slash() -> Result<(), Box<Error>> {
    let p = parse("assets/tests/errors/test-mixed-star-slash.dex");
    assert!(p.is_err());

    let p = p.err().unwrap();
    assert_eq!(p.errors.len(), 1);

    let p = &p.errors[0];
    assert_eq!(p.ty, ErrorType::UnmatchedStar);
    assert_eq!(p.position.line, 1);
    assert_eq!(p.position.column, 21);
    assert_eq!(p.position.offset, 20);

    Ok(())
}

#[test]
fn test_title_no_new_line() -> Result<(), Box<Error>> {
    let p = parse("assets/tests/errors/test-title-no-new-line.dex");
    assert!(p.is_err());

    let p = p.err().unwrap();
    assert_eq!(p.errors.len(), 1);

    let p = &p.errors[0];

    assert_eq!(p.ty, ErrorType::MultipleLinesTitle);
    assert_eq!(p.position.line, 2);
    assert_eq!(p.position.column, 1);
    assert_eq!(p.position.offset, 18);

    Ok(())
}


#[test]
fn test_accent() -> Result<(), Box<Error>> {
    let p = parse("assets/tests/errors/test-accent.dex");
    assert!(p.is_err());

    let p = p.err().unwrap();

    println!("{}", p);
    assert_eq!(p.errors.len(), 1);

    let e = &p.errors[0];
    assert_eq!(e.ty, ErrorType::UnmatchedStar);
    assert_eq!(e.position.line, 1);
    assert_eq!(e.position.column, 46);
    assert_eq!(e.position.offset, 46);

    Ok(())
}

