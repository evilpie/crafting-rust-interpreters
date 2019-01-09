use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::prelude::*;

mod execute;
mod parser;
mod scanner;

use crate::execute::execute_node;
use crate::parser::Parser;
use crate::scanner::scan;

fn main() -> io::Result<()> {
    let mut f = File::open("test.txt")?;

    let mut buffer = String::new();
    f.read_to_string(&mut buffer)?;

    let tokens = scan(&buffer);
    println!("{:?}", tokens);

    let mut parser = Parser::new(tokens.unwrap());
    let node = parser.parse();
    println!("{:?}", node);

    if node.is_ok() {
        let mut vars = HashMap::new();
        execute_node(&Box::new(node.unwrap()), &mut vars);
    }

    // and more! See the other methods for more details.
    Ok(())
}
