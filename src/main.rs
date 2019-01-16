use std::cell::RefCell;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::rc::Rc;

mod environment;
mod execute;
mod object;
mod parser;
mod scanner;
#[cfg(test)]
mod test;
mod value;

use crate::environment::Environment;
use crate::execute::execute_node;
use crate::parser::Parser;
use crate::scanner::scan;
use crate::value::Value;

fn println(_base: Option<Value>, args: Vec<Value>) -> Value {
    println!("println: {:?}", args);
    Value::Nothing
}

fn main() -> Result<(), Box<dyn Error>> {
    let name = env::args().nth(1).ok_or("missing file argument")?;
    let mut f = File::open(name)?;

    let mut buffer = String::new();
    f.read_to_string(&mut buffer)?;

    let tokens = scan(&buffer)?;
    println!("{:?}", tokens);

    let mut parser = Parser::new(tokens);
    let node = parser.parse()?;
    println!("{:?}", node);

    let env = Rc::new(RefCell::new(Environment::new()));
    env.borrow_mut()
        .define("println".to_string(), Value::NativeFunction(println));
    match execute_node(&Box::new(node), &env) {
        Ok(v) => println!("ok: {}", v),
        Err(e) => println!("error: {:?}", e),
    }

    // and more! See the other methods for more details.
    Ok(())
}
