#[cfg(feature = "std")]
use std::sync::{Arc, Mutex};

#[cfg(feature = "no-std")]
use {
    alloc::{sync::Arc, vec, vec::Vec},
    spin::Mutex,
};

use crate::interpreter::Value;
use crate::shape::{lock_shape, PathSegment, Shape, IDENTITY, TRANSPARENT, WHITE};

use anyhow::{anyhow, Result};
use core::f32::consts::PI;

macro_rules! define_builtins {
    (
        $(
            $name:literal => $func:ident
        ),* $(,)?
    ) => {
        // Generate the static BUILTIN_FUNCTIONS array
        pub const BUILTIN_FUNCTIONS: &[&str] = &[
            $($name),*
        ];

        // Generate the handle_builtin function with match statements
        pub fn handle_builtin(name: &str, args: &[Value]) -> Result<Value> {
            match name {
                $(
                    $name => $func(args),
                )*
                _ => Err(anyhow!("Unknown function: {}", name)),
            }
        }
    };
}

define_builtins! {
    "pi" => pi,
    "π" => pi,
    "sin" => sin,
    "cos" => cos,
    "tan" => tan,
    "asin" => asin,
    "acos" => acos,
    "atan" => atan,
    "atan2" => atan2,
    "sinh" => sinh,
    "cosh" => cosh,
    "tanh" => tanh,
    "asinh" => asinh,
    "acosh" => acosh,
    "atanh" => atanh,
    "ln" => ln,
    "log10" => log10,
    "log" => log,
    "floor" => floor,
    "ceil" => ceil,
    "abs" => abs,
    "sqrt" => sqrt,
    "min" => min,
    "max" => max,
    "+" => add,
    "add" => add,
    "-" => sub,
    "sub" => sub,
    "*" => mul,
    "mul" => mul,
    "/" => div,
    "div" => div,
    "%" => modulo,
    "mod" => modulo,
    "**" => pow,
    "pow" => pow,
    "==" => eq,
    "eq" => eq,
    "!=" => neq,
    "neq" => neq,
    "<" => lt,
    "lt" => lt,
    "<=" => lte,
    "lte" => lte,
    ">" => gt,
    "gt" => gt,
    ">=" => gte,
    "gte" => gte,
    "&&" => and,
    "and" => and,
    "||" => or,
    "or" => or,
    ":" => compose,
    "compose" => compose,
    "collect" => collect,
    ".." => range,
    "range" => range,
    "..=" => rangei,
    "rangei" => rangei,
    "t" => translate,
    "translate" => translate,
    "tx" => translatex,
    "translatex" => translatex,
    "ty" => translatey,
    "translatey" => translatey,
    "tt" => translateb,
    "translateb" => translateb,
    // "z" => zindex,
    // "zindex" => zindex,
    "r" => rotate,
    "rotate" => rotate,
    "ra" => rotate_at,
    "rotate_at" => rotate_at,
    "s" => scale,
    "scale" => scale,
    "sx" => scalex,
    "scalex" => scalex,
    "sy" => scaley,
    "scaley" => scaley,
    "ss" => scaleb,
    "scaleb" => scaleb,
    "k" => skew,
    "skew" => skew,
    "kx" => skewx,
    "skewx" => skewx,
    "ky" => skewy,
    "skewy" => skewy,
    "kk" => skewb,
    "skewb" => skewb,
    "f" => flip,
    "flip" => flip,
    "fh" => fliph,
    "fliph" => fliph,
    "fv" => flipv,
    "flipv" => flipv,
    "fd" => flipd,
    "flipd" => flipd,
    "hsl" => hsl,
    "hsla" => hsla,
    "h" => hue,
    "hue" => hue,
    "sat" => saturation,
    "saturation" => saturation,
    "l" => lightness,
    "lightness" => lightness,
    "a" => alpha,
    "alpha" => alpha,
    "hshift" => hshift,
    "sshift" => sshift,
    "lshift" => lshift,
    "ashift" => ashift,
    "hex" => hex,
    // "blend" => blend,
    "move_to" => move_to,
    "line_to" => line_to,
    "quad_to" => quad_to,
    "cubic_to" => cubic_to,
    "close" => close,
}

pub fn pi(args: &[Value]) -> Result<Value> {
    if args.len() != 0 {
        return Err(anyhow!("Invalid number of arguments to `pi` function."));
    }

    Ok(Value::Float(PI))
}

pub fn sin(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `sin` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).sin())),
        Value::Float(n) => Ok(Value::Float(n.sin())),
        _ => Err(anyhow!("Invalid type passed to `sin` function.")),
    }
}

pub fn cos(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `cos` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).cos())),
        Value::Float(n) => Ok(Value::Float(n.cos())),
        _ => Err(anyhow!("Invalid type passed to `cos` function.")),
    }
}

pub fn tan(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `tan` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).tan())),
        Value::Float(n) => Ok(Value::Float(n.tan())),
        _ => Err(anyhow!("Invalid type passed to `tan` function.")),
    }
}

pub fn asin(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `asin` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).asin())),
        Value::Float(n) => Ok(Value::Float(n.asin())),
        _ => Err(anyhow!("Invalid type passed to `asin` function.")),
    }
}

pub fn acos(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `acos` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).acos())),
        Value::Float(n) => Ok(Value::Float(n.acos())),
        _ => Err(anyhow!("Invalid type passed to `acos` function.")),
    }
}

pub fn atan(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `atan` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).atan())),
        Value::Float(n) => Ok(Value::Float(n.atan())),
        _ => Err(anyhow!("Invalid type passed to `atan` function.")),
    }
}

pub fn atan2(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `atan2` function."));
    }

    let y = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `atan2` function.")),
    };

    let x = match args[1] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `atan2` function.")),
    };

    Ok(Value::Float(y.atan2(x)))
}

pub fn sinh(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `sinh` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).sinh())),
        Value::Float(n) => Ok(Value::Float(n.sinh())),
        _ => Err(anyhow!("Invalid type passed to `sinh` function.")),
    }
}

pub fn cosh(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `cosh` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).cosh())),
        Value::Float(n) => Ok(Value::Float(n.cosh())),
        _ => Err(anyhow!("Invalid type passed to `cosh` function.")),
    }
}

pub fn tanh(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `tanh` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).tanh())),
        Value::Float(n) => Ok(Value::Float(n.tanh())),
        _ => Err(anyhow!("Invalid type passed to `tanh` function.")),
    }
}

pub fn asinh(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `asinh` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).asinh())),
        Value::Float(n) => Ok(Value::Float(n.asinh())),
        _ => Err(anyhow!("Invalid type passed to `asinh` function.")),
    }
}

pub fn acosh(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `acosh` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).acosh())),
        Value::Float(n) => Ok(Value::Float(n.acosh())),
        _ => Err(anyhow!("Invalid type passed to `acosh` function.")),
    }
}

pub fn atanh(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `atanh` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).atanh())),
        Value::Float(n) => Ok(Value::Float(n.atanh())),
        _ => Err(anyhow!("Invalid type passed to `atanh` function.")),
    }
}

pub fn ln(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `ln` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).ln())),
        Value::Float(n) => Ok(Value::Float(n.ln())),
        _ => Err(anyhow!("Invalid type passed to `ln` function.")),
    }
}

pub fn log10(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `log10` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).log10())),
        Value::Float(n) => Ok(Value::Float(n.log10())),
        _ => Err(anyhow!("Invalid type passed to `log10` function.")),
    }
}

pub fn log(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `log` function."));
    }

    let n = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `log` function.")),
    };

    let b = match args[0] {
        Value::Integer(b) => b as f32,
        Value::Float(b) => b,
        _ => return Err(anyhow!("Invalid type passed to `log` function.")),
    };

    Ok(Value::Float(n.log(b)))
}

pub fn abs(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `abs` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Integer(n.abs())),
        Value::Float(n) => Ok(Value::Float(n.abs())),
        _ => Err(anyhow!("Invalid type passed to `abs` function.")),
    }
}

pub fn floor(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `floor` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Integer(n)),
        Value::Float(n) => Ok(Value::Integer(n.floor() as i32)),
        _ => Err(anyhow!("Invalid type passed to `floor` function.")),
    }
}

pub fn ceil(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `ceil` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Integer(n)),
        Value::Float(n) => Ok(Value::Integer(n.ceil() as i32)),
        _ => Err(anyhow!("Invalid type passed to `ceil` function.")),
    }
}

pub fn sqrt(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `sqrt` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).sqrt())),
        Value::Float(n) => Ok(Value::Float(n.sqrt())),
        _ => Err(anyhow!("Invalid type passed to `sqrt` function.")),
    }
}

pub fn min(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `min` function."));
    }

    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(*a.min(b))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.min(*b))),
        (Value::Integer(a), Value::Float(b)) | (Value::Float(b), Value::Integer(a)) => {
            Ok(Value::Float((*a as f32).min(*b)))
        }
        _ => Err(anyhow!("Invalid type passed to `min` function.")),
    }
}

pub fn max(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `max` function."));
    }

    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(*a.max(b))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.max(*b))),
        (Value::Integer(a), Value::Float(b)) | (Value::Float(b), Value::Integer(a)) => {
            Ok(Value::Float((*a as f32).max(*b)))
        }
        _ => Err(anyhow!("Invalid type passed to `max` function.")),
    }
}

pub fn add(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `add` function."));
    }

    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
        (Value::Integer(a), Value::Float(b)) | (Value::Float(b), Value::Integer(a)) => {
            Ok(Value::Float(*a as f32 + b))
        }
        _ => Err(anyhow!("Invalid type passed to `add` function.")),
    }
}

pub fn sub(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `sub` function."));
    }

    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f32 - b)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a - *b as f32)),
        _ => Err(anyhow!("Invalid type passed to `sub` function.")),
    }
}

pub fn mul(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `mul` function."));
    }

    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
        (Value::Integer(a), Value::Float(b)) | (Value::Float(b), Value::Integer(a)) => {
            Ok(Value::Float(*a as f32 * b))
        }
        _ => Err(anyhow!("Invalid type passed to `mul` function.")),
    }
}

pub fn div(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `div` function."));
    }

    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a / b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f32 / b)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a / *b as f32)),
        _ => Err(anyhow!("Invalid type passed to `div` function.")),
    }
}

pub fn modulo(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `mod` function."));
    }

    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a % b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a % b)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f32 % b)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a % *b as f32)),
        _ => Err(anyhow!("Invalid type passed to `mod` function.")),
    }
}

pub fn pow(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `pow` function."));
    }

    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Float((*a as f32).powf(*b as f32))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.powf(*b))),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Float((*a as f32).powf(*b))),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a.powf(*b as f32))),
        _ => Err(anyhow!("Invalid type passed to `pow` function.")),
    }
}

pub fn eq(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `eq` function."));
    }

    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a == b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a == b)),
        (Value::Integer(a), Value::Float(b)) | (Value::Float(b), Value::Integer(a)) => {
            Ok(Value::Boolean(*a as f32 == *b))
        }
        (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(a == b)),
        (Value::Shape(_a), Value::Shape(_b)) => todo!(),
        _ => Err(anyhow!("Invalid type passed to `eq` function.")),
    }
}

pub fn neq(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `neq` function."));
    }

    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a != b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a != b)),
        (Value::Integer(a), Value::Float(b)) | (Value::Float(b), Value::Integer(a)) => {
            Ok(Value::Boolean(*a as f32 != *b))
        }
        (Value::Shape(_a), Value::Shape(_b)) => todo!(),
        _ => Err(anyhow!("Invalid type passed to `neq` function.")),
    }
}

pub fn lt(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `lt` function."));
    }

    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a < b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a < b)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Boolean((*a as f32) < *b)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Boolean(*a < *b as f32)),
        _ => Err(anyhow!("Invalid type passed to `lt` function.")),
    }
}

pub fn lte(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `lte` function."));
    }

    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a <= b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a <= b)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Boolean((*a as f32) <= *b)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Boolean(*a <= *b as f32)),
        _ => Err(anyhow!("Invalid type passed to `lte` function.")),
    }
}

pub fn gt(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `gt` function."));
    }

    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a > b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a > b)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Boolean((*a as f32) > *b)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Boolean(*a > *b as f32)),
        _ => Err(anyhow!("Invalid type passed to `gt` function.")),
    }
}

pub fn gte(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `gte` function."));
    }

    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a >= b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a >= b)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Boolean((*a as f32) >= *b)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Boolean(*a >= *b as f32)),
        _ => Err(anyhow!("Invalid type passed to `gte` function.")),
    }
}

pub fn and(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `and` function."));
    }

    match (&args[0], &args[1]) {
        (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(*a && *b)),
        _ => Err(anyhow!("Invalid type passed to `and` function.")),
    }
}

pub fn or(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `or` function."));
    }

    match (&args[0], &args[1]) {
        (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(*a || *b)),
        _ => Err(anyhow!("Invalid type passed to `or` function.")),
    }
}

pub fn compose(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!(
            "Invalid number of arguments to `compose` function."
        ));
    }

    match (&args[0], &args[1]) {
        (Value::Shape(a), Value::Shape(b)) => {
            let shape = match (&*lock_shape(a), &*lock_shape(b)) {
                (
                    Shape::Path {
                        segments: a,
                        transform: a_transform,
                        color,
                    },
                    Shape::Path {
                        segments: b,
                        transform: b_transform,
                        ..
                    },
                ) => {
                    let mut segments = Vec::with_capacity(a.len() + b.len());
                    segments.extend(a);
                    segments.extend(b);
                    Shape::Path {
                        segments,
                        transform: a_transform.post_concat(*b_transform),
                        color: *color,
                    }
                }
                _ => Shape::Composite {
                    a: a.clone(),
                    b: b.clone(),
                    transform: IDENTITY,
                    color: TRANSPARENT,
                },
            };
            Ok(Value::Shape(Arc::new(Mutex::new(shape))))
        }
        _ => Err(anyhow!("Invalid type passed to `compose` function.")),
    }
}

pub fn collect(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!(
            "Invalid number of arguments to `collect` function."
        ));
    }

    match &args[0] {
        Value::List(list) => {
            let shapes: Result<Vec<Arc<Mutex<Shape>>>> = list
                .iter()
                .map(|item| match item {
                    Value::Shape(shape) => Ok(shape.clone()),
                    _ => Err(anyhow!("Invalid type passed to `collect` function.")),
                })
                .collect();
            let shapes = shapes?;

            if shapes.len() < 1 {
                return Err(anyhow!("Cannot collect zero shapes."));
            }

            let is_path = shapes.iter().all(|shape| match *lock_shape(shape) {
                Shape::Path { .. } => true,
                _ => false,
            });
            let shape = if is_path {
                let mut segments = Vec::with_capacity(shapes.len());
                let mut transform = IDENTITY;
                let color = WHITE;

                for path in shapes {
                    match &*lock_shape(&path) {
                        Shape::Path {
                            segments: other_segments,
                            transform: other_transform,
                            ..
                        } => {
                            segments.extend(other_segments);
                            transform = transform.post_concat(*other_transform);
                        }
                        _ => unreachable!(),
                    }
                }

                Shape::Path {
                    segments,
                    transform,
                    color,
                }
            } else {
                Shape::Collection {
                    shapes,
                    transform: IDENTITY,
                    color: TRANSPARENT,
                }
            };
            Ok(Value::Shape(Arc::new(Mutex::new(shape))))
        }
        _ => Err(anyhow!("Invalid type passed to `collect` function.")),
    }
}

pub fn range(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `range` function."));
    }

    match (&args[0], &args[1]) {
        (Value::Integer(from), Value::Integer(to)) => {
            Ok(Value::List((*from..*to).map(Value::Integer).collect()))
        }
        (Value::Float(from), Value::Float(to)) => Ok(Value::List(
            (*from as i32..*to as i32)
                .map(|i| Value::Float(i as f32))
                .collect(),
        )),
        _ => Err(anyhow!("Invalid type passed to `range` function.")),
    }
}

pub fn rangei(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `rangei` function."));
    }

    match (&args[0], &args[1]) {
        (Value::Integer(from), Value::Integer(to)) => {
            Ok(Value::List((*from..=*to).map(Value::Integer).collect()))
        }
        (Value::Float(from), Value::Float(to)) => Ok(Value::List(
            (*from as i32..=*to as i32)
                .map(|i| Value::Float(i as f32))
                .collect(),
        )),
        _ => Err(anyhow!("Invalid type passed to `rangei` function.")),
    }
}

pub fn translate(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(anyhow!(
            "Invalid number of arguments to `translate` function."
        ));
    }

    let tx = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `translate` function.")),
    };

    let ty = match args[1] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `translate` function.")),
    };

    let shape = match &args[2] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `translate` function.")),
    };

    lock_shape(&shape).translate(tx, ty);

    Ok(Value::Shape(shape))
}

pub fn translatex(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!(
            "Invalid number of arguments to `translatex` function."
        ));
    }

    let tx = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `translatex` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `translatex` function.")),
    };

    lock_shape(&shape).translate(tx, 0.0);
    Ok(Value::Shape(shape))
}

pub fn translatey(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!(
            "Invalid number of arguments to `translatey` function."
        ));
    }

    let ty = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `translatey` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `translatey` function.")),
    };

    lock_shape(&shape).translate(0.0, ty);
    Ok(Value::Shape(shape))
}

pub fn translateb(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!(
            "Invalid number of arguments to `translateb` function."
        ));
    }

    let t = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `translateb` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `translateb` function.")),
    };

    lock_shape(&shape).translate(t, t);
    Ok(Value::Shape(shape))
}

pub fn rotate(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `rotate` function."));
    }

    let r = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `rotate` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `rotate` function.")),
    };

    lock_shape(&shape).rotate(r);
    Ok(Value::Shape(shape))
}

pub fn rotate_at(args: &[Value]) -> Result<Value> {
    if args.len() != 4 {
        return Err(anyhow!(
            "Invalid number of arguments to `rotate_at` function."
        ));
    }

    let r = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `rotate_at` function.")),
    };

    let tx = match args[1] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `rotate_at` function.")),
    };

    let ty = match args[2] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `rotate_at` function.")),
    };

    let shape = match &args[3] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `rotate_at` function.")),
    };

    lock_shape(&shape).rotate_at(r, tx, ty);
    Ok(Value::Shape(shape))
}

pub fn scale(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(anyhow!("Invalid number of arguments to `scale` function."));
    }

    let sx = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `scale` function.")),
    };

    let sy = match args[1] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `scale` function.")),
    };

    let shape = match &args[2] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `scale` function.")),
    };

    lock_shape(&shape).scale(sx, sy);
    Ok(Value::Shape(shape))
}

pub fn scalex(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `scalex` function."));
    }

    let sx = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `scalex` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `scalex` function.")),
    };

    lock_shape(&shape).scale(sx, 1.0);
    Ok(Value::Shape(shape))
}

pub fn scaley(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `scaley` function."));
    }

    let sy = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `scaley` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `scaley` function.")),
    };

    lock_shape(&shape).scale(1.0, sy);
    Ok(Value::Shape(shape))
}

pub fn scaleb(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `scaleb` function."));
    }

    let s = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `scaleb` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `scaleb` function.")),
    };

    lock_shape(&shape).scale(s, s);
    Ok(Value::Shape(shape))
}

pub fn skew(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(anyhow!("Invalid number of arguments to `skew` function."));
    }

    let kx = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `skew` function.")),
    };

    let ky = match args[1] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `skew` function.")),
    };

    let shape = match &args[2] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `skew` function.")),
    };

    lock_shape(&shape).skew(kx, ky);
    Ok(Value::Shape(shape))
}

pub fn skewx(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `skewx` function."));
    }

    let kx = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `skewx` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `skewx` function.")),
    };

    lock_shape(&shape).skew(kx, 0.0);
    Ok(Value::Shape(shape))
}

pub fn skewy(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `skewy` function."));
    }

    let ky = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `skewy` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `skewy` function.")),
    };

    lock_shape(&shape).skew(0.0, ky);
    Ok(Value::Shape(shape))
}

pub fn skewb(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `skewb` function."));
    }

    let k = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `skewb` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `skewb` function.")),
    };

    lock_shape(&shape).skew(k, k);
    Ok(Value::Shape(shape))
}

pub fn flip(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `flip` function."));
    }

    let f = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `flip` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `flip` function.")),
    };

    lock_shape(&shape).flip(f);
    Ok(Value::Shape(shape))
}

pub fn fliph(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `fliph` function."));
    }

    let shape = match &args[0] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `fliph` function.")),
    };

    lock_shape(&shape).fliph();
    Ok(Value::Shape(shape))
}

pub fn flipv(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `flipv` function."));
    }

    let shape = match &args[0] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `flipv` function.")),
    };

    lock_shape(&shape).flipv();
    Ok(Value::Shape(shape))
}

pub fn flipd(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `flipd` function."));
    }

    let shape = match &args[0] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `flipd` function.")),
    };

    lock_shape(&shape).flipd();
    Ok(Value::Shape(shape))
}

pub fn hsl(args: &[Value]) -> Result<Value> {
    if args.len() != 4 {
        return Err(anyhow!("Invalid number of arguments to `hsl` function."));
    }

    let h = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `hsl` function.")),
    };

    let s = match args[1] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `hsl` function.")),
    };

    let l = match args[2] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `hsl` function.")),
    };

    let shape = match &args[3] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `hsl` function.")),
    };

    lock_shape(&shape).set_hsl(h, s, l);
    Ok(Value::Shape(shape))
}

pub fn hsla(args: &[Value]) -> Result<Value> {
    if args.len() != 5 {
        return Err(anyhow!("Invalid number of arguments to `hsla` function."));
    }

    let h = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `hsla` function.")),
    };

    let s = match args[1] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `hsla` function.")),
    };

    let l = match args[2] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `hsla` function.")),
    };

    let a = match args[3] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `hsla` function.")),
    };

    let shape = match &args[4] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `hsla` function.")),
    };

    lock_shape(&shape).set_hsla(h, s, l, a);
    Ok(Value::Shape(shape))
}

pub fn hue(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `hue` function."));
    }

    let h = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `hue` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `hue` function.")),
    };

    lock_shape(&shape).set_hue(h);
    Ok(Value::Shape(shape))
}

pub fn saturation(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!(
            "Invalid number of arguments to `saturation` function."
        ));
    }

    let s = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `saturation` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `saturation` function.")),
    };

    lock_shape(&shape).set_saturation(s);
    Ok(Value::Shape(shape))
}

pub fn lightness(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!(
            "Invalid number of arguments to `lightness` function."
        ));
    }

    let l = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `lightness` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `lightness` function.")),
    };

    lock_shape(&shape).set_lightness(l);
    Ok(Value::Shape(shape))
}

pub fn alpha(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `alpha` function."));
    }

    let a = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `alpha` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `alpha` function.")),
    };

    lock_shape(&shape).set_alpha(a);
    Ok(Value::Shape(shape))
}

pub fn hshift(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `hshift` function."));
    }

    let h = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `hshift` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `hshift` function.")),
    };

    lock_shape(&shape).shift_hue(h);
    Ok(Value::Shape(shape))
}

pub fn sshift(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `sshift` function."));
    }

    let s = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `sshift` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `sshift` function.")),
    };

    lock_shape(&shape).shift_saturation(s);
    Ok(Value::Shape(shape))
}

pub fn lshift(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `lshift` function."));
    }

    let l = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `lshift` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `lshift` function.")),
    };

    lock_shape(&shape).shift_lightness(l);
    Ok(Value::Shape(shape))
}

pub fn ashift(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `ashift` function."));
    }

    let a = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `ashift` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `ashift` function.")),
    };

    lock_shape(&shape).shift_alpha(a);
    Ok(Value::Shape(shape))
}

pub fn hex(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `hex` function."));
    }

    let hex = match args[0] {
        Value::Hex(hex) => hex,
        _ => return Err(anyhow!("Invalid type passed to `hex` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `hex` function.")),
    };

    lock_shape(&shape).set_hex(hex);
    Ok(Value::Shape(shape))
}

pub fn move_to(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!(
            "Invalid number of arguments to `move_to` function."
        ));
    }

    let x = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `move_to` function.")),
    };

    let y = match args[1] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `move_to` function.")),
    };

    let segments = vec![PathSegment::MoveTo(x, y)];
    let shape = Shape::Path {
        segments,
        transform: IDENTITY,
        color: WHITE,
    };
    Ok(Value::Shape(Arc::new(Mutex::new(shape))))
}

pub fn line_to(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!(
            "Invalid number of arguments to `line_to` function."
        ));
    }

    let x = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `line_to` function.")),
    };

    let y = match args[1] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `line_to` function.")),
    };

    let segments = vec![PathSegment::LineTo(x, y)];
    let shape = Shape::Path {
        segments,
        transform: IDENTITY,
        color: WHITE,
    };
    Ok(Value::Shape(Arc::new(Mutex::new(shape))))
}

pub fn quad_to(args: &[Value]) -> Result<Value> {
    if args.len() != 4 {
        return Err(anyhow!(
            "Invalid number of arguments to `quad_to` function."
        ));
    }

    let x1 = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `quad_to` function.")),
    };

    let y1 = match args[1] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `quad_to` function.")),
    };

    let x = match args[2] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `quad_to` function.")),
    };

    let y = match args[3] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `quad_to` function.")),
    };

    let segments = vec![PathSegment::QuadTo(x1, y1, x, y)];
    let shape = Shape::Path {
        segments,
        transform: IDENTITY,
        color: WHITE,
    };
    Ok(Value::Shape(Arc::new(Mutex::new(shape))))
}

pub fn cubic_to(args: &[Value]) -> Result<Value> {
    if args.len() != 6 {
        return Err(anyhow!(
            "Invalid number of arguments to `cubic_to` function."
        ));
    }

    let x1 = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `cubic_to` function.")),
    };

    let y1 = match args[1] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `cubic_to` function.")),
    };

    let x2 = match args[2] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `cubic_to` function.")),
    };

    let y2 = match args[3] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `cubic_to` function.")),
    };

    let x = match args[4] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `cubic_to` function.")),
    };

    let y = match args[5] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `cubic_to` function.")),
    };

    let segments = vec![PathSegment::CubicTo(x1, y1, x2, y2, x, y)];
    let shape = Shape::Path {
        segments,
        transform: IDENTITY,
        color: WHITE,
    };
    Ok(Value::Shape(Arc::new(Mutex::new(shape))))
}

pub fn close(args: &[Value]) -> Result<Value> {
    if args.len() != 0 {
        return Err(anyhow!("Invalid number of arguments to `close` function."));
    }

    let shape = Shape::Path {
        segments: vec![PathSegment::Close],
        transform: IDENTITY,
        color: WHITE,
    };
    Ok(Value::Shape(Arc::new(Mutex::new(shape))))
}
