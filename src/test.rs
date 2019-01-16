use std::error::Error;

use crate::parser::{Node, Parser};
use crate::scanner::scan;

fn parse(source: &str) -> Result<Node, Box<dyn Error>> {
    let tokens = scan(source)?;
    let mut parser = Parser::new(tokens);
    Ok(parser.parse()?)
}

#[test]
fn simple_addition() {
    assert!(parse("1 + 1;").is_ok());

    assert!(parse("1 + 1").is_err());
}

#[test]
fn call_call() {
    assert!(parse("a()();").is_ok());

    assert!(parse("a()()").is_err());
}

#[test]
fn assignment() {
    assert!(parse("a = 1;").is_ok());
    assert!(parse("a = 1 + 1;").is_ok());
    assert!(parse("a = b = 1 + 1;").is_ok());

    assert!(parse("a = 1 + 1").is_err());
    assert!(parse("123 = 1 + 1;").is_err());
    assert!(parse("a = 123 = 1 + 1;").is_err());
}

#[test]
fn for_statement() {
    assert!(parse("for (a = 1; a < 10; i = i + 1) print i;").is_ok());
    assert!(parse("for (a = 1; a < 10; i = i + 1) { print i; }").is_ok());
    assert!(parse("for (var a = 1; a < 10; i = i + 1) print i;").is_ok());
    assert!(parse("for (var a = 1; a < 10; i = i + 1) { print i; }").is_ok());
}
