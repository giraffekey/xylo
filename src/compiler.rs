#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, collections::BTreeMap, format, string::String, vec, vec::Vec};

#[cfg(feature = "std")]
use std::collections::BTreeMap;

use crate::functions::{handle_builtin, BUILTIN_FUNCTIONS};
use crate::parser::*;

use anyhow::{anyhow, Result};
use palette::{Hsla, RgbHue};
use tiny_skia::Transform;

pub static IDENTITY: Transform = Transform {
    sx: 1.0,
    kx: 0.0,
    ky: 0.0,
    sy: 1.0,
    tx: 0.0,
    ty: 0.0,
};

pub static WHITE: Hsla<f32> = Hsla::new_const(RgbHue::new(360.0), 0.0, 1.0, 1.0);

pub static TRANSPARENT: Hsla<f32> = Hsla::new_const(RgbHue::new(360.0), 0.0, 1.0, 0.0);

pub static SQUARE: Shape = Shape::Square {
    x: -1.0,
    y: -1.0,
    width: 2.0,
    height: 2.0,
    transform: IDENTITY,
    color: WHITE,
};

pub static CIRCLE: Shape = Shape::Circle {
    x: 0.0,
    y: 0.0,
    radius: 1.0,
    transform: IDENTITY,
    color: WHITE,
};

pub static TRIANGLE: Shape = Shape::Triangle {
    points: [-1.0, 0.577350269, 1.0, 0.577350269, 0.0, -1.154700538],
    transform: IDENTITY,
    color: WHITE,
};

pub static FILL: Shape = Shape::Fill { color: WHITE };

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shape {
    Square {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        transform: Transform,
        color: Hsla<f32>,
    },
    Circle {
        x: f32,
        y: f32,
        radius: f32,
        transform: Transform,
        color: Hsla<f32>,
    },
    Triangle {
        points: [f32; 6],
        transform: Transform,
        color: Hsla<f32>,
    },
    Fill {
        color: Hsla<f32>,
    },
    Composite {
        shapes: &'static [Shape],
        transform: Transform,
        color: Hsla<f32>,
    },
}

impl Shape {
    pub fn translate(self, tx: f32, ty: f32) -> Self {
        match self {
            Self::Square {
                x,
                y,
                width,
                height,
                transform,
                color,
            } => Self::Square {
                x,
                y,
                width,
                height,
                transform: transform.post_translate(tx, ty),
                color,
            },
            Self::Circle {
                x,
                y,
                radius,
                transform,
                color,
            } => Self::Circle {
                x,
                y,
                radius,
                transform: transform.post_translate(tx, ty),
                color,
            },
            Self::Triangle {
                points,
                transform,
                color,
            } => Self::Triangle {
                points,
                transform: transform.post_translate(tx, ty),
                color,
            },
            Self::Composite {
                shapes,
                transform,
                color,
            } => Self::Composite {
                shapes,
                transform: transform.post_translate(tx, ty),
                color,
            },
            Self::Fill { color } => Self::Fill { color },
        }
    }

    pub fn rotate(self, r: f32) -> Self {
        match self {
            Self::Square {
                x,
                y,
                width,
                height,
                transform,
                color,
            } => Self::Square {
                x,
                y,
                width,
                height,
                transform: transform.post_rotate(r),
                color,
            },
            Self::Circle {
                x,
                y,
                radius,
                transform,
                color,
            } => Self::Circle {
                x,
                y,
                radius,
                transform: transform.post_rotate(r),
                color,
            },
            Self::Triangle {
                points,
                transform,
                color,
            } => Self::Triangle {
                points,
                transform: transform.post_rotate(r),
                color,
            },
            Self::Composite {
                shapes,
                transform,
                color,
            } => Self::Composite {
                shapes,
                transform: transform.post_rotate(r),
                color,
            },
            Self::Fill { color } => Self::Fill { color },
        }
    }

    pub fn rotate_at(self, r: f32, tx: f32, ty: f32) -> Self {
        match self {
            Self::Square {
                x,
                y,
                width,
                height,
                transform,
                color,
            } => Self::Square {
                x,
                y,
                width,
                height,
                transform: transform.post_rotate_at(r, tx, ty),
                color,
            },
            Self::Circle {
                x,
                y,
                radius,
                transform,
                color,
            } => Self::Circle {
                x,
                y,
                radius,
                transform: transform.post_rotate_at(r, tx, ty),
                color,
            },
            Self::Triangle {
                points,
                transform,
                color,
            } => Self::Triangle {
                points,
                transform: transform.post_rotate_at(r, tx, ty),
                color,
            },
            Self::Composite {
                shapes,
                transform,
                color,
            } => Self::Composite {
                shapes,
                transform: transform.post_rotate_at(r, tx, ty),
                color,
            },
            Self::Fill { color } => Self::Fill { color },
        }
    }

    pub fn scale(self, sx: f32, sy: f32) -> Self {
        match self {
            Self::Square {
                x,
                y,
                width,
                height,
                transform,
                color,
            } => Self::Square {
                x,
                y,
                width,
                height,
                transform: transform.post_scale(sx, sy),
                color,
            },
            Self::Circle {
                x,
                y,
                radius,
                transform,
                color,
            } => Self::Circle {
                x,
                y,
                radius,
                transform: transform.post_scale(sx, sy),
                color,
            },
            Self::Triangle {
                points,
                transform,
                color,
            } => Self::Triangle {
                points,
                transform: transform.post_scale(sx, sy),
                color,
            },
            Self::Composite {
                shapes,
                transform,
                color,
            } => Self::Composite {
                shapes,
                transform: transform.post_scale(sx, sy),
                color,
            },
            Self::Fill { color } => Self::Fill { color },
        }
    }

    pub fn skew(self, kx: f32, ky: f32) -> Self {
        match self {
            Self::Square {
                x,
                y,
                width,
                height,
                transform,
                color,
            } => Self::Square {
                x,
                y,
                width,
                height,
                transform: transform.post_concat(Transform::from_skew(kx, ky)),
                color,
            },
            Self::Circle {
                x,
                y,
                radius,
                transform,
                color,
            } => Self::Circle {
                x,
                y,
                radius,
                transform: transform.post_concat(Transform::from_skew(kx, ky)),
                color,
            },
            Self::Triangle {
                points,
                transform,
                color,
            } => Self::Triangle {
                points,
                transform: transform.post_concat(Transform::from_skew(kx, ky)),
                color,
            },
            Self::Composite {
                shapes,
                transform,
                color,
            } => Self::Composite {
                shapes,
                transform: transform.post_concat(Transform::from_skew(kx, ky)),
                color,
            },
            Self::Fill { color } => Self::Fill { color },
        }
    }

    pub fn flip(self, f: f32) -> Self {
        todo!()
        // match self {
        //     Self::Square {
        //         x,
        //         y,
        //         width,
        //         height,
        //         transform,
        //         color,
        //     } => Self::Square {
        //         x,
        //         y,
        //         width,
        //         height,
        //         transform: flip(transform, f),
        //         color,
        //     },
        //     Self::Circle {
        //         x,
        //         y,
        //         radius,
        //         transform,
        //         color,
        //     } => Self::Circle {
        //         x,
        //         y,
        //         radius,
        //         transform: flip(transform, f),
        //         color,
        //     },
        //     Self::Triangle {
        //         points,
        //         transform,
        //         color,
        //     } => Self::Triangle {
        //         points,
        //         transform: flip(transform, f),
        //         color,
        //     },
        //     Self::Composite {
        //         shapes,
        //         transform,
        //         color,
        //     } => Self::Composite {
        //         shapes,
        //         transform: flip(transform, f),
        //         color,
        //     },
        //     Self::Fill { color } => Self::Fill { color },
        // }
    }

    pub fn hue(self, hue: f32) -> Self {
        match self {
            Self::Square {
                x,
                y,
                width,
                height,
                transform,
                color,
            } => Self::Square {
                x,
                y,
                width,
                height,
                transform,
                color: set_hue(color, hue),
            },
            Self::Circle {
                x,
                y,
                radius,
                transform,
                color,
            } => Self::Circle {
                x,
                y,
                radius,
                transform,
                color: set_hue(color, hue),
            },
            Self::Triangle {
                points,
                transform,
                color,
            } => Self::Triangle {
                points,
                transform,
                color: set_hue(color, hue),
            },
            Self::Composite {
                shapes,
                transform,
                color,
            } => Self::Composite {
                shapes,
                transform,
                color: set_hue(color, hue),
            },
            Self::Fill { color } => Self::Fill {
                color: set_hue(color, hue),
            },
        }
    }

    pub fn saturation(self, saturation: f32) -> Self {
        match self {
            Self::Square {
                x,
                y,
                width,
                height,
                transform,
                color,
            } => Self::Square {
                x,
                y,
                width,
                height,
                transform,
                color: set_saturation(color, saturation),
            },
            Self::Circle {
                x,
                y,
                radius,
                transform,
                color,
            } => Self::Circle {
                x,
                y,
                radius,
                transform,
                color: set_saturation(color, saturation),
            },
            Self::Triangle {
                points,
                transform,
                color,
            } => Self::Triangle {
                points,
                transform,
                color: set_saturation(color, saturation),
            },
            Self::Composite {
                shapes,
                transform,
                color,
            } => Self::Composite {
                shapes,
                transform,
                color: set_saturation(color, saturation),
            },
            Self::Fill { color } => Self::Fill {
                color: set_saturation(color, saturation),
            },
        }
    }

    pub fn lightness(self, lightness: f32) -> Self {
        match self {
            Self::Square {
                x,
                y,
                width,
                height,
                transform,
                color,
            } => Self::Square {
                x,
                y,
                width,
                height,
                transform,
                color: set_lightness(color, lightness),
            },
            Self::Circle {
                x,
                y,
                radius,
                transform,
                color,
            } => Self::Circle {
                x,
                y,
                radius,
                transform,
                color: set_lightness(color, lightness),
            },
            Self::Triangle {
                points,
                transform,
                color,
            } => Self::Triangle {
                points,
                transform,
                color: set_lightness(color, lightness),
            },
            Self::Composite {
                shapes,
                transform,
                color,
            } => Self::Composite {
                shapes,
                transform,
                color: set_lightness(color, lightness),
            },
            Self::Fill { color } => Self::Fill {
                color: set_lightness(color, lightness),
            },
        }
    }

    pub fn alpha(self, alpha: f32) -> Self {
        match self {
            Self::Square {
                x,
                y,
                width,
                height,
                transform,
                color,
            } => Self::Square {
                x,
                y,
                width,
                height,
                transform,
                color: set_alpha(color, alpha),
            },
            Self::Circle {
                x,
                y,
                radius,
                transform,
                color,
            } => Self::Circle {
                x,
                y,
                radius,
                transform,
                color: set_alpha(color, alpha),
            },
            Self::Triangle {
                points,
                transform,
                color,
            } => Self::Triangle {
                points,
                transform,
                color: set_alpha(color, alpha),
            },
            Self::Composite {
                shapes,
                transform,
                color,
            } => Self::Composite {
                shapes,
                transform,
                color: set_alpha(color, alpha),
            },
            Self::Fill { color } => Self::Fill {
                color: set_alpha(color, alpha),
            },
        }
    }
}

fn set_hue(mut color: Hsla<f32>, hue: f32) -> Hsla<f32> {
    color.hue = hue.into();
    color
}

fn set_saturation(mut color: Hsla<f32>, saturation: f32) -> Hsla<f32> {
    color.saturation = saturation;
    color
}

fn set_lightness(mut color: Hsla<f32>, lightness: f32) -> Hsla<f32> {
    color.lightness = lightness;
    color
}

fn set_alpha(mut color: Hsla<f32>, alpha: f32) -> Hsla<f32> {
    color.alpha = alpha;
    color
}

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
    Shape(Shape),
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

fn reduce_literal(literal: Literal) -> Result<Value> {
    match literal {
        Literal::Integer(n) => Ok(Value::Integer(n)),
        Literal::Float(n) => Ok(Value::Float(n)),
        Literal::Boolean(b) => Ok(Value::Boolean(b)),
        Literal::Shape(kind) => Ok(Value::Shape(match kind {
            ShapeKind::Square => SQUARE,
            ShapeKind::Circle => CIRCLE,
            ShapeKind::Triangle => TRIANGLE,
            ShapeKind::Fill => FILL,
        })),
    }
}

fn reduce_binary_operation(stack: Stack, operation: BinaryOperation) -> Result<Value> {
    let a = reduce_expr(stack.clone(), *operation.a)?;
    let b = reduce_expr(stack, *operation.b)?;
    handle_builtin(operation.op.as_str(), &[a, b])
}

fn reduce_call<'a>(mut stack: Stack<'a>, call: Call<'a>) -> Result<Value> {
    if BUILTIN_FUNCTIONS.contains(&call.name) {
        let args: Result<Vec<Value>> = call
            .args
            .iter()
            .map(|arg| reduce_expr(stack.clone(), arg.clone()))
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
                    stack.depth += 1;
                    for (param, arg) in function.params.clone().iter().zip(call.args) {
                        let arg = reduce_expr(stack.clone(), arg)?;
                        stack.functions.insert(
                            param,
                            Function {
                                params: vec![],
                                block: Block::Value(arg),
                            },
                        );
                    }

                    reduce_expr(stack, expr)
                }
            }
        }
        None => Err(anyhow!(format!("Function `{}` not found", call.name))),
    }
}

fn reduce_let<'a>(mut stack: Stack<'a>, let_statement: Let<'a>) -> Result<Value> {
    for definition in let_statement.definitions {
        stack.functions.insert(
            definition.name,
            Function {
                params: definition.params,
                block: Block::Expr(definition.block),
            },
        );
    }

    reduce_expr(stack, *let_statement.block)
}

fn reduce_if(stack: Stack, if_statement: If) -> Result<Value> {
    let condition = reduce_expr(stack.clone(), *if_statement.condition)?;
    let is_true = match condition {
        Value::Boolean(b) => b,
        _ => return Err(anyhow!("If condition must reduce to a boolean.")),
    };
    if is_true {
        reduce_expr(stack, *if_statement.if_block)
    } else {
        reduce_expr(stack, *if_statement.else_block)
    }
}

fn reduce_match(stack: Stack, match_statement: Match) -> Result<Value> {
    todo!()
}

fn reduce_for(stack: Stack, for_statement: For) -> Result<Value> {
    let iter = reduce_expr(stack.clone(), *for_statement.iter)?;
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

    let mut list = Vec::new();
    for item in items {
        let mut stack = stack.clone();
        stack.functions.insert(
            for_statement.var,
            Function {
                params: vec![],
                block: Block::Value(item),
            },
        );
        list.push(reduce_expr(stack, *for_statement.block.clone())?);
    }

    let list = Value::List(list);
    list.kind()?;
    Ok(list)
}

fn reduce_loop(stack: Stack, loop_statement: Loop) -> Result<Value> {
    let count = reduce_expr(stack.clone(), *loop_statement.count)?;
    let count = match count {
        Value::Integer(n) => n,
        Value::Float(n) => n as i32,
        _ => return Err(anyhow!("Value must be a number.")),
    };
    if count < 0 {
        return Err(anyhow!("Cannot iterate over negative number."));
    }

    let mut list = Vec::new();
    for _ in 0..count {
        list.push(reduce_expr(stack.clone(), *loop_statement.block.clone())?);
    }

    let list = Value::List(list);
    list.kind()?;
    Ok(list)
}

fn reduce_expr(stack: Stack, expr: Expr) -> Result<Value> {
    match expr {
        Expr::Literal(literal) => reduce_literal(literal),
        Expr::BinaryOperation(operation) => reduce_binary_operation(stack, operation),
        Expr::Call(call) => reduce_call(stack, call),
        Expr::Let(let_statement) => reduce_let(stack, let_statement),
        Expr::If(if_statement) => reduce_if(stack, if_statement),
        Expr::Match(match_statement) => reduce_match(stack, match_statement),
        Expr::For(for_statement) => reduce_for(stack, for_statement),
        Expr::Loop(loop_statement) => reduce_loop(stack, loop_statement),
    }
}

pub fn compile(tree: Tree) -> Result<Shape> {
    let mut stack = Stack {
        functions: BTreeMap::new(),
        depth: 0,
    };
    for definition in tree {
        stack.functions.insert(
            definition.name,
            Function {
                params: definition.params,
                block: Block::Expr(definition.block),
            },
        );
    }
    match stack.functions.get("root") {
        Some(root_fn) => {
            let value = match root_fn.block.clone() {
                Block::Value(value) => value,
                Block::Expr(expr) => reduce_expr(stack, expr)?,
            };
            match value {
                Value::Shape(shape) => Ok(shape),
                _ => Err(anyhow!("The `root` function must return a shape.")),
            }
        }
        None => Err(anyhow!("Could not find `root` function definition.")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::f32::consts::PI;

    #[test]
    fn test() {
        let (_, tree) = parse(
            "
root =
	let shape = SQUARE
		s (pi * 25) 50 (add_circle shape)

add_circle shape = shape : CIRCLE
    		",
        )
        .unwrap();
        let shape = compile(tree).unwrap();
        assert_eq!(
            shape,
            Shape::Composite {
                shapes: vec![SQUARE, CIRCLE].leak(),
                transform: Transform::from_scale(PI * 25.0, 50.0),
                color: TRANSPARENT,
            }
        );
    }
}
