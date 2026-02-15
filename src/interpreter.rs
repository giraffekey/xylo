#[cfg(feature = "std")]
use std::rc::Rc;

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, rc::Rc, string::String, vec, vec::Vec};

#[cfg(feature = "io")]
use {image::imageops::FilterType, imageproc::distance_transform::Norm};

#[cfg(not(feature = "io"))]
use crate::parser::{FilterType, Norm};

use crate::error::{Error, Result};
use crate::functions::{builtin_param_count, handle_builtin, BUILTIN_FUNCTIONS};
use crate::out::Config;
use crate::parser::*;
use crate::shape::{Gradient, Shape};

use core::cell::RefCell;
use hashbrown::HashMap;
use noise::Perlin;
use num::Complex;
use rand::distr::{weighted::WeightedIndex, Distribution};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use tiny_skia::{BlendMode, FilterQuality, LineCap, LineJoin, SpreadMode};

#[derive(Debug, Clone, PartialEq)]
pub enum ValueKind {
    Integer,
    Float,
    Complex,
    Boolean,
    Hex,
    Char,
    String,
    Gradient,
    Shape,
    Enum,
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
    Char(char),
    String(String),
    Gradient(Gradient),
    Shape(Rc<RefCell<Shape>>),
    BlendMode(BlendMode),
    LineCap(LineCap),
    LineJoin(LineJoin),
    SpreadMode(SpreadMode),
    FilterQuality(FilterQuality),
    FilterType(FilterType),
    ThresholdType(ThresholdType),
    Norm(Norm),
    SortMode(SortMode),
    SortDirection(SortDirection),
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
            Self::Char(_) => Ok(ValueKind::Char),
            Self::String(_) => Ok(ValueKind::String),
            Self::Gradient(_) => Ok(ValueKind::Gradient),
            Self::Shape(_) => Ok(ValueKind::Shape),
            Self::BlendMode(_)
            | Self::LineCap(_)
            | Self::LineJoin(_)
            | Self::SpreadMode(_)
            | Self::FilterQuality(_)
            | Self::FilterType(_)
            | Self::ThresholdType(_)
            | Self::Norm(_)
            | Self::SortMode(_)
            | Self::SortDirection(_) => Ok(ValueKind::Enum),
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
            (Value::Char(a), Value::Char(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Gradient(a), Value::Gradient(b)) => a == b,
            (Value::Shape(a), Value::Shape(b)) => *a.borrow() == *b.borrow(),
            (Value::BlendMode(a), Value::BlendMode(b)) => a == b,
            (Value::LineCap(a), Value::LineCap(b)) => a == b,
            (Value::LineJoin(a), Value::LineJoin(b)) => a == b,
            (Value::SpreadMode(a), Value::SpreadMode(b)) => a == b,
            (Value::FilterQuality(a), Value::FilterQuality(b)) => a == b,
            (Value::FilterType(a), Value::FilterType(b)) => a == b,
            (Value::ThresholdType(a), Value::ThresholdType(b)) => a == b,
            (Value::Norm(a), Value::Norm(b)) => a == b,
            (Value::SortMode(a), Value::SortMode(b)) => a == b,
            (Value::SortDirection(a), Value::SortDirection(b)) => a == b,
            (Value::Function(a_name, a_argc, a_args), Value::Function(b_name, b_argc, b_args)) => {
                a_name == b_name && a_argc == b_argc && a_args == b_args
            }
            (Value::List(a), Value::List(b)) => a == b,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct Data {
    pub dimensions: (u32, u32),
    pub max_depth: usize,
    pub perlin: Perlin,
}

impl Default for Data {
    fn default() -> Data {
        Data {
            dimensions: (400, 400),
            max_depth: 1500,
            perlin: Perlin::new(0),
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
    scopes: Vec<usize>,
    lets: Vec<bool>,
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
            scopes: Vec::new(),
            lets: vec![false],
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
    base: bool,
    blocks: Vec<(FunctionBlock, f32)>,
}

fn reduce_literal(literal: &Literal) -> Result<Value> {
    match literal {
        Literal::Integer(n) => Ok(Value::Integer(*n)),
        Literal::Float(n) => Ok(Value::Float(*n)),
        Literal::Complex(n) => Ok(Value::Complex(*n)),
        Literal::Boolean(b) => Ok(Value::Boolean(*b)),
        Literal::Hex(hex) => Ok(Value::Hex(*hex)),
        Literal::Char(c) => Ok(Value::Char(*c)),
        Literal::String(s) => Ok(Value::String(s.clone())),
        Literal::Shape(kind) => {
            let shape = match kind {
                ShapeKind::Square => Shape::square(),
                ShapeKind::Circle => Shape::circle(),
                ShapeKind::Triangle => Shape::triangle(),
                ShapeKind::Fill => Shape::fill(),
                ShapeKind::Empty => Shape::empty(),
            };
            Ok(Value::Shape(Rc::new(RefCell::new(shape))))
        }
        Literal::BlendMode(bm) => Ok(Value::BlendMode(*bm)),
        Literal::LineCap(lc) => Ok(Value::LineCap(*lc)),
        Literal::LineJoin(lj) => Ok(Value::LineJoin(*lj)),
        Literal::SpreadMode(sm) => Ok(Value::SpreadMode(*sm)),
        Literal::FilterQuality(fq) => Ok(Value::FilterQuality(*fq)),
        Literal::FilterType(ft) => Ok(Value::FilterType(*ft)),
        Literal::ThresholdType(tt) => Ok(Value::ThresholdType(*tt)),
        Literal::Norm(norm) => Ok(Value::Norm(*norm)),
        Literal::SortMode(sm) => Ok(Value::SortMode(sm.clone())),
        Literal::SortDirection(sd) => Ok(Value::SortDirection(sd.clone())),
    }
}

fn reduce_call(
    stack: &mut Stack,
    rng: &mut ChaCha8Rng,
    data: &Data,
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
                let value = handle_builtin(name, rng, data, &args)?;
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
                                        base: false,
                                        blocks: vec![(FunctionBlock::Value(arg), 0.0)],
                                    },
                                )
                            })
                            .collect();

                        if function.base {
                            stack.scope = stack.frames.len();
                        }

                        stack.frames.push(functions);
                        stack.lets.push(false);

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
        _ => return Err(Error::InvalidMatch),
    }
}

#[derive(Debug)]
enum Operand {
    Value(Option<Value>),
    Function,
}

fn next_operand(
    stack: &mut Stack,
    rng: &mut ChaCha8Rng,
    data: &Data,
    index: &mut usize,
) -> Result<Operand> {
    match stack.operands.pop() {
        Some(Value::Function(name, argc, pre_args)) => {
            let mut args = Vec::with_capacity(pre_args.len() + argc);
            args.extend(pre_args);

            for _ in 0..argc {
                let arg = match next_operand(stack, rng, data, index)? {
                    Operand::Value(value) => value.unwrap(),
                    Operand::Function => {
                        stack.operands.extend(args);
                        return Ok(Operand::Function);
                    }
                };
                args.push(arg);
            }
            args.reverse();

            let function_block = reduce_call(stack, rng, data, &name, args)?;
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

fn start_block<'a>(
    stack: &mut Stack<'a>,
    rng: &mut ChaCha8Rng,
    data: &Data,
    block: &[Token<'a>],
    start: usize,
) -> Result<Value> {
    let mut index = start;

    'a: while index < block.len() {
        if stack.calls.len() > data.max_depth {
            return Err(Error::MaxDepthReached);
        }

        match &block[index] {
            Token::Literal(literal) => {
                stack.operands.push(reduce_literal(literal)?);
                index += 1;
            }
            Token::List(size) => {
                let mut elems = Vec::with_capacity(*size);
                for _ in 0..*size {
                    let elem = match next_operand(stack, rng, data, &mut index)? {
                        Operand::Value(value) => value.unwrap(),
                        Operand::Function => continue 'a,
                    };
                    elems.push(elem);
                }
                elems.reverse();

                let list = match elems.get(0).map(Value::kind) {
                    Some(Ok(ValueKind::Char)) => {
                        let s: Result<String> = elems
                            .iter()
                            .map(|value| match value {
                                Value::Char(c) => Ok(*c),
                                _ => Err(Error::InvalidList),
                            })
                            .collect();

                        Value::String(s?)
                    }
                    _ => {
                        let list = Value::List(elems);
                        list.kind()?;
                        list
                    }
                };

                stack.operands.push(list);
                index += 1;
            }
            Token::UnaryOperator(op) => {
                let arg = match next_operand(stack, rng, data, &mut index)? {
                    Operand::Value(value) => value.unwrap(),
                    Operand::Function => continue 'a,
                };

                let value = handle_builtin(op.as_str(), rng, data, &[arg])?;
                stack.operands.push(value);
                index += 1;
            }
            Token::BinaryOperator(op) => {
                let b = match op {
                    BinaryOperator::Pipe => stack.operands.pop().unwrap(),
                    _ => match next_operand(stack, rng, data, &mut index)? {
                        Operand::Value(value) => value.unwrap(),
                        Operand::Function => continue 'a,
                    },
                };

                let a = match next_operand(stack, rng, data, &mut index)? {
                    Operand::Value(value) => value.unwrap(),
                    Operand::Function => {
                        stack.operands.push(b);
                        continue 'a;
                    }
                };

                let value = handle_builtin(op.as_str(), rng, data, &[a, b])?;
                stack.operands.push(value);
                index += 1;
            }
            Token::Call(name, argc) => {
                let mut args = Vec::with_capacity(*argc);
                for _ in 0..*argc {
                    // let arg = match next_operand(stack, rng, data, &mut index)? {
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

                stack.scopes.push(stack.scope);
                let function_block = reduce_call(stack, rng, data, name, args.clone())?;
                match function_block {
                    FunctionBlock::Value(value) => {
                        stack.operands.push(value);
                        stack.scopes.pop().unwrap();
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
                                let mut lets = Vec::new();
                                let mut values = Vec::new();
                                for value in list.clone() {
                                    let mut args = Vec::with_capacity(pre_args.len() + 1);
                                    args.extend(pre_args.clone());
                                    args.push(value);
                                    args.reverse();

                                    let function_block =
                                        reduce_call(stack, rng, data, &name, args)?;
                                    match function_block {
                                        FunctionBlock::Value(value) => values.push(value),
                                        FunctionBlock::Start(start) => {
                                            frames.push(stack.frames.pop().unwrap());
                                            lets.push(false);
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
                                    stack.lets.extend(lets);
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
                stack.lets.pop().unwrap();
                index += 1;
            }
            Token::Return(start) => {
                let value = match next_operand(stack, rng, data, &mut index)? {
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
                        stack.lets.pop().unwrap();
                        stack.scope = stack.scopes.pop().unwrap();
                        index = last_index;
                    }
                    None => return Ok(value.unwrap()),
                }
            }
            Token::Let(name, params, skip) => {
                let func = Function {
                    params: params.iter().map(|s| (*s).into()).collect(),
                    weighted: false,
                    base: false,
                    blocks: vec![(FunctionBlock::Start(index + 1), 0.0)],
                };
                if *stack.lets.last().unwrap() {
                    stack
                        .frames
                        .last_mut()
                        .unwrap()
                        .insert((*name).into(), func);
                } else {
                    stack.frames.push([((*name).into(), func)].into());
                    stack.lets.push(true);
                }
                index += skip + 1;
            }
            Token::If(skip) => {
                let condition = match next_operand(stack, rng, data, &mut index)? {
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
                let a = match next_operand(stack, rng, data, &mut index)? {
                    Operand::Value(value) => value.unwrap(),
                    Operand::Function => continue 'a,
                };

                let mut found = false;
                'b: for (pattern, has_guard, skip) in patterns {
                    let is_guard = if *has_guard {
                        let condition = match next_operand(stack, rng, data, &mut index)? {
                            Operand::Value(value) => value.unwrap(),
                            Operand::Function => continue 'a,
                        };
                        match condition {
                            Value::Boolean(b) => b,
                            _ => return Err(Error::InvalidCondition),
                        }
                    } else {
                        true
                    };

                    if is_guard {
                        match pattern {
                            Pattern::Matches(matches) => {
                                for b in matches {
                                    if pattern_match(&a, b)? {
                                        found = true;
                                        break 'b;
                                    }
                                }
                                index += skip;
                            }
                            Pattern::Wildcard => {
                                found = true;
                                break 'b;
                            }
                        }
                    } else {
                        index += skip;
                    }
                }

                if !found {
                    return Err(Error::MatchNotFound);
                }

                index += 1;
            }
            Token::ForStart(var) => {
                let iter = match next_operand(stack, rng, data, &mut index)? {
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
                            base: false,
                            blocks: vec![(FunctionBlock::Value(items.pop().unwrap()), 0.0)],
                        },
                    )]
                    .into(),
                );
                stack.lets.push(false);

                stack.fors.push(ForStack {
                    start: index + 1,
                    var: *var,
                    items,
                    values: Vec::with_capacity(len),
                });
                index += 1;
            }
            Token::ForEnd => {
                let value = match next_operand(stack, rng, data, &mut index)? {
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
                                    base: false,
                                    blocks: vec![(
                                        FunctionBlock::Value(for_stack.items.pop().unwrap()),
                                        0.0,
                                    )],
                                },
                            )]
                            .into(),
                        );
                        stack.lets.push(false);
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
                let count = match next_operand(stack, rng, data, &mut index)? {
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
                let value = match next_operand(stack, rng, data, &mut index)? {
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

#[derive(Debug)]
pub struct Env<'a> {
    pub rng: ChaCha8Rng,
    pub data: Data,
    pub functions: HashMap<String, Function>,
    pub block: Vec<Token<'a>>,
}

pub fn load_env(tree: Tree, config: Config) -> Result<Env> {
    let seed = match config.seed {
        Some(seed) => seed,
        None => {
            #[cfg(feature = "std")]
            {
                gen_seed()
            }
            #[cfg(feature = "alloc")]
            return Err(Error::MissingSeed);
        }
    };
    let mut rng = ChaCha8Rng::from_seed(seed);

    let perlin = Perlin::new(rng.random());
    let data = Data {
        dimensions: config.dimensions,
        max_depth: config.max_depth,
        perlin,
    };

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
                        base: true,
                        blocks: vec![(FunctionBlock::Start(start), definition.weight)],
                    },
                );
            }
        }
    }

    Ok(Env {
        rng,
        data,
        functions,
        block,
    })
}

pub fn exec_model(env: &mut Env) -> Result<Option<Value>> {
    let mut stack = Stack::new(env.functions.clone());
    match reduce_call(&mut stack, &mut env.rng, &env.data, "model", Vec::new()) {
        Ok(FunctionBlock::Start(start)) => {
            let model = start_block(&mut stack, &mut env.rng, &env.data, &env.block, start)?;
            Ok(Some(model))
        }
        _ => Ok(None),
    }
}

pub fn exec_update(env: &mut Env, model: Value) -> Result<Option<Value>> {
    let mut stack = Stack::new(env.functions.clone());
    let args = vec![model];
    match reduce_call(&mut stack, &mut env.rng, &env.data, "update", args) {
        Ok(FunctionBlock::Start(start)) => {
            let model = start_block(&mut stack, &mut env.rng, &env.data, &env.block, start)?;
            Ok(Some(model))
        }
        _ => Ok(None),
    }
}

pub fn exec_start(env: &mut Env) -> Result<Option<Rc<RefCell<Shape>>>> {
    let mut stack = Stack::new(env.functions.clone());
    match reduce_call(&mut stack, &mut env.rng, &env.data, "start", Vec::new()) {
        Ok(FunctionBlock::Start(start)) => {
            match start_block(&mut stack, &mut env.rng, &env.data, &env.block, start)? {
                Value::Shape(shape) => Ok(Some(shape)),
                _ => Err(Error::InvalidStart),
            }
        }
        _ => Ok(None),
    }
}

pub fn exec_view(env: &mut Env, model: Value) -> Result<Option<Rc<RefCell<Shape>>>> {
    let mut stack = Stack::new(env.functions.clone());
    let args = vec![model];
    match reduce_call(&mut stack, &mut env.rng, &env.data, "view", args) {
        Ok(FunctionBlock::Start(start)) => {
            match start_block(&mut stack, &mut env.rng, &env.data, &env.block, start)? {
                Value::Shape(shape) => Ok(Some(shape)),
                _ => Err(Error::InvalidView),
            }
        }
        _ => Ok(None),
    }
}

#[cfg(feature = "std")]
fn gen_seed() -> [u8; 32] {
    let mut rng = rand::rng();
    let mut seed = [0u8; 32];
    rng.fill(&mut seed);
    seed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape::{BasicShape, Color, PathSegment, Style, IDENTITY, WHITE};
    use palette::{rgb::Rgba, FromColor, Hsla};
    use tiny_skia::{BlendMode, FillRule, SpreadMode, Stroke, Transform};

    // Helper function to create a test config with a fixed seed
    fn test_config() -> Config {
        Config {
            seed: Some([0; 32]),
            ..Config::default()
        }
    }

    #[test]
    fn test_value_kind() {
        assert_eq!(Value::Integer(42).kind().ok(), Some(ValueKind::Integer));
        assert_eq!(Value::Float(3.14).kind().ok(), Some(ValueKind::Float));
        assert_eq!(
            Value::Complex(Complex::new(1.0, 2.0)).kind().ok(),
            Some(ValueKind::Complex)
        );
        assert_eq!(Value::Boolean(true).kind().ok(), Some(ValueKind::Boolean));
        assert_eq!(Value::Hex([255, 0, 0]).kind().ok(), Some(ValueKind::Hex));
        assert_eq!(Value::Char('a').kind().ok(), Some(ValueKind::Char));
        assert_eq!(
            Value::String("test".into()).kind().ok(),
            Some(ValueKind::String)
        );
        assert_eq!(
            Value::Shape(Rc::new(RefCell::new(Shape::empty())))
                .kind()
                .ok(),
            Some(ValueKind::Shape)
        );
        assert_eq!(
            Value::BlendMode(BlendMode::Multiply).kind().ok(),
            Some(ValueKind::Enum)
        );
        assert_eq!(
            Value::Function("test".into(), 1, vec![]).kind().ok(),
            Some(ValueKind::Function(1))
        );
        assert_eq!(
            Value::List(vec![Value::Integer(1), Value::Integer(2)])
                .kind()
                .ok(),
            Some(ValueKind::List(Box::new(ValueKind::Integer)))
        );
    }

    #[test]
    fn test_reduce_literal() {
        assert_eq!(
            reduce_literal(&Literal::Integer(42)).ok(),
            Some(Value::Integer(42))
        );
        assert_eq!(
            reduce_literal(&Literal::Float(3.14)).ok(),
            Some(Value::Float(3.14))
        );
        assert_eq!(
            reduce_literal(&Literal::Complex(Complex::new(1.0, 2.0))).ok(),
            Some(Value::Complex(Complex::new(1.0, 2.0)))
        );
        assert_eq!(
            reduce_literal(&Literal::Shape(ShapeKind::Square)).ok(),
            Some(Value::Shape(Rc::new(RefCell::new(Shape::square()))))
        );
        assert_eq!(
            reduce_literal(&Literal::BlendMode(BlendMode::Multiply)).ok(),
            Some(Value::BlendMode(BlendMode::Multiply))
        );
    }

    #[test]
    fn test_list_expressions() {
        let mut env = load_env(
            parse("start = sx (last [1, 2, 3]) SQUARE").unwrap(),
            test_config(),
        )
        .unwrap();
        let res = exec_start(&mut env);
        let expected = Shape::Basic(
            BasicShape::Square {
                x: -1.0,
                y: -1.0,
                width: 2.0,
                height: 2.0,
                transform: Transform::from_scale(3.0, 1.0),
                zindex: None,
                color: Color::Solid(WHITE),
                blend_mode: BlendMode::SourceOver,
                anti_alias: true,
                style: Style::Fill(FillRule::Winding),
            },
            None,
            None,
        );
        assert_eq!(res.unwrap(), Some(Rc::new(RefCell::new(expected))));
    }

    #[test]
    fn test_shape_operations() {
        // Basic shape creation
        let mut env = load_env(parse("start = SQUARE").unwrap(), test_config()).unwrap();
        let res = exec_start(&mut env);
        assert_eq!(res.unwrap(), Some(Rc::new(RefCell::new(Shape::square()))));

        // Composition
        let mut env = load_env(
            parse(
                "
start = compose SQUARE CIRCLE
                ",
            )
            .unwrap(),
            test_config(),
        )
        .unwrap();
        let res = exec_start(&mut env);
        assert_eq!(
            res.unwrap(),
            Some(Rc::new(RefCell::new(Shape::composite(
                Rc::new(RefCell::new(Shape::square())),
                Rc::new(RefCell::new(Shape::circle())),
            ))))
        );

        // Styling
        let mut env = load_env(
            parse(
                "
start = stroke 2 (hex 0xff0000 SQUARE)
                ",
            )
            .unwrap(),
            test_config(),
        )
        .unwrap();
        let res = exec_start(&mut env);
        let expected = Shape::Basic(
            BasicShape::Square {
                x: -1.0,
                y: -1.0,
                width: 2.0,
                height: 2.0,
                transform: Transform::default(),
                zindex: None,
                color: Color::Solid(Hsla::from_color(Rgba::new(1.0, 0.0, 0.0, 1.0))),
                blend_mode: BlendMode::SourceOver,
                anti_alias: true,
                style: Style::Stroke(Stroke {
                    width: 2.0,
                    ..Stroke::default()
                }),
            },
            None,
            None,
        );
        assert_eq!(res.unwrap(), Some(Rc::new(RefCell::new(expected))));
    }

    #[test]
    fn test_control_flow() {
        // If expression
        let mut env = load_env(
            parse(
                "
start =
    if true -> SQUARE
    else -> CIRCLE
                ",
            )
            .unwrap(),
            test_config(),
        )
        .unwrap();
        let res = exec_start(&mut env);
        assert_eq!(res.unwrap(), Some(Rc::new(RefCell::new(Shape::square()))));

        // Match expression
        let mut env = load_env(
            parse(
                "
start =
    match 2
        _ if false -> SQUARE
        2 if true -> CIRCLE
        3 -> TRIANGLE
                ",
            )
            .unwrap(),
            test_config(),
        )
        .unwrap();
        let res = exec_start(&mut env);
        assert_eq!(res.unwrap(), Some(Rc::new(RefCell::new(Shape::circle()))));

        // For loop
        let mut env = load_env(
            parse(
                "
start = collect (for i in 1..3 -> SQUARE)
                ",
            )
            .unwrap(),
            test_config(),
        )
        .unwrap();
        let res = exec_start(&mut env);
        assert_eq!(
            res.unwrap(),
            Some(Rc::new(RefCell::new(Shape::collection(vec![
                Rc::new(RefCell::new(Shape::square())),
                Rc::new(RefCell::new(Shape::square())),
            ]))))
        );

        // Loop
        let mut env = load_env(
            parse(
                "
start = collect (loop 3 -> SQUARE)
                ",
            )
            .unwrap(),
            test_config(),
        )
        .unwrap();
        let res = exec_start(&mut env);
        assert_eq!(
            res.unwrap(),
            Some(Rc::new(RefCell::new(Shape::collection(vec![
                Rc::new(RefCell::new(Shape::square())),
                Rc::new(RefCell::new(Shape::square())),
                Rc::new(RefCell::new(Shape::square())),
            ]))))
        );
    }

    #[test]
    fn test_functions() {
        // Simple function
        let mut env = load_env(
            parse(
                "
start = double_scale SQUARE
double_scale shape = ss 2 shape
                ",
            )
            .unwrap(),
            test_config(),
        )
        .unwrap();
        let res = exec_start(&mut env);
        assert_eq!(
            res.unwrap(),
            Some(Rc::new(RefCell::new(Shape::Basic(
                BasicShape::Square {
                    x: -1.0,
                    y: -1.0,
                    width: 2.0,
                    height: 2.0,
                    transform: IDENTITY.post_scale(2.0, 2.0),
                    zindex: None,
                    color: Color::Solid(WHITE),
                    blend_mode: BlendMode::SourceOver,
                    anti_alias: true,
                    style: Style::Fill(FillRule::Winding),
                },
                None,
                None
            )))),
        );
    }

    #[test]
    fn test_complex_shapes() {
        // Path creation
        let mut env = load_env(
            parse(
                r#"
start =
    move_to 0 0
    : line_to 10 0
    : line_to 5 10
    : close
                "#,
            )
            .unwrap(),
            test_config(),
        )
        .unwrap();
        let res = exec_start(&mut env);
        let expected = Shape::path(vec![
            PathSegment::MoveTo(0.0, 0.0),
            PathSegment::LineTo(10.0, 0.0),
            PathSegment::LineTo(5.0, 10.0),
            PathSegment::Close,
        ]);
        assert_eq!(*res.unwrap().unwrap().borrow(), expected);

        // Gradient
        let mut env = load_env(
            parse(
                r#"
start = g grad SQUARE

grad =
    grad_stop_hex 0 RED (
    grad_stop_hex 1 BLUE (
    linear_grad 0 0 1 1))
                "#,
            )
            .unwrap(),
            test_config(),
        )
        .unwrap();
        let res = exec_start(&mut env);
        let expected = Shape::Basic(
            BasicShape::Square {
                x: -1.0,
                y: -1.0,
                width: 2.0,
                height: 2.0,
                transform: IDENTITY,
                zindex: None,
                color: Color::Gradient(Gradient {
                    start: (0.0, 0.0),
                    end: (1.0, 1.0),
                    stops: vec![
                        (0.0, Hsla::from_color(Rgba::new(1.0, 0.0, 0.0, 1.0))),
                        (1.0, Hsla::from_color(Rgba::new(0.0, 0.0, 1.0, 1.0))),
                    ],
                    spread_mode: SpreadMode::Pad,
                    transform: Transform::default(),
                    radius: None,
                }),
                blend_mode: BlendMode::SourceOver,
                anti_alias: true,
                style: Style::Fill(FillRule::Winding),
            },
            None,
            None,
        );
        assert_eq!(*res.unwrap().unwrap().borrow(), expected);
    }

    #[test]
    fn test_randomness() {
        // Verify random produces consistent results with same seed
        let mut env = load_env(parse("start = ss rand SQUARE").unwrap(), test_config()).unwrap();
        let res1 = exec_start(&mut env);
        let mut env = load_env(parse("start = ss rand SQUARE").unwrap(), test_config()).unwrap();
        let res2 = exec_start(&mut env);
        assert_eq!(res1.unwrap(), res2.unwrap());

        // Verify different seeds produce different results
        let mut env = load_env(
            parse("start = ss rand SQUARE").unwrap(),
            Config {
                seed: Some([1; 32]),
                ..test_config()
            },
        )
        .unwrap();
        let res1 = exec_start(&mut env);
        let mut env = load_env(
            parse("start = ss rand SQUARE").unwrap(),
            Config {
                seed: Some([2; 32]),
                ..test_config()
            },
        )
        .unwrap();
        let res2 = exec_start(&mut env);
        assert_ne!(res1.unwrap(), res2.unwrap());
    }
}
