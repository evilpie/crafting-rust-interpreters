use std::rc::Rc;
use std::cell::RefCell;

use crate::parser::{Expr, Node};
use crate::value::Value;
use crate::environment::Environment;

#[derive(Debug)]
pub enum VMError {
    Message(String),
    Return(Value)
}

pub type VMResult = Result<Value, VMError>;

fn err(msg: &str) -> VMResult {
    Err(VMError::Message(msg.to_string()))
}

fn get(base: Value, key: Value) -> VMResult {
    if let Value::Array(ref array) = base {
        match key {
            Value::Number(n) => {
                if n < 0 {
                    return err("negative array index");
                }

                match array.borrow().get(n as usize) {
                    Some(v) => Ok(v.clone()),
                    _ => err("array index of range")
                }
            }
            Value::String(ref string) if string == "length" => {
                Ok(Value::Number(array.borrow().len() as i32))
            }
            _ => err("invalid key")
        }
    } else {
        err("invalid base")
    }
}

// Todo: This is probably going to require a different ownership story
pub fn execute_node(node: &Box<Node>, env: &Rc<RefCell<Environment>>) -> VMResult {
    match **node {
        Node::Statements(ref statements) => {
            let mut last = Value::Nothing;
            for node in statements {
                last = execute_node(&node, env)?;
            }
            Ok(last)
        }

        Node::ExpressionStatement(ref expr) => {
            execute_expr(&expr, env)
        }

        Node::Fun(ref name, ref parameters, ref body) => {
            // ToDo: This probably leaks the environment.
            env.borrow_mut().set(name.clone(), Value::Function(parameters.clone(), body.clone(), env.clone()));
            Ok(Value::Nothing)
        }

        Node::Return(ref expr) => {
            let expr = execute_expr(&expr, env)?;
            Err(VMError::Return(expr))
        }

        Node::Print(ref expr) => {
            let expr = execute_expr(&expr, env)?;
            println!("print: {:?}", expr);
            Ok(expr)
        }

        Node::While(ref condition, ref block) => {
            loop {
                match execute_expr(&condition, env)? {
                    Value::Boolean(true) => execute_node(&block, env)?,
                    Value::Boolean(false) => break,
                    _ => return err("while expects boolean operand"),
                };
            }

            Ok(Value::Nothing)
        },

        Node::If(ref condition, ref then, ref other) => match execute_expr(&condition, env)? {
            Value::Boolean(true) => execute_node(&then, env),
            Value::Boolean(false) => execute_node(&other, env),
            _ => err("if expects boolean operand"),
        },
    }
}

fn execute_expr(expr: &Box<Expr>, env: &Rc<RefCell<Environment>>) -> VMResult {
    match **expr {
        Expr::Eq(ref l, ref r) => {
            let left = execute_expr(&l, env)?;
            let right = execute_expr(&r, env)?;
            match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a == b)),
                _ => err("Unexpected Eq operands"),
            }
        }
        Expr::Ne(ref l, ref r) => {
            let left = execute_expr(&l, env)?;
            let right = execute_expr(&r, env)?;
            match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a != b)),
                _ => err("Unexpected Ne operands"),
            }
        }
        Expr::Greater(ref l, ref r) => {
            let left = execute_expr(&l, env)?;
            let right = execute_expr(&r, env)?;
            match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a > b)),
                _ => err("Unexpected > operands"),
            }
        }
        Expr::GreaterEqual(ref l, ref r) => {
            let left = execute_expr(&l, env)?;
            let right = execute_expr(&r, env)?;
            match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a >= b)),
                _ => err("Unexpected >= operands"),
            }
        }
        Expr::Less(ref l, ref r) => {
            let left = execute_expr(&l, env)?;
            let right = execute_expr(&r, env)?;
            match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a < b)),
                _ => err("Unexpected < operands"),
            }
        }
        Expr::LessEqual(ref l, ref r) => {
            let left = execute_expr(&l, env)?;
            let right = execute_expr(&r, env)?;
            match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a <= b)),
                _ => err("Unexpected <= operands"),
            }
        }
        Expr::Plus(ref l, ref r) => {
            let left = execute_expr(&l, env)?;
            let right = execute_expr(&r, env)?;
            match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                _ => err("Unexpected Plus operands"),
            }
        }
        Expr::Minus(ref l, ref r) => {
            let left = execute_expr(&l, env)?;
            let right = execute_expr(&r, env)?;
            match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
                _ => err("Unexpected Minus operands"),
            }
        }
        Expr::Multiply(ref l, ref r) => {
            let left = execute_expr(&l, env)?;
            let right = execute_expr(&r, env)?;
            match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
                _ => err("Unexpected Multiply operands"),
            }
        }
        Expr::Number(n) => Ok(Value::Number(n)),
        Expr::String(ref string) => Ok(Value::String(string.clone())),
        Expr::Boolean(b) => Ok(Value::Boolean(b)),
        Expr::Call(ref callee, ref arguments) => {
            match execute_expr(&callee, env)? {
                Value::NativeFunction(ref fun) => {
                    let args: Result<Vec<Value>, _> = arguments.iter().map(|arg| {
                        execute_expr(&arg, env)
                    }).collect();

                    Ok(fun(args?))
                },
                Value::Function(ref parameters, ref body, ref scope) => {
                    let args: Result<Vec<Value>, _> = arguments.iter().map(|arg| {
                        execute_expr(&arg, env)
                    }).collect();

                    // ToDo: argument count != paramter count
                    let local = Rc::new(RefCell::new(Environment::new_enclosing(scope.clone())));
                    for (name, arg) in parameters.iter().zip(args?) {
                        local.borrow_mut().set(name.clone(), arg);
                    }

                    match execute_node(&body, &local) {
                        Err(VMError::Return(v)) => Ok(v.clone()),
                        e @ Err(_) => e,
                        Ok(_) => Ok(Value::Nothing) // No implicit return!
                    }
                }
                _ => err("expected function callee")
            }
        }
        Expr::Array(ref values) => {
            let vals: Result<Vec<Value>, _> = values.iter().map(|arg| {
                execute_expr(&arg, env)
            }).collect();

            Ok(Value::Array(Rc::new(RefCell::new(vals?))))
        }
        Expr::Assign(ref name, ref expr) => {
            let right = execute_expr(&expr, env)?;
            env.borrow_mut().set(name.to_string(), right.clone());
            Ok(right.clone())
        }
        Expr::Identifier(ref name) => match env.borrow().get(name) {
            Some(v) => Ok(v.clone()),
            None => Err(VMError::Message(format!("no such variable '{}'", name))),
        }
        Expr::Get(ref b, ref k) => {
            let base = execute_expr(b, env)?;
            let key = execute_expr(k, env)?;

            get(base, key)
        }
        Expr::Set(ref b, ref k, ref v) => {
            let base = execute_expr(b, env)?;
            let key = execute_expr(k, env)?;
            let value = execute_expr(v, env)?;

            match (base, key) {
                (Value::Array(ref array), Value::Number(n)) if n >= 0 => {
                    match array.borrow_mut().get_mut(n as usize) {
                        Some(elem) => *elem = value.clone(),
                        _ => return err("array index of range")
                    }
                }
                _ => return err("array only")
            }

            Ok(value)
        }
    }
}
