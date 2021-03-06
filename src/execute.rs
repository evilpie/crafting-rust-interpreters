use std::cell::RefCell;
use std::rc::Rc;

use crate::environment::Environment;
use crate::object::Object;
use crate::parser::{Expr, Node};
use crate::value::Value;

#[derive(Debug)]
pub enum VMError {
    Message(String),
    Return(Value),
}

pub type VMResult = Result<Value, VMError>;

fn err(msg: &str) -> VMResult {
    Err(VMError::Message(msg.to_string()))
}

fn array_push(base: Option<Value>, args: Vec<Value>) -> Value {
    if let Some(Value::Array(ref array)) = base {
        array.borrow_mut().extend(args)
    }

    Value::Nothing
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
                    _ => err("array index of range"),
                }
            }
            Value::String(ref string) if string == "length" => {
                Ok(Value::Number(array.borrow().len() as i32))
            }
            Value::String(ref string) if string == "push" => Ok(Value::NativeFunction(array_push)),
            _ => err("invalid key"),
        }
    } else if let Value::Object(ref object) = base {
        if let Value::String(ref string) = key {
            object.borrow().get(string.clone())
        } else {
            err("value lookup only with string key")
        }
    } else {
        err("invalid base")
    }
}

fn call(
    callee: Value,
    base: Option<Value>,
    arguments: &Vec<Box<Expr>>,
    env: &Rc<RefCell<Environment>>,
) -> VMResult {
    match callee {
        Value::NativeFunction(ref fun) => {
            let args: Result<Vec<Value>, _> = arguments
                .iter()
                .map(|arg| execute_expr(&arg, env))
                .collect();

            Ok(fun(base, args?))
        }
        Value::Function(ref parameters, ref body, ref scope) => {
            let args: Result<Vec<Value>, _> = arguments
                .iter()
                .map(|arg| execute_expr(&arg, env))
                .collect();

            // ToDo: argument count != paramter count
            let local = Rc::new(RefCell::new(Environment::new_enclosing(scope.clone())));
            for (name, arg) in parameters.iter().zip(args?) {
                local.borrow_mut().define(name.clone(), arg);
            }

            match execute_node(&body, &local) {
                Err(VMError::Return(v)) => Ok(v.clone()),
                e @ Err(_) => e,
                Ok(_) => Ok(Value::Nothing), // No implicit return!
            }
        }
        _ => err("expected function callee"),
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

        Node::ExpressionStatement(ref expr) => execute_expr(&expr, env),

        Node::Block(ref statements) => {
            let block_scope = Rc::new(RefCell::new(Environment::new_enclosing(env.clone())));
            let mut last = Value::Nothing;
            for node in statements {
                last = execute_node(&node, &block_scope)?;
            }
            Ok(last)
        }

        Node::Var(ref name, ref init) => {
            let value = match init {
                Some(ref expr) => execute_expr(expr, env)?,
                None => Value::Nothing,
            };

            env.borrow_mut().define(name.clone(), value.clone());
            Ok(value)
        }

        Node::Fun(ref name, ref parameters, ref body) => {
            // ToDo: This probably leaks the environment.
            env.borrow_mut().define(
                name.clone(),
                Value::Function(parameters.clone(), body.clone(), env.clone()),
            );
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
        }

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
        Expr::Call(ref c, ref arguments) => {
            let callee = execute_expr(c, env)?;
            call(callee, None, arguments, env)
        }
        Expr::MethodCall(ref b, ref k, ref arguments) => {
            let base = execute_expr(b, env)?;
            let key = execute_expr(k, env)?;

            let callee = get(base.clone(), key)?;
            call(callee, Some(base), arguments, env)
        }
        Expr::Array(ref values) => {
            let vals: Result<Vec<Value>, _> =
                values.iter().map(|arg| execute_expr(&arg, env)).collect();

            Ok(Value::Array(Rc::new(RefCell::new(vals?))))
        }
        Expr::Object(ref fields) => {
            let mut object = Object::new();
            for (name, expr) in fields {
                let value = execute_expr(expr, env)?;
                object.set(name.clone(), value);
            }
            Ok(Value::Object(Rc::new(RefCell::new(object))))
        }
        Expr::Assign(ref name, ref expr) => {
            let right = execute_expr(&expr, env)?;
            env.borrow_mut().set(name.to_string(), right.clone())
        }
        Expr::Identifier(ref name) => env.borrow().get(name),
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
                        _ => return err("array index of range"),
                    }
                }
                (Value::Object(ref object), Value::String(ref string)) => {
                    object.borrow_mut().set(string.clone(), value.clone());
                }
                _ => return err("array only"),
            }

            Ok(value)
        }
    }
}
