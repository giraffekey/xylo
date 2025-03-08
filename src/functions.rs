#[cfg(feature = "std")]
use std::rc::Rc;

#[cfg(feature = "no-std")]
use alloc::{rc::Rc, vec, vec::Vec};

use crate::interpreter::Value;
use crate::shape::{PathSegment, Shape, IDENTITY, TRANSPARENT, WHITE};
use core::cell::RefCell;

use anyhow::{anyhow, Result};
use core::f32::consts::PI;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

macro_rules! define_builtins {
    (
        $(
            $name:literal => $func:ident
        ),* $(,)?
    ) => {
        // Generate the static BUILTIN_FUNCTIONS array
        pub static BUILTIN_FUNCTIONS: &[&str] = &[
            $($name),*
        ];

        // Generate the handle_builtin function with match statements
        pub fn handle_builtin(name: &str, rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
            match name {
                $(
                    $name => $func(rng, args),
                )*
                _ => Err(anyhow!("Unknown function: {}", name)),
            }
        }
    };
}

define_builtins! {
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
    "&" => bitand,
    "bitand" => bitand,
    "|" => bitor,
    "bitor" => bitor,
    "^" => bitxor,
    "bitxor" => bitxor,
    "<<" => bitleft,
    "bitleft" => bitleft,
    ">>" => bitright,
    "bitright" => bitright,
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
    "|>" => pipe,
    "pipe" => pipe,
    // "." => compose_fn,
    // "compose_fn" => compose_fn,
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
    "rand" => rand,
    "randi" => randi,
    "rand_range" => rand_range,
    "randi_range" => randi_range,
    "rand_rangei" => rand_rangei,
    "randi_rangei" => randi_rangei,
    "shuffle" => shuffle,
    "choose" => choose,
    "t" => translate,
    "translate" => translate,
    "tx" => translatex,
    "translatex" => translatex,
    "ty" => translatey,
    "translatey" => translatey,
    "tt" => translateb,
    "translateb" => translateb,
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
    "z" => zindex,
    "zindex" => zindex,
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

pub fn add(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

pub fn sub(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

pub fn mul(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

pub fn div(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

pub fn modulo(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

pub fn pow(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

pub fn bitand(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `bitand` function."));
    }

    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a & b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Integer(*a as i32 & *b as i32)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Integer(a & *b as i32)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Integer(*a as i32 & b)),
        _ => Err(anyhow!("Invalid type passed to `bitand` function.")),
    }
}

pub fn bitor(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `bitor` function."));
    }

    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a | b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Integer(*a as i32 | *b as i32)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Integer(a | *b as i32)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Integer(*a as i32 | b)),
        _ => Err(anyhow!("Invalid type passed to `bitor` function.")),
    }
}

pub fn bitxor(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `bitxor` function."));
    }

    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a ^ b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Integer(*a as i32 ^ *b as i32)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Integer(a ^ *b as i32)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Integer(*a as i32 ^ b)),
        _ => Err(anyhow!("Invalid type passed to `bitxor` function.")),
    }
}

pub fn bitleft(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!(
            "Invalid number of arguments to `bitleft` function."
        ));
    }

    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a << b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Integer((*a as i32) << *b as i32)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Integer(a << *b as i32)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Integer((*a as i32) << b)),
        _ => Err(anyhow!("Invalid type passed to `bitleft` function.")),
    }
}

pub fn bitright(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!(
            "Invalid number of arguments to `bitright` function."
        ));
    }

    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a >> b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Integer(*a as i32 >> *b as i32)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Integer(a >> *b as i32)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Integer(*a as i32 >> b)),
        _ => Err(anyhow!("Invalid type passed to `bitright` function.")),
    }
}

pub fn eq(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

pub fn neq(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

pub fn lt(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

pub fn lte(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

pub fn gt(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

pub fn gte(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

pub fn and(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `and` function."));
    }

    match (&args[0], &args[1]) {
        (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(*a && *b)),
        _ => Err(anyhow!("Invalid type passed to `and` function.")),
    }
}

pub fn or(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `or` function."));
    }

    match (&args[0], &args[1]) {
        (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(*a || *b)),
        _ => Err(anyhow!("Invalid type passed to `or` function.")),
    }
}

pub fn compose(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!(
            "Invalid number of arguments to `compose` function."
        ));
    }

    match (&args[0], &args[1]) {
        (Value::Shape(a), Value::Shape(b)) => {
            let shape = match (&*a.borrow(), &*b.borrow()) {
                (
                    Shape::Path {
                        segments: a,
                        transform: a_transform,
                        zindex,
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
                        zindex: *zindex,
                        color: *color,
                    }
                }
                _ => Shape::Composite {
                    a: a.clone(),
                    b: b.clone(),
                    transform: IDENTITY,
                    zindex: None,
                    color: TRANSPARENT,
                },
            };
            Ok(Value::Shape(Rc::new(RefCell::new(shape))))
        }
        _ => Err(anyhow!("Invalid type passed to `compose` function.")),
    }
}

pub fn collect(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!(
            "Invalid number of arguments to `collect` function."
        ));
    }

    match &args[0] {
        Value::List(list) => {
            let shapes: Result<Vec<Rc<RefCell<Shape>>>> = list
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

            let is_path = shapes.iter().all(|shape| match &*shape.borrow() {
                Shape::Path { .. } => true,
                _ => false,
            });
            let shape = if is_path {
                let mut segments = Vec::with_capacity(shapes.len());
                let mut transform = IDENTITY;
                let color = WHITE;

                for path in shapes {
                    match &*path.borrow() {
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
                    zindex: None,
                    color,
                }
            } else {
                Shape::Collection {
                    shapes,
                    transform: IDENTITY,
                    zindex: None,
                    color: TRANSPARENT,
                }
            };
            Ok(Value::Shape(Rc::new(RefCell::new(shape))))
        }
        _ => Err(anyhow!("Invalid type passed to `collect` function.")),
    }
}

pub fn range(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

pub fn rangei(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

pub fn pipe(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `pipe` function."));
    }

    let arg = args[0].clone();

    match args[1].clone() {
        Value::Function(name, argc, mut pre_args) => {
            pre_args.push(arg);
            Ok(Value::Function(name, argc - 1, pre_args))
        }
        _ => Err(anyhow!("Invalid type passed to `pipe` function.")),
    }
}

pub fn pi(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 0 {
        return Err(anyhow!("Invalid number of arguments to `pi` function."));
    }

    Ok(Value::Float(PI))
}

pub fn sin(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `sin` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).sin())),
        Value::Float(n) => Ok(Value::Float(n.sin())),
        _ => Err(anyhow!("Invalid type passed to `sin` function.")),
    }
}

pub fn cos(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `cos` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).cos())),
        Value::Float(n) => Ok(Value::Float(n.cos())),
        _ => Err(anyhow!("Invalid type passed to `cos` function.")),
    }
}

pub fn tan(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `tan` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).tan())),
        Value::Float(n) => Ok(Value::Float(n.tan())),
        _ => Err(anyhow!("Invalid type passed to `tan` function.")),
    }
}

pub fn asin(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `asin` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).asin())),
        Value::Float(n) => Ok(Value::Float(n.asin())),
        _ => Err(anyhow!("Invalid type passed to `asin` function.")),
    }
}

pub fn acos(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `acos` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).acos())),
        Value::Float(n) => Ok(Value::Float(n.acos())),
        _ => Err(anyhow!("Invalid type passed to `acos` function.")),
    }
}

pub fn atan(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `atan` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).atan())),
        Value::Float(n) => Ok(Value::Float(n.atan())),
        _ => Err(anyhow!("Invalid type passed to `atan` function.")),
    }
}

pub fn atan2(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

pub fn sinh(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `sinh` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).sinh())),
        Value::Float(n) => Ok(Value::Float(n.sinh())),
        _ => Err(anyhow!("Invalid type passed to `sinh` function.")),
    }
}

pub fn cosh(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `cosh` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).cosh())),
        Value::Float(n) => Ok(Value::Float(n.cosh())),
        _ => Err(anyhow!("Invalid type passed to `cosh` function.")),
    }
}

pub fn tanh(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `tanh` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).tanh())),
        Value::Float(n) => Ok(Value::Float(n.tanh())),
        _ => Err(anyhow!("Invalid type passed to `tanh` function.")),
    }
}

pub fn asinh(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `asinh` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).asinh())),
        Value::Float(n) => Ok(Value::Float(n.asinh())),
        _ => Err(anyhow!("Invalid type passed to `asinh` function.")),
    }
}

pub fn acosh(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `acosh` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).acosh())),
        Value::Float(n) => Ok(Value::Float(n.acosh())),
        _ => Err(anyhow!("Invalid type passed to `acosh` function.")),
    }
}

pub fn atanh(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `atanh` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).atanh())),
        Value::Float(n) => Ok(Value::Float(n.atanh())),
        _ => Err(anyhow!("Invalid type passed to `atanh` function.")),
    }
}

pub fn ln(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `ln` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).ln())),
        Value::Float(n) => Ok(Value::Float(n.ln())),
        _ => Err(anyhow!("Invalid type passed to `ln` function.")),
    }
}

pub fn log10(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `log10` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).log10())),
        Value::Float(n) => Ok(Value::Float(n.log10())),
        _ => Err(anyhow!("Invalid type passed to `log10` function.")),
    }
}

pub fn log(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

pub fn abs(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `abs` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Integer(n.abs())),
        Value::Float(n) => Ok(Value::Float(n.abs())),
        _ => Err(anyhow!("Invalid type passed to `abs` function.")),
    }
}

pub fn floor(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `floor` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Integer(n)),
        Value::Float(n) => Ok(Value::Integer(n.floor() as i32)),
        _ => Err(anyhow!("Invalid type passed to `floor` function.")),
    }
}

pub fn ceil(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `ceil` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Integer(n)),
        Value::Float(n) => Ok(Value::Integer(n.ceil() as i32)),
        _ => Err(anyhow!("Invalid type passed to `ceil` function.")),
    }
}

pub fn sqrt(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `sqrt` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).sqrt())),
        Value::Float(n) => Ok(Value::Float(n.sqrt())),
        _ => Err(anyhow!("Invalid type passed to `sqrt` function.")),
    }
}

pub fn min(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

pub fn max(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

pub fn rand(rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 0 {
        return Err(anyhow!("Invalid number of arguments to `rand` function."));
    }

    Ok(Value::Float(rng.random()))
}

pub fn randi(rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 0 {
        return Err(anyhow!("Invalid number of arguments to `randi` function."));
    }

    if rng.random() {
        Ok(Value::Integer(1))
    } else {
        Ok(Value::Integer(0))
    }
}

pub fn rand_range(rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!(
            "Invalid number of arguments to `rand_range` function."
        ));
    }

    let a = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `rand_range` function.")),
    };

    let b = match args[1] {
        Value::Integer(b) => b as f32,
        Value::Float(b) => b,
        _ => return Err(anyhow!("Invalid type passed to `rand_range` function.")),
    };

    Ok(Value::Float(rng.random_range(a..b)))
}

pub fn randi_range(rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!(
            "Invalid number of arguments to `randi_range` function."
        ));
    }

    let a = match args[0] {
        Value::Integer(n) => n,
        Value::Float(n) => n as i32,
        _ => return Err(anyhow!("Invalid type passed to `randi_range` function.")),
    };

    let b = match args[1] {
        Value::Integer(b) => b,
        Value::Float(b) => b as i32,
        _ => return Err(anyhow!("Invalid type passed to `randi_range` function.")),
    };

    Ok(Value::Integer(rng.random_range(a..b)))
}

pub fn rand_rangei(rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!(
            "Invalid number of arguments to `rand_rangei` function."
        ));
    }

    let a = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `rand_rangei` function.")),
    };

    let b = match args[1] {
        Value::Integer(b) => b as f32,
        Value::Float(b) => b,
        _ => return Err(anyhow!("Invalid type passed to `rand_rangei` function.")),
    };

    Ok(Value::Float(rng.random_range(a..=b)))
}

pub fn randi_rangei(rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!(
            "Invalid number of arguments to `randi_rangei` function."
        ));
    }

    let a = match args[0] {
        Value::Integer(n) => n,
        Value::Float(n) => n as i32,
        _ => return Err(anyhow!("Invalid type passed to `randi_rangei` function.")),
    };

    let b = match args[1] {
        Value::Integer(b) => b,
        Value::Float(b) => b as i32,
        _ => return Err(anyhow!("Invalid type passed to `randi_rangei` function.")),
    };

    Ok(Value::Integer(rng.random_range(a..=b)))
}

pub fn shuffle(rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!(
            "Invalid number of arguments to `shuffle` function."
        ));
    }

    let mut list = match &args[0] {
        Value::List(list) => list.clone(),
        _ => return Err(anyhow!("Invalid type passed to `shuffle` function.")),
    };

    list.shuffle(rng);
    Ok(Value::List(list))
}

pub fn choose(rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `choose` function."));
    }

    let list = match &args[0] {
        Value::List(list) => list,
        _ => return Err(anyhow!("Invalid type passed to `choose` function.")),
    };

    Ok(list.choose(rng).unwrap().clone())
}

pub fn translate(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

    shape.borrow_mut().translate(tx, ty);

    Ok(Value::Shape(shape))
}

pub fn translatex(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

    shape.borrow_mut().translate(tx, 0.0);
    Ok(Value::Shape(shape))
}

pub fn translatey(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

    shape.borrow_mut().translate(0.0, ty);
    Ok(Value::Shape(shape))
}

pub fn translateb(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

    shape.borrow_mut().translate(t, t);
    Ok(Value::Shape(shape))
}

pub fn rotate(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

    shape.borrow_mut().rotate(r);
    Ok(Value::Shape(shape))
}

pub fn rotate_at(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

    shape.borrow_mut().rotate_at(r, tx, ty);
    Ok(Value::Shape(shape))
}

pub fn scale(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

    shape.borrow_mut().scale(sx, sy);
    Ok(Value::Shape(shape))
}

pub fn scalex(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

    shape.borrow_mut().scale(sx, 1.0);
    Ok(Value::Shape(shape))
}

pub fn scaley(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

    shape.borrow_mut().scale(1.0, sy);
    Ok(Value::Shape(shape))
}

pub fn scaleb(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

    shape.borrow_mut().scale(s, s);
    Ok(Value::Shape(shape))
}

pub fn skew(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

    shape.borrow_mut().skew(kx, ky);
    Ok(Value::Shape(shape))
}

pub fn skewx(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

    shape.borrow_mut().skew(kx, 0.0);
    Ok(Value::Shape(shape))
}

pub fn skewy(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

    shape.borrow_mut().skew(0.0, ky);
    Ok(Value::Shape(shape))
}

pub fn skewb(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

    shape.borrow_mut().skew(k, k);
    Ok(Value::Shape(shape))
}

pub fn flip(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

    shape.borrow_mut().flip(f);
    Ok(Value::Shape(shape))
}

pub fn fliph(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `fliph` function."));
    }

    let shape = match &args[0] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `fliph` function.")),
    };

    shape.borrow_mut().fliph();
    Ok(Value::Shape(shape))
}

pub fn flipv(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `flipv` function."));
    }

    let shape = match &args[0] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `flipv` function.")),
    };

    shape.borrow_mut().flipv();
    Ok(Value::Shape(shape))
}

pub fn flipd(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `flipd` function."));
    }

    let shape = match &args[0] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `flipd` function.")),
    };

    shape.borrow_mut().flipd();
    Ok(Value::Shape(shape))
}

pub fn zindex(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `zindex` function."));
    }

    let z = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `zindex` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `zindex` function.")),
    };

    shape.borrow_mut().set_zindex(z);
    Ok(Value::Shape(shape))
}

pub fn hsl(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

    shape.borrow_mut().set_hsl(h, s, l);
    Ok(Value::Shape(shape))
}

pub fn hsla(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

    shape.borrow_mut().set_hsla(h, s, l, a);
    Ok(Value::Shape(shape))
}

pub fn hue(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

    shape.borrow_mut().set_hue(h);
    Ok(Value::Shape(shape))
}

pub fn saturation(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

    shape.borrow_mut().set_saturation(s);
    Ok(Value::Shape(shape))
}

pub fn lightness(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

    shape.borrow_mut().set_lightness(l);
    Ok(Value::Shape(shape))
}

pub fn alpha(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

    shape.borrow_mut().set_alpha(a);
    Ok(Value::Shape(shape))
}

pub fn hshift(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

    shape.borrow_mut().shift_hue(h);
    Ok(Value::Shape(shape))
}

pub fn sshift(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

    shape.borrow_mut().shift_saturation(s);
    Ok(Value::Shape(shape))
}

pub fn lshift(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

    shape.borrow_mut().shift_lightness(l);
    Ok(Value::Shape(shape))
}

pub fn ashift(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

    shape.borrow_mut().shift_alpha(a);
    Ok(Value::Shape(shape))
}

pub fn hex(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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

    shape.borrow_mut().set_hex(hex);
    Ok(Value::Shape(shape))
}

pub fn move_to(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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
        zindex: None,
        color: WHITE,
    };
    Ok(Value::Shape(Rc::new(RefCell::new(shape))))
}

pub fn line_to(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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
        zindex: None,
        color: WHITE,
    };
    Ok(Value::Shape(Rc::new(RefCell::new(shape))))
}

pub fn quad_to(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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
        zindex: None,
        color: WHITE,
    };
    Ok(Value::Shape(Rc::new(RefCell::new(shape))))
}

pub fn cubic_to(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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
        zindex: None,
        color: WHITE,
    };
    Ok(Value::Shape(Rc::new(RefCell::new(shape))))
}

pub fn close(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    if args.len() != 0 {
        return Err(anyhow!("Invalid number of arguments to `close` function."));
    }

    let shape = Shape::Path {
        segments: vec![PathSegment::Close],
        transform: IDENTITY,
        zindex: None,
        color: WHITE,
    };
    Ok(Value::Shape(Rc::new(RefCell::new(shape))))
}
