use std::collections::HashMap;

use crate::parser::{Expr, Node};

#[derive(Debug, Copy, Clone)]
pub enum Value {
    Number(i32),
    Boolean(bool),
}

pub fn execute_node(node: &Box<Node>, vars: &mut HashMap<String, Value>) {
    match **node {
        Node::Statements(ref statements) => {
            for node in statements {
                execute_node(&node, vars)
            }
        }

        Node::ExpressionStatement(ref expr) => {
            execute_expr(&expr, vars);
        }

        Node::Print(ref expr) => {
            println!("print: {:?}", execute_expr(&expr, vars));
        }

        Node::While(ref condition, ref block) => loop {
            match execute_expr(&condition, vars) {
                Value::Boolean(true) => execute_node(&block, vars),
                Value::Boolean(false) => break,
                _ => panic!("while expects boolean operand"),
            }
        },

        Node::If(ref condition, ref then, ref other) => match execute_expr(&condition, vars) {
            Value::Boolean(true) => execute_node(&then, vars),
            Value::Boolean(false) => execute_node(&other, vars),
            _ => panic!("if expects boolean operand"),
        },
    }
}

fn execute_expr(expr: &Box<Expr>, vars: &mut HashMap<String, Value>) -> Value {
    match **expr {
        Expr::Eq(ref l, ref r) => {
            let left = execute_expr(&l, vars);
            let right = execute_expr(&r, vars);
            match (left, right) {
                (Value::Number(a), Value::Number(b)) => Value::Boolean(a == b),
                _ => panic!("Unexpected Eq operands"),
            }
        }
        Expr::Ne(ref l, ref r) => {
            let left = execute_expr(&l, vars);
            let right = execute_expr(&r, vars);
            match (left, right) {
                (Value::Number(a), Value::Number(b)) => Value::Boolean(a != b),
                _ => panic!("Unexpected Ne operands"),
            }
        }
        Expr::Plus(ref l, ref r) => {
            let left = execute_expr(&l, vars);
            let right = execute_expr(&r, vars);
            match (left, right) {
                (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
                _ => panic!("Unexpected Plus operands"),
            }
        }
        Expr::Minus(ref l, ref r) => {
            let left = execute_expr(&l, vars);
            let right = execute_expr(&r, vars);
            match (left, right) {
                (Value::Number(a), Value::Number(b)) => Value::Number(a - b),
                _ => panic!("Unexpected Minus operands"),
            }
        }
        Expr::Multiply(ref l, ref r) => {
            let left = execute_expr(&l, vars);
            let right = execute_expr(&r, vars);
            match (left, right) {
                (Value::Number(a), Value::Number(b)) => Value::Number(a * b),
                _ => panic!("Unexpected Multiply operands"),
            }
        }
        Expr::Number(n) => Value::Number(n),
        Expr::Boolean(b) => Value::Boolean(b),
        Expr::Assign(ref name, ref expr) => {
            let right = execute_expr(&expr, vars);
            vars.insert(name.to_string(), right);
            right
        }
        Expr::Identifier(ref name) => match vars.get(name) {
            Some(v) => *v,
            None => panic!("no such variable '{}'", name),
        },
    }
}
