#[cfg(feature = "std")]
use std::rc::Rc;

#[cfg(feature = "no-std")]
use alloc::{boxed::Box, format, rc::Rc, vec, vec::Vec};

use crate::cache::Cache;
use crate::functions::{handle_builtin, BUILTIN_FUNCTIONS, RAND_FUNCTIONS};
use crate::parser::*;
use crate::shape::{Shape, CIRCLE, EMPTY, FILL, SQUARE, TRIANGLE};

use anyhow::{anyhow, Result};
use core::cell::RefCell;
use hashbrown::HashMap;
use rand::distr::{weighted::WeightedIndex, Distribution};

#[derive(Debug, Clone, PartialEq)]
pub enum ValueKind {
    Integer,
    Float,
    Boolean,
    Hex,
    Shape,
    List(Box<ValueKind>),
    Unknown,
}

#[derive(Debug, Clone)]
pub enum Value {
    Integer(i32),
    Float(f32),
    Boolean(bool),
    Hex([u8; 3]),
    Shape(Rc<RefCell<Shape>>),
    List(Vec<Value>),
}

impl Value {
    pub fn kind(&self) -> Result<ValueKind> {
        match self {
            Self::Integer(_) => Ok(ValueKind::Integer),
            Self::Float(_) => Ok(ValueKind::Float),
            Self::Boolean(_) => Ok(ValueKind::Boolean),
            Self::Hex(_) => Ok(ValueKind::Hex),
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

#[derive(Debug)]
struct Stack<'a> {
    frames: Vec<HashMap<&'a str, Function<'a>>>,
    operands: Vec<Value>,
    scope: Option<(&'a str, usize)>,
}

impl Stack<'_> {
    pub fn get_function(&self, name: &str) -> Option<&Function> {
        self.frames
            .iter()
            .rev()
            .find_map(|functions| match functions.get(name) {
                Some(function) if function.scope.is_none() || function.scope == self.scope => {
                    Some(function)
                }
                _ => None,
            })
    }
}

#[derive(Debug, Clone)]
enum FunctionBlock<'a> {
    Value(Value),
    Block(Block<'a>),
}

#[derive(Debug, Clone)]
struct Function<'a> {
    params: Vec<&'a str>,
    weighted: bool,
    blocks: Vec<(FunctionBlock<'a>, f32)>,
    scope: Option<(&'a str, usize)>,
}

fn reduce_literal(literal: &Literal) -> Result<Value> {
    match literal {
        Literal::Integer(n) => Ok(Value::Integer(*n)),
        Literal::Float(n) => Ok(Value::Float(*n)),
        Literal::Boolean(b) => Ok(Value::Boolean(*b)),
        Literal::Hex(hex) => Ok(Value::Hex(*hex)),
        Literal::Shape(kind) => {
            let shape = match kind {
                ShapeKind::Square => SQUARE,
                ShapeKind::Circle => CIRCLE,
                ShapeKind::Triangle => TRIANGLE,
                ShapeKind::Fill => FILL,
                ShapeKind::Empty => EMPTY,
            };
            Ok(Value::Shape(Rc::new(RefCell::new(Shape::Basic(shape)))))
        }
        Literal::List(list) => {
            let list: Result<Vec<Value>> = list.iter().map(reduce_literal).collect();
            let list = Value::List(list?);
            list.kind()?;
            Ok(list)
        }
    }
}

fn reduce_call<'a>(
    stack: &mut Stack<'a>,
    cache: &mut Cache,
    name: &'a str,
    argc: usize,
) -> Result<Value> {
    let mut args = Vec::with_capacity(argc);
    for _ in 0..argc {
        args.push(stack.operands.pop().unwrap());
    }
    args.reverse();

    if BUILTIN_FUNCTIONS.contains(&name) {
        if RAND_FUNCTIONS.contains(&name) {
            handle_builtin(name, cache, &args)
        } else {
            let key = Cache::hash_call(name, 0, &args, stack.scope);
            if let Some(value) = cache.get(key) {
                return Ok(value);
            }

            let value = handle_builtin(name, cache, &args)?;
            cache.insert(key, &value);
            Ok(value)
        }
    } else {
        match stack.get_function(name) {
            Some(function) => {
                // Temporary. Will implement currying later.
                if args.len() != function.params.len() {
                    return Err(anyhow!(format!(
                        "Incorrect number of arguments passed to `{}` function.",
                        name
                    )));
                }

                let (i, block) = if function.weighted {
                    let index_weights = function.blocks.iter().enumerate();
                    let dist =
                        WeightedIndex::new(index_weights.clone().map(|(_, (_, weight))| weight))
                            .unwrap();
                    index_weights
                        .map(|(i, (block, _))| (i, block))
                        .nth(dist.sample(&mut cache.rng))
                        .unwrap()
                } else {
                    (0, &function.blocks[0].0)
                };

                let key = if function.scope.is_none() {
                    let key = Cache::hash_call(name, i, &args, stack.scope);
                    if let Some(value) = cache.get(key) {
                        return Ok(value);
                    }
                    Some(key)
                } else {
                    None
                };

                let scope = Some((name, i));
                match block {
                    FunctionBlock::Value(value) => Ok(value.clone()),
                    FunctionBlock::Block(block) => {
                        let functions = function
                            .params
                            .clone()
                            .iter()
                            .zip(args)
                            .map(|(param, arg)| {
                                (
                                    *param,
                                    Function {
                                        params: vec![],
                                        weighted: false,
                                        blocks: vec![(FunctionBlock::Value(arg), 0.0)],
                                        scope,
                                    },
                                )
                            })
                            .collect();

                        let mut frames = stack.frames.clone();
                        frames.push(functions);

                        let mut stack = Stack {
                            frames,
                            operands: Vec::new(),
                            scope,
                        };
                        let value = reduce_block(&mut stack, cache, block)?;

                        if let Some(key) = key {
                            cache.insert(key, &value);
                        }

                        Ok(value)
                    }
                }
            }
            None => Err(anyhow!(format!("Function `{}` not found", name))),
        }
    }
}

fn pattern_match(a: &Value, b: &Literal) -> Result<bool> {
    match (&a, &b) {
        (Value::Integer(a), Literal::Integer(b)) => Ok(a == b),
        (Value::Float(a), Literal::Float(b)) => Ok(a == b),
        (Value::Integer(a), Literal::Float(b)) | (Value::Float(b), Literal::Integer(a)) => {
            Ok(*a as f32 == *b)
        }
        (Value::Boolean(a), Literal::Boolean(b)) => Ok(a == b),
        (Value::Hex(a), Literal::Hex(b)) => Ok(a == b),
        (Value::Shape(_a), Literal::Shape(_b)) => todo!(),
        (Value::List(a), Literal::List(b)) => {
            if a.len() != b.len() {
                return Ok(false);
            }
            let matches: Result<Vec<bool>> =
                a.iter().zip(b).map(|(a, b)| pattern_match(a, b)).collect();
            Ok(matches?.iter().all(|does_match| *does_match))
        }
        _ => return Err(anyhow!("Incorrect type comparison in match statement.")),
    }
}

fn reduce_block<'a>(
    stack: &mut Stack<'a>,
    cache: &mut Cache,
    block: &[Token<'a>],
) -> Result<Value> {
    let mut index = 0;
    let mut for_stack = Vec::new();
    let mut loop_stack = Vec::new();

    while index < block.len() {
        match &block[index] {
            Token::Literal(literal) => {
                stack.operands.push(reduce_literal(literal)?);
                index += 1;
            }
            Token::BinaryOperator(op) => {
                let b = stack.operands.pop().unwrap();
                let a = stack.operands.pop().unwrap();
                let value = handle_builtin(op.as_str(), cache, &[a, b])?;
                stack.operands.push(value);
                index += 1;
            }
            Token::Call(name, argc) => {
                let value = reduce_call(stack, cache, name, *argc)?;
                stack.operands.push(value);
                index += 1;
            }
            Token::Jump(skip) => index += skip + 1,
            Token::Pop => {
                stack.frames.pop().unwrap();
                index += 1;
            }
            Token::Let(definitions) => {
                stack.frames.push(
                    definitions
                        .iter()
                        .map(|definition| {
                            (
                                definition.name,
                                Function {
                                    params: definition.params.clone(),
                                    weighted: false,
                                    blocks: vec![(
                                        FunctionBlock::Block(definition.block.clone()),
                                        0.0,
                                    )],
                                    scope: stack.scope,
                                },
                            )
                        })
                        .collect(),
                );
                index += 1;
            }
            Token::If(skip) => {
                let condition = stack.operands.pop().unwrap();
                let is_true = match condition {
                    Value::Boolean(b) => b,
                    _ => return Err(anyhow!("If condition must reduce to a boolean.")),
                };

                if is_true {
                    index += 1;
                } else {
                    index += skip + 1;
                }
            }
            Token::Match(patterns) => {
                let a = stack.operands.pop().unwrap();

                let mut found = false;
                'a: for (pattern, skip) in patterns {
                    match pattern {
                        Pattern::Matches(matches) => {
                            for b in matches {
                                if pattern_match(&a, b)? {
                                    found = true;
                                    break 'a;
                                }
                                index += skip;
                            }
                        }
                        Pattern::Wildcard => {
                            found = true;
                            break 'a;
                        }
                    }
                }

                if !found {
                    return Err(anyhow!("Not all possibilities covered in match statement"));
                }

                index += 1;
            }
            Token::ForStart(var) => {
                let iter = stack.operands.pop().unwrap();
                let mut items = match iter {
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
                items.reverse();

                stack.frames.push(
                    [(
                        *var,
                        Function {
                            params: vec![],
                            weighted: false,
                            blocks: vec![(FunctionBlock::Value(items.pop().unwrap()), 0.0)],
                            scope: stack.scope,
                        },
                    )]
                    .into(),
                );

                for_stack.push((index, *var, items, Vec::new()));
                index += 1;
            }
            Token::ForEnd => {
                if let Some((start, var, items, values)) = for_stack.last_mut() {
                    values.push(stack.operands.pop().unwrap());

                    if items.len() > 0 {
                        stack.frames.push(
                            [(
                                *var,
                                Function {
                                    params: vec![],
                                    weighted: false,
                                    blocks: vec![(FunctionBlock::Value(items.pop().unwrap()), 0.0)],
                                    scope: stack.scope,
                                },
                            )]
                            .into(),
                        );
                        index = *start + 1; // Jump back to ForStart
                    } else {
                        let list = Value::List(values.to_vec());
                        list.kind()?;
                        stack.operands.push(list);

                        for_stack.pop().unwrap(); // Exit for loop
                        index += 1;
                    }
                }
            }
            Token::LoopStart => {
                let count = stack.operands.pop().unwrap();
                let count = match count {
                    Value::Integer(n) => n,
                    Value::Float(n) => n as i32,
                    _ => return Err(anyhow!("Value must be a number.")),
                };
                if count < 0 {
                    return Err(anyhow!("Cannot iterate over negative number."));
                }

                loop_stack.push((index, count, Vec::new()));
                index += 1;
            }
            Token::LoopEnd => {
                if let Some((start, remaining, values)) = loop_stack.last_mut() {
                    *remaining -= 1;
                    values.push(stack.operands.pop().unwrap());

                    if *remaining > 0 {
                        index = *start + 1; // Jump back to LoopStart
                    } else {
                        let list = Value::List(values.to_vec());
                        list.kind()?;
                        stack.operands.push(list);

                        loop_stack.pop().unwrap(); // Exit loop
                        index += 1;
                    }
                }
            }
        }
    }

    stack
        .operands
        .pop()
        .ok_or_else(|| anyhow!("Missing operand"))
}

pub fn reduce(tree: Tree, seed: Option<[u8; 32]>) -> Result<Shape> {
    let mut functions: HashMap<&str, Function> = HashMap::new();
    for definition in tree {
        match functions.get_mut(definition.name) {
            Some(function) => {
                if definition.params != function.params {
                    return Err(anyhow!("Incorrect parameters in duplicate function."));
                }

                function.weighted = true;
                function
                    .blocks
                    .push((FunctionBlock::Block(definition.block), definition.weight));
            }
            None => {
                functions.insert(
                    definition.name,
                    Function {
                        params: definition.params,
                        weighted: false,
                        blocks: vec![(FunctionBlock::Block(definition.block), definition.weight)],
                        scope: None,
                    },
                );
            }
        }
    }

    let mut stack = Stack {
        frames: vec![functions],
        operands: Vec::new(),
        scope: None,
    };
    let mut cache = Cache::new(seed)?;

    let shape = match reduce_call(&mut stack, &mut cache, "root", 0)? {
        Value::Shape(shape) => shape,
        _ => return Err(anyhow!("The `root` function must return a shape.")),
    };
    Ok(Rc::try_unwrap(shape).unwrap().into_inner())
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
        //  let shape = SQUARE
        //      s (pi * 25) 50 (add_circle shape)

        // add_circle shape = shape : CIRCLE
        //          ",
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
