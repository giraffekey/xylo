#[cfg(feature = "std")]
use std::rc::Rc;

#[cfg(feature = "no-std")]
use alloc::{boxed::Box, format, rc::Rc, string::String, vec, vec::Vec};

use crate::functions::{handle_builtin, BUILTIN_FUNCTIONS};
use crate::parser::*;
use crate::shape::{Shape, CIRCLE, EMPTY, FILL, SQUARE, TRIANGLE};

use anyhow::{anyhow, Result};
use core::cell::RefCell;
use hashbrown::HashMap;
use rand::distr::{weighted::WeightedIndex, Distribution};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

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
struct Stack {
    frames: Vec<HashMap<String, Function>>,
    operands: Vec<Value>,
    scope: usize,
}

impl Stack {
    pub fn get_function(&self, name: &str) -> Option<&Function> {
        match self.frames[0].get(name) {
            Some(function) => Some(function),
            None => self.frames[self.scope..]
                .iter()
                .rev()
                .find_map(|functions| functions.get(name)),
        }
    }
}

#[derive(Debug, Clone)]
enum FunctionBlock {
    Value(Value),
    Block { start: usize },
}

#[derive(Debug, Clone)]
struct Function {
    params: Vec<String>,
    weighted: bool,
    blocks: Vec<(FunctionBlock, f32)>,
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

fn reduce_call(
    stack: &mut Stack,
    rng: &mut ChaCha8Rng,
    name: &str,
    argc: usize,
) -> Result<FunctionBlock> {
    let mut args = Vec::with_capacity(argc);
    for _ in 0..argc {
        args.push(stack.operands.pop().unwrap());
    }
    args.reverse();

    if BUILTIN_FUNCTIONS.contains(&name) {
        let value = handle_builtin(name, rng, &args)?;
        Ok(FunctionBlock::Value(value))
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

                let (_i, block) = if function.weighted {
                    let index_weights = function.blocks.iter().enumerate();
                    let dist =
                        WeightedIndex::new(index_weights.clone().map(|(_, (_, weight))| weight))
                            .unwrap();
                    index_weights
                        .map(|(i, (block, _))| (i, block))
                        .nth(dist.sample(rng))
                        .unwrap()
                } else {
                    (0, &function.blocks[0].0)
                };

                match block {
                    FunctionBlock::Value(_) => Ok(block.clone()),
                    FunctionBlock::Block { start } => {
                        let start = *start;

                        let functions = function
                            .params
                            .clone()
                            .iter()
                            .zip(args)
                            .map(|(param, arg)| {
                                (
                                    param.into(),
                                    Function {
                                        params: vec![],
                                        weighted: false,
                                        blocks: vec![(FunctionBlock::Value(arg), 0.0)],
                                    },
                                )
                            })
                            .collect();

                        stack.scope = stack.frames.len();
                        stack.frames.push(functions);

                        Ok(FunctionBlock::Block { start })
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

fn execute_block(
    stack: &mut Stack,
    rng: &mut ChaCha8Rng,
    block: &[Token],
    start: usize,
) -> Result<Value> {
    let mut index = start;
    let mut call_stack = Vec::new();
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
                let value = handle_builtin(op.as_str(), rng, &[a, b])?;
                stack.operands.push(value);
                index += 1;
            }
            Token::Call(name, argc) => {
                let function_block = reduce_call(stack, rng, name, *argc)?;
                match function_block {
                    FunctionBlock::Value(value) => {
                        stack.operands.push(value);
                        index += 1;
                    }
                    FunctionBlock::Block { start } => {
                        call_stack.push(index + 1);
                        index = start;
                    }
                }
            }
            Token::Jump(skip) => index += skip + 1,
            Token::Pop => {
                stack.frames.pop().unwrap();
                index += 1;
            }
            Token::Return => match call_stack.pop() {
                Some(last_index) => {
                    stack.frames.pop().unwrap();
                    stack.scope = stack.frames.len() - 1;
                    index = last_index;
                }
                None => return Ok(stack.operands.pop().unwrap()),
            },
            Token::Let(name, params, skip) => {
                stack.frames.push(
                    [(
                        (*name).into(),
                        Function {
                            params: params.iter().map(|s| (*s).into()).collect(),
                            weighted: false,
                            blocks: vec![(FunctionBlock::Block { start: index + 1 }, 0.0)],
                        },
                    )]
                    .into(),
                );
                index += skip + 1;
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

                let len = items.len();
                stack.frames.push(
                    [(
                        (*var).into(),
                        Function {
                            params: vec![],
                            weighted: false,
                            blocks: vec![(FunctionBlock::Value(items.pop().unwrap()), 0.0)],
                        },
                    )]
                    .into(),
                );

                for_stack.push((index, *var, items, Vec::with_capacity(len)));
                index += 1;
            }
            Token::ForEnd => {
                if let Some((start, var, items, values)) = for_stack.last_mut() {
                    values.push(stack.operands.pop().unwrap());

                    if items.len() > 0 {
                        stack.frames.push(
                            [(
                                (*var).into(),
                                Function {
                                    params: vec![],
                                    weighted: false,
                                    blocks: vec![(FunctionBlock::Value(items.pop().unwrap()), 0.0)],
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

                loop_stack.push((index, count, Vec::with_capacity(count as usize)));
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

    Ok(stack.operands.pop().unwrap())
}

pub fn execute(tree: Tree, seed: Option<[u8; 32]>) -> Result<Shape> {
    let seed = match seed {
        Some(seed) => seed,
        None => {
            #[cfg(feature = "std")]
            {
                gen_seed()
            }
            #[cfg(feature = "no-std")]
            return Err(anyhow!("Seed required for rng."));
        }
    };
    let mut rng = ChaCha8Rng::from_seed(seed);

    let mut functions: HashMap<String, Function> = HashMap::new();
    let mut block = Vec::new();
    for definition in tree {
        let start = block.len();
        block.extend(definition.block);
        block.push(Token::Return);

        match functions.get_mut(definition.name) {
            Some(function) => {
                if definition.params != function.params {
                    return Err(anyhow!("Incorrect parameters in duplicate function."));
                }

                function.weighted = true;
                function
                    .blocks
                    .push((FunctionBlock::Block { start }, definition.weight));
            }
            None => {
                functions.insert(
                    definition.name.into(),
                    Function {
                        params: definition.params.iter().map(|s| (*s).into()).collect(),
                        weighted: false,
                        blocks: vec![(FunctionBlock::Block { start }, definition.weight)],
                    },
                );
            }
        }
    }

    let mut stack = Stack {
        frames: vec![functions],
        operands: Vec::new(),
        scope: 0,
    };
    let shape = match reduce_call(&mut stack, &mut rng, "root", 0)? {
        FunctionBlock::Block { start } => {
            match execute_block(&mut stack, &mut rng, &block, start)? {
                Value::Shape(shape) => shape,
                _ => return Err(anyhow!("The `root` function must return a shape.")),
            }
        }
        _ => unreachable!(),
    };
    Ok(Rc::try_unwrap(shape).unwrap().into_inner())
}

#[cfg(feature = "std")]
fn gen_seed() -> [u8; 32] {
    let mut rng = rand::rng();
    let mut seed = [0u8; 32];
    rng.fill(&mut seed);
    seed
}
