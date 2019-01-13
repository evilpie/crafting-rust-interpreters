use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use crate::value::Value;
use crate::execute::{VMError, VMResult};

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

    pub fn define(&mut self, name: String, value: Value) {
        self.bindings.insert(name, value);
    }

    pub fn set(&mut self, name: String, value: Value) -> VMResult {
        if self.bindings.contains_key(&name) {
            self.bindings.insert(name, value.clone());
            return Ok(value);
        }

        if let Some(ref env) = self.enclosing {
            return env.borrow_mut().set(name, value);
        }

        Err(VMError::Message(format!("no such variable '{}'", name)))
    }

    pub fn get(&self, name: &str) -> VMResult {
        if let Some(val) = self.bindings.get(name) {
            return Ok(val.clone());
        }

        if let Some(ref env) = self.enclosing {
            return env.borrow().get(name);
        }

        Err(VMError::Message(format!("no such variable '{}'", name)))
    }
}