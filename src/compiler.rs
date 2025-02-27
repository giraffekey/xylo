#[cfg(feature = "std")]
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

#[cfg(feature = "no-std")]
use {
    alloc::{boxed::Box, collections::BTreeMap, format, sync::Arc, vec, vec::Vec},
    spin::Mutex,
};

use crate::functions::{handle_builtin, BUILTIN_FUNCTIONS};
use crate::parser::*;
use crate::shape::{unwrap_shape, Shape, CIRCLE, FILL, SQUARE, TRIANGLE};

use anyhow::{anyhow, Result};
use rayon::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum ValueKind {
    Integer,
    Float,
    Boolean,
    Shape,
    List(Box<ValueKind>),
    Unknown,
}

#[derive(Debug, Clone)]
pub enum Value {
    Integer(i32),
    Float(f32),
    Boolean(bool),
    Shape(Arc<Mutex<Shape>>),
    List(Vec<Value>),
}

impl Value {
    pub fn kind(&self) -> Result<ValueKind> {
        match self {
            Self::Integer(_) => Ok(ValueKind::Integer),
            Self::Float(_) => Ok(ValueKind::Float),
            Self::Boolean(_) => Ok(ValueKind::Boolean),
            Self::Shape(_) => Ok(ValueKind::Shape),
            Self::List(list) => {
                let kind = list
                    .get(0)
                    .map(Self::kind)
                    .unwrap_or(Ok(ValueKind::Unknown))?;
                if list
                    .iter()
                    .map(Self::kind)
                    .all(|other| other.is_ok() && other.unwrap() == kind)
                {
                    Ok(ValueKind::List(Box::new(kind)))
                } else {
                    Err(anyhow!("Type mismatch in list."))
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Stack<'a> {
    functions: BTreeMap<&'a str, Function<'a>>,
    depth: usize,
}

#[derive(Debug, Clone)]
enum Block<'a> {
    Value(Value),
    Expr(Expr<'a>),
}

#[derive(Debug, Clone)]
struct Function<'a> {
    params: Vec<&'a str>,
    block: Block<'a>,
}

fn reduce_literal(literal: &Literal) -> Result<Value> {
    match literal {
        Literal::Integer(n) => Ok(Value::Integer(*n)),
        Literal::Float(n) => Ok(Value::Float(*n)),
        Literal::Boolean(b) => Ok(Value::Boolean(*b)),
        Literal::Shape(kind) => {
            let shape = match kind {
                ShapeKind::Square => SQUARE,
                ShapeKind::Circle => CIRCLE,
                ShapeKind::Triangle => TRIANGLE,
                ShapeKind::Fill => FILL,
            };
            Ok(Value::Shape(Arc::new(Mutex::new(Shape::Basic(shape)))))
        }
    }
}

fn reduce_binary_operation(stack: &Stack, operation: &BinaryOperation) -> Result<Value> {
    match (&*operation.a, &*operation.b) {
        (Expr::Literal(_), Expr::Literal(_)) | (Expr::Literal(_), _) | (_, Expr::Literal(_)) => {
            // Calculate synchronously if non-recursive
            let a = reduce_expr(stack, &operation.a)?;
            let b = reduce_expr(stack, &operation.b)?;
            handle_builtin(operation.op.as_str(), &[a, b])
        }
        _ => {
            // Calculate in parallel if recursion is possible
            let args: Result<Vec<Value>> = [operation.a.clone(), operation.b.clone()]
                .into_par_iter()
                .map(|arg| reduce_expr(stack, &arg))
                .collect();
            handle_builtin(operation.op.as_str(), &args?)
        }
    }
}

fn reduce_call(stack: &Stack, call: &Call) -> Result<Value> {
    if BUILTIN_FUNCTIONS.contains(&call.name) {
        let args: Result<Vec<Value>> = call
            .args
            .clone()
            .into_par_iter()
            .map(|arg| reduce_expr(stack, &arg))
            .collect();
        return handle_builtin(call.name, &args?);
    }

    match stack.functions.get(call.name) {
        Some(function) => {
            // Temporary. Will implement currying later.
            if call.args.len() != function.params.len() {
                return Err(anyhow!("Incorrect number of arguments."));
            }

            match function.block.clone() {
                Block::Value(value) => Ok(value),
                Block::Expr(expr) => {
                    let functions: Result<Vec<(&str, Function)>> = function
                        .params
                        .clone()
                        .par_iter()
                        .zip(call.args.clone())
                        .map(|(param, arg)| {
                            let arg = reduce_expr(stack, &arg)?;
                            Ok((
                                *param,
                                Function {
                                    params: vec![],
                                    block: Block::Value(arg),
                                },
                            ))
                        })
                        .collect();

                    let mut stack = stack.clone();
                    stack.depth += 1;
                    stack.functions.extend(functions?);
                    reduce_expr(&stack, &expr)
                }
            }
        }
        None => Err(anyhow!(format!("Function `{}` not found", call.name))),
    }
}

fn reduce_let<'a>(stack: &Stack<'a>, let_statement: &Let<'a>) -> Result<Value> {
    let mut stack = stack.clone();
    stack.functions.extend(
        let_statement
            .definitions
            .clone()
            .into_iter()
            .map(|definition| {
                (
                    definition.name,
                    Function {
                        params: definition.params,
                        block: Block::Expr(definition.block),
                    },
                )
            }),
    );

    reduce_expr(&stack, &let_statement.block)
}

fn reduce_if(stack: &Stack, if_statement: &If) -> Result<Value> {
    let condition = reduce_expr(stack, &if_statement.condition)?;
    let is_true = match condition {
        Value::Boolean(b) => b,
        _ => return Err(anyhow!("If condition must reduce to a boolean.")),
    };
    if is_true {
        reduce_expr(stack, &if_statement.if_block)
    } else {
        reduce_expr(stack, &if_statement.else_block)
    }
}

fn reduce_match(stack: &Stack, match_statement: &Match) -> Result<Value> {
    let a = reduce_expr(stack, &match_statement.condition)?;
    for (pattern, block) in &match_statement.patterns {
        match pattern {
            Pattern::Matches(matches) => {
                for b in matches {
                    let b = reduce_expr(stack, &b)?;
                    let matching = match (&a, &b) {
                        (Value::Integer(a), Value::Integer(b)) => a == b,
                        (Value::Float(a), Value::Float(b)) => a == b,
                        (Value::Integer(a), Value::Float(b))
                        | (Value::Float(b), Value::Integer(a)) => *a as f32 == *b,
                        (Value::Boolean(a), Value::Boolean(b)) => a == b,
                        (Value::Shape(_a), Value::Shape(_b)) => todo!(),
                        (Value::Integer(a), Value::List(list)) => list
                            .iter()
                            .map(|value| match value {
                                Value::Integer(b) => Ok(*a == *b),
                                _ => Err(anyhow!("Incorrect type comparison in match statement.")),
                            })
                            .try_fold(false, |acc, res| res.map(|x| acc || x))?,
                        (Value::Float(a), Value::List(list)) => list
                            .iter()
                            .map(|value| match value {
                                Value::Float(b) => Ok(*a == *b),
                                _ => Err(anyhow!("Incorrect type comparison in match statement.")),
                            })
                            .try_fold(false, |acc, res| res.map(|x| acc || x))?,
                        (Value::Boolean(a), Value::List(list)) => list
                            .iter()
                            .map(|value| match value {
                                Value::Boolean(b) => Ok(*a == *b),
                                _ => Err(anyhow!("Incorrect type comparison in match statement.")),
                            })
                            .try_fold(false, |acc, res| res.map(|x| acc || x))?,
                        (Value::Shape(_a), Value::List(_list)) => todo!(),
                        _ => return Err(anyhow!("Incorrect type comparison in match statement.")),
                    };
                    if matching {
                        return reduce_expr(stack, &block);
                    }
                }
            }
        }
    }
    return Err(anyhow!("Not all possibilities covered in match statement"));
}

fn reduce_for(stack: &Stack, for_statement: &For) -> Result<Value> {
    let iter = reduce_expr(stack, &for_statement.iter)?;
    let items = match iter {
        Value::List(list) => list,
        Value::Integer(n) => {
            if n < 0 {
                return Err(anyhow!("Cannot iterate over negative number."));
            }
            (0..n).map(Value::Integer).collect()
        }
        Value::Float(n) => {
            if n < 0.0 {
                return Err(anyhow!("Cannot iterate over negative number."));
            }
            (0..n as i32).map(Value::Integer).collect()
        }
        _ => return Err(anyhow!("Value is not iterable.")),
    };

    let res: Result<Vec<Value>> = items
        .into_par_iter()
        .map(|item| {
            let mut stack = stack.clone();
            stack.functions.insert(
                for_statement.var,
                Function {
                    params: vec![],
                    block: Block::Value(item),
                },
            );
            reduce_expr(&stack, &for_statement.block)
        })
        .collect();

    let list = Value::List(res?);
    list.kind()?;
    Ok(list)
}

fn reduce_loop(stack: &Stack, loop_statement: &Loop) -> Result<Value> {
    let count = reduce_expr(stack, &loop_statement.count)?;
    let count = match count {
        Value::Integer(n) => n,
        Value::Float(n) => n as i32,
        _ => return Err(anyhow!("Value must be a number.")),
    };
    if count < 0 {
        return Err(anyhow!("Cannot iterate over negative number."));
    }

    let res: Result<Vec<Value>> = (0..count)
        .into_par_iter()
        .map(|_| reduce_expr(stack, &loop_statement.block))
        .collect();

    let list = Value::List(res?);
    list.kind()?;
    Ok(list)
}

fn reduce_expr(stack: &Stack, expr: &Expr) -> Result<Value> {
    match expr {
        Expr::Literal(literal) => reduce_literal(literal),
        Expr::BinaryOperation(operation) => reduce_binary_operation(stack, operation),
        Expr::Call(call) => reduce_call(stack, call),
        Expr::Let(let_statement) => reduce_let(&stack, let_statement),
        Expr::If(if_statement) => reduce_if(stack, if_statement),
        Expr::Match(match_statement) => reduce_match(stack, match_statement),
        Expr::For(for_statement) => reduce_for(stack, for_statement),
        Expr::Loop(loop_statement) => reduce_loop(stack, loop_statement),
    }
}

pub fn compile(tree: Tree) -> Result<Shape> {
    let functions = tree
        .into_iter()
        .map(|definition| {
            (
                definition.name,
                Function {
                    params: definition.params,
                    block: Block::Expr(definition.block),
                },
            )
        })
        .collect();
    let stack = Stack {
        functions,
        depth: 0,
    };
    let shape = match stack.functions.get("root") {
        Some(root_fn) => {
            let value = match root_fn.block.clone() {
                Block::Value(value) => value,
                Block::Expr(expr) => reduce_expr(&stack, &expr)?,
            };
            match value {
                Value::Shape(shape) => shape,
                _ => return Err(anyhow!("The `root` function must return a shape.")),
            }
        }
        None => return Err(anyhow!("Could not find `root` function definition.")),
    };
    Ok(unwrap_shape(shape)?)
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use core::f32::consts::PI;

    #[test]
    fn test() {
        //         let (_, tree) = parse(
        //             "
        // root =
        // 	let shape = SQUARE
        // 		s (pi * 25) 50 (add_circle shape)

        // add_circle shape = shape : CIRCLE
        //     		",
        //         )
        //         .unwrap();
        //         let shape = compile(tree).unwrap();
        //         assert_eq!(
        //             shape,
        //             Shape::Composite {
        //                 a: Box::new(SQUARE.clone()),
        //                 b: Box::new(CIRCLE.clone()),
        //                 transform: Transform::from_scale(PI * 25.0, 50.0),
        //                 color: TRANSPARENT,
        //             }
        //         );
    }
}
