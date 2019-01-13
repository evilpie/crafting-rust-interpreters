use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use crate::value::Value;

#[derive(Debug, Clone)]
pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    bindings: HashMap<String, Value>
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            enclosing: None,
            bindings: HashMap::new()
        }
    }

    pub fn new_enclosing(env: Rc<RefCell<Environment>>) -> Self {
        Environment {
            enclosing: Some(env),
            bindings: HashMap::new()
        }
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.bindings.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(val) = self.bindings.get(name) {
            return Some(val.clone());
        }

        if let Some(ref env) = self.enclosing {
            return env.borrow().get(name)
        }

        None
    }
}