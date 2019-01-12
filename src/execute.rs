use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use crate::parser::{Expr, Node};

#[derive(Debug, Clone)]
pub enum Value {
    Number(i32),
    Boolean(bool),
    NativeFunction(fn(Vec<Value>) -> Value),
    Function(Vec<String>, Box<Node>, Rc<RefCell<Environment>>)
}

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

// Todo: This is probably going to require a different ownership story
pub fn execute_node(node: &Box<Node>, env: &Rc<RefCell<Environment>>) {
    match **node {
        Node::Statements(ref statements) => {
            for node in statements {
                execute_node(&node, env)
            }
        }

        Node::ExpressionStatement(ref expr) => {
            execute_expr(&expr, env);
        }

        Node::Fun(ref name, ref parameters, ref body) => {
            // ToDo: This probably leaks the environment.
            env.borrow_mut().set(name.clone(), Value::Function(parameters.clone(), body.clone(), env.clone()));
        }

        Node::Print(ref expr) => {
            println!("print: {:?}", execute_expr(&expr, env));
        }

        Node::While(ref condition, ref block) => loop {
            match execute_expr(&condition, env) {
                Value::Boolean(true) => execute_node(&block, env),
                Value::Boolean(false) => break,
                _ => panic!("while expects boolean operand"),
            }
        },

        Node::If(ref condition, ref then, ref other) => match execute_expr(&condition, env) {
            Value::Boolean(true) => execute_node(&then, env),
            Value::Boolean(false) => execute_node(&other, env),
            _ => panic!("if expects boolean operand"),
        },
    }
}

fn execute_expr(expr: &Box<Expr>, env: &Rc<RefCell<Environment>>) -> Value {
    match **expr {
        Expr::Eq(ref l, ref r) => {
            let left = execute_expr(&l, env);
            let right = execute_expr(&r, env);
            match (left, right) {
                (Value::Number(a), Value::Number(b)) => Value::Boolean(a == b),
                _ => panic!("Unexpected Eq operands"),
            }
        }
        Expr::Ne(ref l, ref r) => {
            let left = execute_expr(&l, env);
            let right = execute_expr(&r, env);
            match (left, right) {
                (Value::Number(a), Value::Number(b)) => Value::Boolean(a != b),
                _ => panic!("Unexpected Ne operands"),
            }
        }
        Expr::Plus(ref l, ref r) => {
            let left = execute_expr(&l, env);
            let right = execute_expr(&r, env);
            match (left, right) {
                (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
                _ => panic!("Unexpected Plus operands"),
            }
        }
        Expr::Minus(ref l, ref r) => {
            let left = execute_expr(&l, env);
            let right = execute_expr(&r, env);
            match (left, right) {
                (Value::Number(a), Value::Number(b)) => Value::Number(a - b),
                _ => panic!("Unexpected Minus operands"),
            }
        }
        Expr::Multiply(ref l, ref r) => {
            let left = execute_expr(&l, env);
            let right = execute_expr(&r, env);
            match (left, right) {
                (Value::Number(a), Value::Number(b)) => Value::Number(a * b),
                _ => panic!("Unexpected Multiply operands"),
            }
        }
        Expr::Number(n) => Value::Number(n),
        Expr::Boolean(b) => Value::Boolean(b),
        Expr::Call(ref callee, ref arguments) => {
            match execute_expr(&callee, env) {
                Value::NativeFunction(fun) => {
                    let args = arguments.iter().map(|arg| {
                        execute_expr(&arg, env)
                    }).collect();

                    fun(args)
                },
                Value::Function(parameters, body, scope) => {
                    let args = arguments.iter().map(|arg| {
                        execute_expr(&arg, env)
                    });

                    // ToDo: argument count != paramter count
                    let local = Rc::new(RefCell::new(Environment::new_enclosing(scope)));
                    for (name, arg) in parameters.iter().zip(args) {
                        local.borrow_mut().set(name.clone(), arg);
                    }

                    execute_node(&body, &local);
                    // ToDo: return statment
                    Value::Number(0)
                }
                _ => panic!("expected function callee")
            }
        }
        Expr::Assign(ref name, ref expr) => {
            let right = execute_expr(&expr, env);
            env.borrow_mut().set(name.to_string(), right.clone());
            right.clone()
        }
        Expr::Identifier(ref name) => match env.borrow().get(name) {
            Some(v) => v.clone(),
            None => panic!("no such variable '{}'", name),
        },
    }
}
