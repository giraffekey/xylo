#[cfg(feature = "std")]
use std::rc::Rc;

#[cfg(feature = "no-std")]
use alloc::{boxed::Box, rc::Rc, string::String, vec, vec::Vec};

use crate::error::{Error, Result};
use crate::functions::{builtin_param_count, handle_builtin, BUILTIN_FUNCTIONS};
use crate::out::Config;
use crate::parser::*;
use crate::shape::{Gradient, Shape, CIRCLE, EMPTY, FILL, SQUARE, TRIANGLE};

use core::cell::RefCell;
use hashbrown::HashMap;
use noise::Perlin;
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
    Gradient,
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
    Gradient(Gradient),
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
            Self::Gradient(_) => Ok(ValueKind::Gradient),
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
            (Value::Gradient(a), Value::Gradient(b)) => a == b,
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

#[derive(Debug)]
pub struct Data {
    pub dimensions: (u32, u32),
    pub perlin: Perlin,
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
        Literal::Shape(kind) => {
            let shape = match kind {
                ShapeKind::Square => SQUARE.clone(),
                ShapeKind::Circle => CIRCLE.clone(),
                ShapeKind::Triangle => TRIANGLE.clone(),
                ShapeKind::Fill => FILL.clone(),
                ShapeKind::Empty => EMPTY.clone(),
            };
            Ok(Value::Shape(Rc::new(RefCell::new(Shape::Basic(shape)))))
        }
        Literal::BlendMode(b) => Ok(Value::BlendMode(*b)),
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

fn execute_block<'a>(
    stack: &mut Stack<'a>,
    rng: &mut ChaCha8Rng,
    data: &Data,
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
            Token::List(blocks) => {
                let values: Result<Vec<_>> = blocks
                    .iter()
                    .map(|block| execute_block(stack, rng, data, block, 0))
                    .collect();
                stack.operands.push(Value::List(values?));
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
                'b: for (pattern, skip) in patterns {
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

pub fn execute(tree: Tree, config: Config) -> Result<Shape> {
    let seed = match config.seed {
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

    let perlin = Perlin::new(rng.random());
    let data = Data {
        dimensions: config.dimensions,
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

    let mut stack = Stack::new(functions);
    let shape = match reduce_call(&mut stack, &mut rng, &data, "root", Vec::new())? {
        FunctionBlock::Start(start) => {
            match execute_block(&mut stack, &mut rng, &data, &block, start)? {
                Value::Shape(shape) => shape,
                _ => return Err(Error::InvalidRoot),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape::{BasicShape, Color, ColorChange, HslaChange, IDENTITY, WHITE};
    use tiny_skia::Transform;

    #[test]
    fn test_binary_operation() {
        let config = Config {
            dimensions: (400, 400),
            seed: Some([0; 32]),
        };
        let res = execute(
            parse(
                "
root = square

square = ss (3 + 5) SQUARE
                ",
            )
            .unwrap(),
            config,
        );
        assert!(res.is_ok());
        assert_eq!(
            res.unwrap(),
            Shape::Basic(BasicShape::Square {
                x: -1.0,
                y: -1.0,
                width: 2.0,
                height: 2.0,
                transform: Transform::from_scale(8.0, 8.0),
                zindex: None,
                color: Color::Solid(WHITE),
                blend_mode: BlendMode::SourceOver,
                anti_alias: true,
            })
        );
    }

    #[test]
    fn test_let_statement() {
        let config = Config {
            dimensions: (400, 400),
            seed: Some([0; 32]),
        };
        let res = execute(
            parse(
                "
root = square

square =
    let n1 = 3
        n2 = 5
        n3 = 2
        ss (n1 * n2 * n3) SQUARE
                ",
            )
            .unwrap(),
            config,
        );
        assert!(res.is_ok());
        assert_eq!(
            res.unwrap(),
            Shape::Basic(BasicShape::Square {
                x: -1.0,
                y: -1.0,
                width: 2.0,
                height: 2.0,
                transform: Transform::from_scale(30.0, 30.0),
                zindex: None,
                color: Color::Solid(WHITE),
                blend_mode: BlendMode::SourceOver,
                anti_alias: true,
            })
        );
    }

    #[test]
    fn test_if_statement() {
        let config = Config {
            dimensions: (400, 400),
            seed: Some([0; 32]),
        };
        let res = execute(
            parse(
                "
root = shape true : shape false

shape is_square =
    if is_square
        SQUARE
    else
        EMPTY
                ",
            )
            .unwrap(),
            config,
        );
        assert!(res.is_ok());
        assert_eq!(
            res.unwrap(),
            Shape::Composite {
                a: Rc::new(RefCell::new(Shape::Basic(SQUARE.clone()))),
                b: Rc::new(RefCell::new(Shape::Basic(EMPTY.clone()))),
                transform: IDENTITY,
                zindex_overwrite: None,
                zindex_shift: None,
                color_overwrite: ColorChange::default(),
                color_shift: HslaChange::default(),
                blend_mode_overwrite: None,
                anti_alias_overwrite: None,
            }
        );
    }

    #[test]
    fn test_match_statement() {
        let config = Config {
            dimensions: (400, 400),
            seed: Some([0; 32]),
        };
        let res = execute(
            parse(
                "
root = shape 1 : shape 2 : shape 3 : shape 4

shape n =
    match n
        1 -> SQUARE
        2 -> CIRCLE
        3 -> TRIANGLE
        _ -> EMPTY
                ",
            )
            .unwrap(),
            config,
        );
        assert!(res.is_ok());
        assert_eq!(
            res.unwrap(),
            Shape::Composite {
                a: Rc::new(RefCell::new(Shape::Composite {
                    a: Rc::new(RefCell::new(Shape::Composite {
                        a: Rc::new(RefCell::new(Shape::Basic(SQUARE.clone()))),
                        b: Rc::new(RefCell::new(Shape::Basic(CIRCLE.clone()))),
                        transform: IDENTITY,
                        zindex_overwrite: None,
                        zindex_shift: None,
                        color_overwrite: ColorChange::default(),
                        color_shift: HslaChange::default(),
                        blend_mode_overwrite: None,
                        anti_alias_overwrite: None,
                    })),
                    b: Rc::new(RefCell::new(Shape::Basic(TRIANGLE.clone()))),
                    transform: IDENTITY,
                    zindex_overwrite: None,
                    zindex_shift: None,
                    color_overwrite: ColorChange::default(),
                    color_shift: HslaChange::default(),
                    blend_mode_overwrite: None,
                    anti_alias_overwrite: None,
                })),
                b: Rc::new(RefCell::new(Shape::Basic(EMPTY.clone()))),
                transform: IDENTITY,
                zindex_overwrite: None,
                zindex_shift: None,
                color_overwrite: ColorChange::default(),
                color_shift: HslaChange::default(),
                blend_mode_overwrite: None,
                anti_alias_overwrite: None,
            }
        );
    }

    #[test]
    fn test_for_statement() {
        let config = Config {
            dimensions: (400, 400),
            seed: Some([0; 32]),
        };
        let res = execute(
            parse(
                "
root = collect shapes

shapes =
    for i in 0..10
        if i % 2 == 0
            SQUARE
        else
            CIRCLE
                ",
            )
            .unwrap(),
            config,
        );
        assert!(res.is_ok());
        assert_eq!(
            res.unwrap(),
            Shape::Collection {
                shapes: vec![
                    Rc::new(RefCell::new(Shape::Basic(SQUARE.clone()))),
                    Rc::new(RefCell::new(Shape::Basic(CIRCLE.clone()))),
                    Rc::new(RefCell::new(Shape::Basic(SQUARE.clone()))),
                    Rc::new(RefCell::new(Shape::Basic(CIRCLE.clone()))),
                    Rc::new(RefCell::new(Shape::Basic(SQUARE.clone()))),
                    Rc::new(RefCell::new(Shape::Basic(CIRCLE.clone()))),
                    Rc::new(RefCell::new(Shape::Basic(SQUARE.clone()))),
                    Rc::new(RefCell::new(Shape::Basic(CIRCLE.clone()))),
                    Rc::new(RefCell::new(Shape::Basic(SQUARE.clone()))),
                    Rc::new(RefCell::new(Shape::Basic(CIRCLE.clone()))),
                ],
                transform: IDENTITY,
                zindex_overwrite: None,
                zindex_shift: None,
                color_overwrite: ColorChange::default(),
                color_shift: HslaChange::default(),
                blend_mode_overwrite: None,
                anti_alias_overwrite: None,
            }
        );
    }

    #[test]
    fn test_loop_statement() {
        let config = Config {
            dimensions: (400, 400),
            seed: Some([0; 32]),
        };
        let res = execute(
            parse(
                "
root = collect shapes

shapes =
    loop 3
        ss (rand * 100) SQUARE
                ",
            )
            .unwrap(),
            config,
        );
        assert!(res.is_ok());
        assert_eq!(
            res.unwrap(),
            Shape::Collection {
                shapes: vec![
                    Rc::new(RefCell::new(Shape::Basic(BasicShape::Square {
                        x: -1.0,
                        y: -1.0,
                        width: 2.0,
                        height: 2.0,
                        transform: Transform::from_scale(83.69197, 83.69197),
                        zindex: None,
                        color: Color::Solid(WHITE),
                        blend_mode: BlendMode::SourceOver,
                        anti_alias: true,
                    }))),
                    Rc::new(RefCell::new(Shape::Basic(BasicShape::Square {
                        x: -1.0,
                        y: -1.0,
                        width: 2.0,
                        height: 2.0,
                        transform: Transform::from_scale(90.9063, 90.9063),
                        zindex: None,
                        color: Color::Solid(WHITE),
                        blend_mode: BlendMode::SourceOver,
                        anti_alias: true,
                    }))),
                    Rc::new(RefCell::new(Shape::Basic(BasicShape::Square {
                        x: -1.0,
                        y: -1.0,
                        width: 2.0,
                        height: 2.0,
                        transform: Transform::from_scale(63.14245, 63.14245),
                        zindex: None,
                        color: Color::Solid(WHITE),
                        blend_mode: BlendMode::SourceOver,
                        anti_alias: true,
                    }))),
                ],
                transform: IDENTITY,
                zindex_overwrite: None,
                zindex_shift: None,
                color_overwrite: ColorChange::default(),
                color_shift: HslaChange::default(),
                blend_mode_overwrite: None,
                anti_alias_overwrite: None,
            }
        );
    }
}
