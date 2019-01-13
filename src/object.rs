use std::collections::HashMap;

use crate::value::Value;
use crate::execute::{VMError, VMResult};

#[derive(Debug)]
pub struct Object {
    fields: HashMap<String, Value>
}

impl Object {
    pub fn new() -> Self {
        Object {
            fields: HashMap::new()
        }
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.fields.insert(name, value);
    }

    pub fn get(&self, name: String) -> VMResult {
        if let Some(val) = self.fields.get(&name) {
            return Ok(val.clone());
        }

        Err(VMError::Message(format!("no property named '{}'", name)))
    }
}