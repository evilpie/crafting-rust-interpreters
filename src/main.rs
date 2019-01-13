use std::fs::File;
use std::io;
use std::env;
use std::io::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

mod execute;
mod parser;
mod scanner;

use crate::execute::{execute_node, Value, Environment};
use crate::parser::Parser;
use crate::scanner::scan;

fn println(args: Vec<Value>) -> Value {
    println!("{:?}", args);
    Value::Nothing
}

fn main() -> io::Result<()> {
    let name = match env::args().nth(1) {
        Some(name) => name,
        _ => panic!("missing file argument")
    };
    let mut f = File::open(name)?;

    let mut buffer = String::new();
    f.read_to_string(&mut buffer)?;

    let tokens = scan(&buffer);
    println!("{:?}", tokens);

    let mut parser = Parser::new(tokens.unwrap());
    let node = parser.parse();
    println!("{:?}", node);

    if node.is_ok() {
        let env = Rc::new(RefCell::new(Environment::new()));
        env.borrow_mut().set("println".to_string(), Value::NativeFunction(println));
        match execute_node(&Box::new(node.unwrap()), &env) {
            Ok(v) => println!("ok: {}", v),
            Err(e) => println!("error: {:?}", e)
        }
    }

    // and more! See the other methods for more details.
    Ok(())
}
