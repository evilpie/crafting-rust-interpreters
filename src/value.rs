use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;

use crate::parser::Node;
use crate::environment::Environment;

#[derive(Debug, Clone)]
pub enum Value {
    Nothing,
    Number(i32),
    String(String),
    Boolean(bool),
    NativeFunction(fn(Option<Value>, Vec<Value>) -> Value),
    Function(Vec<String>, Box<Node>, Rc<RefCell<Environment>>),
    Array(Rc<RefCell<Vec<Value>>>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Nothing => write!(f, "<nothing>"),
            Value::Number(n) => write!(f, "<number: {}>", n),
            Value::String(ref string) => write!(f, "<string: {}>", string),
            Value::Boolean(b) => write!(f, "<boolean: {}>", b),
            Value::NativeFunction(_) => write!(f, "<native function>"),
            Value::Function(_, _, _) => write!(f, "<function>"),
            Value::Array(ref array) => write!(f, "<array: {}>", array.borrow().len()),
        }
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        // println!("dropping {}", self);
    }
}