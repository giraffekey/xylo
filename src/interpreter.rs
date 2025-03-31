#[cfg(feature = "std")]
use std::rc::Rc;

#[cfg(feature = "no-std")]
use alloc::{boxed::Box, rc::Rc, string::String, vec, vec::Vec};

use crate::error::{Error, Result};
use crate::functions::{builtin_param_count, handle_builtin, BUILTIN_FUNCTIONS};
use crate::parser::*;
use crate::shape::{Shape, CIRCLE, EMPTY, FILL, SQUARE, TRIANGLE};

use core::cell::RefCell;
use hashbrown::HashMap;
use num::Complex;
use rand::distr::{weighted::WeightedIndex, Distribution};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use tiny_skia::BlendMode;

#[derive(Debug, Clone, PartialEq)]
pub enum ValueKind {
    Integer,
    Float,
    Complex,
    Boolean,
    Hex,
    Shape,
    BlendMode,
    Function(usize),
    List(Box<ValueKind>),
    Unknown,
}

#[derive(Debug, Clone)]
pub enum Value {
    Integer(i32),
    Float(f32),
    Complex(Complex<f32>),
    Boolean(bool),
    Hex([u8; 3]),
    Shape(Rc<RefCell<Shape>>),
    BlendMode(BlendMode),
    Function(String, usize, Vec<Value>),
    List(Vec<Value>),
}

impl Value {
    pub fn kind(&self) -> Result<ValueKind> {
        match self {
            Self::Integer(_) => Ok(ValueKind::Integer),
            Self::Float(_) => Ok(ValueKind::Float),
            Self::Complex(_) => Ok(ValueKind::Complex),
            Self::Boolean(_) => Ok(ValueKind::Boolean),
            Self::Hex(_) => Ok(ValueKind::Hex),
            Self::Shape(_) => Ok(ValueKind::Shape),
            Self::BlendMode(_) => Ok(ValueKind::BlendMode),
            Self::Function(_, argc, _) => Ok(ValueKind::Function(*argc)),
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
                    Err(Error::InvalidList)
                }
            }
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Integer(a), Value::Float(b)) | (Value::Float(b), Value::Integer(a)) => {
                (*a as f32) == *b
            }
            (Value::Complex(a), Value::Complex(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Hex(a), Value::Hex(b)) => a == b,
            (Value::Shape(a), Value::Shape(b)) => *a.borrow() == *b.borrow(),
            (Value::BlendMode(a), Value::BlendMode(b)) => a == b,
            (Value::Function(a_name, a_argc, a_args), Value::Function(b_name, b_argc, b_args)) => {
                a_name == b_name && a_argc == b_argc && a_args == b_args
            }
            (Value::List(a), Value::List(b)) => a == b,
            _ => false,
        }
    }
}

type Frame = HashMap<String, Function>;

#[derive(Debug)]
struct ForStack<'a> {
    start: usize,
    var: &'a str,
    items: Vec<Value>,
    values: Vec<Value>,
}

#[derive(Debug)]
struct LoopStack {
    start: usize,
    remaining: usize,
    values: Vec<Value>,
}

#[derive(Debug, Clone)]
enum HigherOrder {
    Map(usize, usize, Vec<Value>),
}

#[derive(Debug)]
struct Stack<'a> {
    frames: Vec<Frame>,
    operands: Vec<Value>,
    scope: usize,
    calls: Vec<usize>,
    fors: Vec<ForStack<'a>>,
    loops: Vec<LoopStack>,
    higher_order: Option<HigherOrder>,
}

impl Stack<'_> {
    pub fn new(frame: Frame) -> Self {
        Self {
            frames: vec![frame],
            operands: Vec::new(),
            scope: 0,
            calls: Vec::new(),
            fors: Vec::new(),
            loops: Vec::new(),
            higher_order: None,
        }
    }

    pub fn get_function(&self, name: &str) -> Option<Function> {
        match self.frames[0].get(name) {
            Some(function) => Some(function.clone()),
            None => self.frames[self.scope..]
                .iter()
                .rev()
                .find_map(|functions| functions.get(name))
                .cloned(),
        }
    }
}

#[derive(Debug, Clone)]
enum FunctionBlock {
    Value(Value),
    Start(usize),
    HigherOrder,
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
        Literal::Complex(n) => Ok(Value::Complex(*n)),
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
        Literal::BlendMode(b) => Ok(Value::BlendMode(*b)),
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
    mut args: Vec<Value>,
) -> Result<FunctionBlock> {
    if BUILTIN_FUNCTIONS.contains(&name) {
        let param_count = builtin_param_count(name);
        if args.len() > param_count {
            stack.operands.extend(args.drain(param_count..));
        } else if args.len() < param_count {
            let argc = param_count - args.len();
            return Ok(FunctionBlock::Value(Value::Function(
                name.into(),
                argc,
                args,
            )));
        }

        match name {
            "map" => return Ok(FunctionBlock::HigherOrder),
            _ => {
                let value = handle_builtin(name, rng, &args)?;
                Ok(FunctionBlock::Value(value))
            }
        }
    } else {
        match stack.get_function(name) {
            Some(function) => {
                if args.len() > function.params.len() {
                    stack.operands.extend(args.drain(function.params.len()..));
                } else if args.len() < function.params.len() {
                    let argc = function.params.len() - args.len();
                    return Ok(FunctionBlock::Value(Value::Function(
                        name.into(),
                        argc,
                        args,
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
                    FunctionBlock::Start(start) => {
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

                        Ok(FunctionBlock::Start(*start))
                    }
                    _ => unreachable!(),
                }
            }
            None => Err(Error::UnknownFunction(name.into())),
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
        _ => return Err(Error::InvalidMatch),
    }
}

#[derive(Debug)]
enum Operand {
    Value(Option<Value>),
    Function,
}

fn next_operand(stack: &mut Stack, rng: &mut ChaCha8Rng, index: &mut usize) -> Result<Operand> {
    match stack.operands.pop() {
        Some(Value::Function(name, argc, pre_args)) => {
            let mut args = Vec::with_capacity(pre_args.len() + argc);
            args.extend(pre_args);

            for _ in 0..argc {
                let arg = match next_operand(stack, rng, index)? {
                    Operand::Value(value) => value.unwrap(),
                    Operand::Function => {
                        stack.operands.extend(args);
                        return Ok(Operand::Function);
                    }
                };
                args.push(arg);
            }
            args.reverse();

            let function_block = reduce_call(stack, rng, &name, args)?;
            match function_block {
                FunctionBlock::Value(value) => Ok(Operand::Value(Some(value))),
                FunctionBlock::Start(start) => {
                    stack.calls.push(*index);
                    *index = start;
                    Ok(Operand::Function)
                }
                FunctionBlock::HigherOrder => todo!(),
            }
        }
        value => Ok(Operand::Value(value)),
    }
}

fn execute_block<'a>(
    stack: &mut Stack<'a>,
    rng: &mut ChaCha8Rng,
    block: &[Token<'a>],
    start: usize,
) -> Result<Value> {
    let mut index = start;

    'a: while index < block.len() {
        match &block[index] {
            Token::Literal(literal) => {
                stack.operands.push(reduce_literal(literal)?);
                index += 1;
            }
            Token::UnaryOperator(op) => {
                let arg = match next_operand(stack, rng, &mut index)? {
                    Operand::Value(value) => value.unwrap(),
                    Operand::Function => continue 'a,
                };

                let value = handle_builtin(op.as_str(), rng, &[arg])?;
                stack.operands.push(value);
                index += 1;
            }
            Token::BinaryOperator(op) => {
                let b = match op {
                    BinaryOperator::Pipe => stack.operands.pop().unwrap(),
                    _ => match next_operand(stack, rng, &mut index)? {
                        Operand::Value(value) => value.unwrap(),
                        Operand::Function => continue 'a,
                    },
                };

                let a = match next_operand(stack, rng, &mut index)? {
                    Operand::Value(value) => value.unwrap(),
                    Operand::Function => {
                        stack.operands.push(b);
                        continue 'a;
                    }
                };

                let value = handle_builtin(op.as_str(), rng, &[a, b])?;
                stack.operands.push(value);
                index += 1;
            }
            Token::Call(name, argc) => {
                let mut args = Vec::with_capacity(*argc);
                for _ in 0..*argc {
                    // let arg = match next_operand(stack, rng, &mut index)? {
                    //     Operand::Value(value) => value.unwrap(),
                    //     Operand::Function => {
                    //         stack.operands.extend(args);
                    //         continue 'a;
                    //     }
                    // };
                    // args.push(arg);
                    args.push(stack.operands.pop().unwrap());
                }
                args.reverse();

                let function_block = reduce_call(stack, rng, name, args.clone())?;
                match function_block {
                    FunctionBlock::Value(value) => {
                        stack.operands.push(value);
                        index += 1;
                    }
                    FunctionBlock::Start(start) => {
                        stack.calls.push(index + 1);
                        index = start;
                    }
                    FunctionBlock::HigherOrder => match *name {
                        "map" => match (&args[0], &args[1]) {
                            (Value::Function(name, _argc, pre_args), Value::List(list)) => {
                                index += 1;

                                let mut frames = Vec::new();
                                let mut values = Vec::new();
                                for value in list.clone() {
                                    let mut args = Vec::with_capacity(pre_args.len() + 1);
                                    args.extend(pre_args.clone());
                                    args.push(value);
                                    args.reverse();

                                    let function_block = reduce_call(stack, rng, &name, args)?;
                                    match function_block {
                                        FunctionBlock::Value(value) => values.push(value),
                                        FunctionBlock::Start(start) => {
                                            frames.push(stack.frames.pop().unwrap());
                                            stack.calls.push(index);
                                            index = start;
                                        }
                                        FunctionBlock::HigherOrder => todo!(),
                                    }
                                }

                                if frames.len() == 0 {
                                    let list = Value::List(values);
                                    list.kind()?;
                                    stack.operands.push(list);
                                } else {
                                    stack.higher_order = Some(HigherOrder::Map(
                                        index,
                                        frames.len() - values.len(),
                                        values,
                                    ));
                                    frames.reverse();
                                    stack.frames.extend(frames);
                                }
                            }
                            _ => return Err(Error::UnknownFunction("map".into())),
                        },
                        _ => unreachable!(),
                    },
                }
            }
            Token::Jump(skip) => index += skip + 1,
            Token::Pop => {
                stack.frames.pop().unwrap();
                index += 1;
            }
            Token::Return(start) => {
                let value = match next_operand(stack, rng, &mut index)? {
                    Operand::Value(value) => value,
                    Operand::Function => continue 'a,
                };

                match stack.calls.pop() {
                    Some(last_index) => {
                        match &mut stack.higher_order {
                            Some(higher_order) => match higher_order {
                                HigherOrder::Map(other_start, count, values)
                                    if *start == Some(*other_start) =>
                                {
                                    *count -= 1;
                                    values.push(value.unwrap());

                                    if *count == 0 {
                                        let list = Value::List(values.clone());
                                        list.kind()?;
                                        stack.operands.push(list);
                                        stack.higher_order = None;
                                    }
                                }
                                _ => (),
                            },
                            _ => {
                                stack.operands.push(value.unwrap());
                            }
                        }

                        stack.frames.pop().unwrap();
                        stack.scope = stack.frames.len() - 1;
                        index = last_index;
                    }
                    None => return Ok(value.unwrap()),
                }
            }
            Token::Let(name, params, skip) => {
                stack.frames.push(
                    [(
                        (*name).into(),
                        Function {
                            params: params.iter().map(|s| (*s).into()).collect(),
                            weighted: false,
                            blocks: vec![(FunctionBlock::Start(index + 1), 0.0)],
                        },
                    )]
                    .into(),
                );
                index += skip + 1;
            }
            Token::If(skip) => {
                let condition = match next_operand(stack, rng, &mut index)? {
                    Operand::Value(value) => value.unwrap(),
                    Operand::Function => continue 'a,
                };
                let is_true = match condition {
                    Value::Boolean(b) => b,
                    _ => return Err(Error::InvalidCondition),
                };

                if is_true {
                    index += 1;
                } else {
                    index += skip + 1;
                }
            }
            Token::Match(patterns) => {
                let a = match next_operand(stack, rng, &mut index)? {
                    Operand::Value(value) => value.unwrap(),
                    Operand::Function => continue 'a,
                };

                let mut found = false;
                'b: for (pattern, skip) in patterns {
                    match pattern {
                        Pattern::Matches(matches) => {
                            for b in matches {
                                if pattern_match(&a, b)? {
                                    found = true;
                                    break 'b;
                                }
                                index += skip;
                            }
                        }
                        Pattern::Wildcard => {
                            found = true;
                            break 'b;
                        }
                    }
                }

                if !found {
                    return Err(Error::MatchNotFound);
                }

                index += 1;
            }
            Token::ForStart(var) => {
                let iter = match next_operand(stack, rng, &mut index)? {
                    Operand::Value(value) => value.unwrap(),
                    Operand::Function => continue 'a,
                };
                let mut items = match iter {
                    Value::List(list) => list,
                    Value::Integer(n) => {
                        if n < 0 {
                            return Err(Error::NotIterable);
                        }
                        (0..n).map(Value::Integer).collect()
                    }
                    Value::Float(n) => {
                        if n < 0.0 {
                            return Err(Error::NotIterable);
                        }
                        (0..n as i32).map(Value::Integer).collect()
                    }
                    _ => return Err(Error::NotIterable),
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

                stack.fors.push(ForStack {
                    start: index + 1,
                    var: *var,
                    items,
                    values: Vec::with_capacity(len),
                });
                index += 1;
            }
            Token::ForEnd => {
                let value = match next_operand(stack, rng, &mut index)? {
                    Operand::Value(value) => value,
                    Operand::Function => continue 'a,
                };
                if let Some(for_stack) = stack.fors.last_mut() {
                    for_stack.values.push(value.unwrap());

                    if for_stack.items.len() > 0 {
                        stack.frames.push(
                            [(
                                for_stack.var.into(),
                                Function {
                                    params: vec![],
                                    weighted: false,
                                    blocks: vec![(
                                        FunctionBlock::Value(for_stack.items.pop().unwrap()),
                                        0.0,
                                    )],
                                },
                            )]
                            .into(),
                        );
                        index = for_stack.start; // Jump back to ForStart
                    } else {
                        let list = Value::List(for_stack.values.to_vec());
                        list.kind()?;
                        stack.operands.push(list);

                        stack.fors.pop().unwrap(); // Exit for loop
                        index += 1;
                    }
                } else {
                    if let Some(value) = value {
                        stack.operands.push(value);
                    }
                }
            }
            Token::LoopStart => {
                let count = match next_operand(stack, rng, &mut index)? {
                    Operand::Value(value) => value.unwrap(),
                    Operand::Function => continue 'a,
                };
                let count = match count {
                    Value::Integer(n) => n,
                    Value::Float(n) => n as i32,
                    _ => return Err(Error::NotIterable),
                };
                if count < 0 {
                    return Err(Error::NotIterable);
                }

                stack.loops.push(LoopStack {
                    start: index + 1,
                    remaining: count as usize,
                    values: Vec::with_capacity(count as usize),
                });
                index += 1;
            }
            Token::LoopEnd => {
                let value = match next_operand(stack, rng, &mut index)? {
                    Operand::Value(value) => value,
                    Operand::Function => continue 'a,
                };
                if let Some(loop_stack) = stack.loops.last_mut() {
                    loop_stack.remaining -= 1;
                    loop_stack.values.push(value.unwrap());

                    if loop_stack.remaining > 0 {
                        index = loop_stack.start; // Jump back to LoopStart
                    } else {
                        let list = Value::List(loop_stack.values.to_vec());
                        list.kind()?;
                        stack.operands.push(list);

                        stack.loops.pop().unwrap(); // Exit loop
                        index += 1;
                    }
                } else {
                    if let Some(value) = value {
                        stack.operands.push(value);
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
            return Err(Error::MissingSeed);
        }
    };
    let mut rng = ChaCha8Rng::from_seed(seed);

    let mut functions: HashMap<String, Function> = HashMap::new();
    let mut block = Vec::new();
    for definition in tree {
        let start = block.len();
        block.extend(definition.block);
        block.push(Token::Return(Some(start)));

        match functions.get_mut(definition.name) {
            Some(function) => {
                if definition.params != function.params {
                    return Err(Error::InvalidDefinition(definition.name.into()));
                }

                function.weighted = true;
                function
                    .blocks
                    .push((FunctionBlock::Start(start), definition.weight));
            }
            None => {
                functions.insert(
                    definition.name.into(),
                    Function {
                        params: definition.params.iter().map(|s| (*s).into()).collect(),
                        weighted: false,
                        blocks: vec![(FunctionBlock::Start(start), definition.weight)],
                    },
                );
            }
        }
    }

    let mut stack = Stack::new(functions);
    let shape = match reduce_call(&mut stack, &mut rng, "root", Vec::new())? {
        FunctionBlock::Start(start) => match execute_block(&mut stack, &mut rng, &block, start)? {
            Value::Shape(shape) => shape,
            _ => return Err(Error::InvalidRoot),
        },
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
