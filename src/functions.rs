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
            $name:literal => $func:ident $param_count:literal
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

        // Generate the builtin_param_count function with match statements
        pub fn builtin_param_count(name: &str) -> usize {
            match name {
                $(
                    $name => $param_count,
                )*
                _ => unreachable!(),
            }
        }
    };
}

define_builtins! {
    "+" => add 2,
    "add" => add 2,
    "-" => sub 2,
    "sub" => sub 2,
    "*" => mul 2,
    "mul" => mul 2,
    "/" => div 2,
    "div" => div 2,
    "%" => modulo 2,
    "mod" => modulo 2,
    "**" => pow 2,
    "pow" => pow 2,
    "&" => bitand 2,
    "bitand" => bitand 2,
    "|" => bitor 2,
    "bitor" => bitor 2,
    "^" => bitxor 2,
    "bitxor" => bitxor 2,
    "<<" => bitleft 2,
    "bitleft" => bitleft 2,
    ">>" => bitright 2,
    "bitright" => bitright 2,
    "==" => eq 2,
    "eq" => eq 2,
    "!=" => neq 2,
    "neq" => neq 2,
    "<" => lt 2,
    "lt" => lt 2,
    "<=" => lte 2,
    "lte" => lte 2,
    ">" => gt 2,
    "gt" => gt 2,
    ">=" => gte 2,
    "gte" => gte 2,
    "&&" => and 2,
    "and" => and 2,
    "||" => or 2,
    "or" => or 2,
    ":" => compose 2,
    "compose" => compose 2,
    "collect" => collect 1,
    ".." => range 2,
    "range" => range 2,
    "..=" => rangei 2,
    "rangei" => rangei 2,
    "|>" => pipe 2,
    "pipe" => pipe 2,
    // "." => compose_fn 2,
    // "compose_fn" => compose_fn 2,
    "pi" => pi 0,
    "π" => pi 0,
    "sin" => sin 1,
    "cos" => cos 1,
    "tan" => tan 1,
    "asin" => asin 1,
    "acos" => acos 1,
    "atan" => atan 1,
    "atan2" => atan2 2,
    "sinh" => sinh 1,
    "cosh" => cosh 1,
    "tanh" => tanh 1,
    "asinh" => asinh 1,
    "acosh" => acosh 1,
    "atanh" => atanh 1,
    "ln" => ln 1,
    "log10" => log10 1,
    "log" => log 2,
    "floor" => floor 1,
    "ceil" => ceil 1,
    "abs" => abs 1,
    "sqrt" => sqrt 1,
    "min" => min 2,
    "max" => max 2,
    "rand" => rand 0,
    "randi" => randi 0,
    "rand_range" => rand_range 2,
    "randi_range" => randi_range 2,
    "rand_rangei" => rand_rangei 2,
    "randi_rangei" => randi_rangei 2,
    "shuffle" => shuffle 1,
    "choose" => choose 1,
    "t" => translate 3,
    "translate" => translate 3,
    "tx" => translatex 2,
    "translatex" => translatex 2,
    "ty" => translatey 2,
    "translatey" => translatey 2,
    "tt" => translateb 2,
    "translateb" => translateb 2,
    "r" => rotate 2,
    "rotate" => rotate 2,
    "ra" => rotate_at 4,
    "rotate_at" => rotate_at 4,
    "s" => scale 3,
    "scale" => scale 3,
    "sx" => scalex 2,
    "scalex" => scalex 2,
    "sy" => scaley 2,
    "scaley" => scaley 2,
    "ss" => scaleb 2,
    "scaleb" => scaleb 2,
    "k" => skew 3,
    "skew" => skew 3,
    "kx" => skewx 2,
    "skewx" => skewx 2,
    "ky" => skewy 2,
    "skewy" => skewy 2,
    "kk" => skewb 2,
    "skewb" => skewb 2,
    "f" => flip 2,
    "flip" => flip 2,
    "fh" => fliph 1,
    "fliph" => fliph 1,
    "fv" => flipv 1,
    "flipv" => flipv 1,
    "fd" => flipd 1,
    "flipd" => flipd 1,
    "z" => zindex 2,
    "zindex" => zindex 2,
    "hsl" => hsl 4,
    "hsla" => hsla 5,
    "h" => hue 2,
    "hue" => hue 2,
    "sat" => saturation 2,
    "saturation" => saturation 2,
    "l" => lightness 2,
    "lightness" => lightness 2,
    "a" => alpha 2,
    "alpha" => alpha 2,
    "hshift" => hshift 2,
    "sshift" => sshift 2,
    "lshift" => lshift 2,
    "ashift" => ashift 2,
    "hex" => hex 2,
    // "blend" => blend 2,
    "move_to" => move_to 2,
    "line_to" => line_to 2,
    "quad_to" => quad_to 4,
    "cubic_to" => cubic_to 6,
    "close" => close 0,
}

pub fn add(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f32 - b)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a - *b as f32)),
        _ => Err(anyhow!("Invalid type passed to `sub` function.")),
    }
}

pub fn mul(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a / b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f32 / b)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a / *b as f32)),
        _ => Err(anyhow!("Invalid type passed to `div` function.")),
    }
}

pub fn modulo(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a % b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a % b)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f32 % b)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a % *b as f32)),
        _ => Err(anyhow!("Invalid type passed to `mod` function.")),
    }
}

pub fn pow(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Float((*a as f32).powf(*b as f32))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.powf(*b))),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Float((*a as f32).powf(*b))),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a.powf(*b as f32))),
        _ => Err(anyhow!("Invalid type passed to `pow` function.")),
    }
}

pub fn bitand(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a & b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Integer(*a as i32 & *b as i32)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Integer(a & *b as i32)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Integer(*a as i32 & b)),
        _ => Err(anyhow!("Invalid type passed to `bitand` function.")),
    }
}

pub fn bitor(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a | b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Integer(*a as i32 | *b as i32)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Integer(a | *b as i32)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Integer(*a as i32 | b)),
        _ => Err(anyhow!("Invalid type passed to `bitor` function.")),
    }
}

pub fn bitxor(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a ^ b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Integer(*a as i32 ^ *b as i32)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Integer(a ^ *b as i32)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Integer(*a as i32 ^ b)),
        _ => Err(anyhow!("Invalid type passed to `bitxor` function.")),
    }
}

pub fn bitleft(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a << b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Integer((*a as i32) << *b as i32)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Integer(a << *b as i32)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Integer((*a as i32) << b)),
        _ => Err(anyhow!("Invalid type passed to `bitleft` function.")),
    }
}

pub fn bitright(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a >> b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Integer(*a as i32 >> *b as i32)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Integer(a >> *b as i32)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Integer(*a as i32 >> b)),
        _ => Err(anyhow!("Invalid type passed to `bitright` function.")),
    }
}

pub fn eq(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a < b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a < b)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Boolean((*a as f32) < *b)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Boolean(*a < *b as f32)),
        _ => Err(anyhow!("Invalid type passed to `lt` function.")),
    }
}

pub fn lte(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a <= b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a <= b)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Boolean((*a as f32) <= *b)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Boolean(*a <= *b as f32)),
        _ => Err(anyhow!("Invalid type passed to `lte` function.")),
    }
}

pub fn gt(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a > b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a > b)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Boolean((*a as f32) > *b)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Boolean(*a > *b as f32)),
        _ => Err(anyhow!("Invalid type passed to `gt` function.")),
    }
}

pub fn gte(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a >= b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a >= b)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Boolean((*a as f32) >= *b)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Boolean(*a >= *b as f32)),
        _ => Err(anyhow!("Invalid type passed to `gte` function.")),
    }
}

pub fn and(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match (&args[0], &args[1]) {
        (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(*a && *b)),
        _ => Err(anyhow!("Invalid type passed to `and` function.")),
    }
}

pub fn or(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match (&args[0], &args[1]) {
        (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(*a || *b)),
        _ => Err(anyhow!("Invalid type passed to `or` function.")),
    }
}

pub fn compose(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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
    let arg = args[0].clone();

    match args[1].clone() {
        Value::Function(name, argc, mut pre_args) => {
            pre_args.push(arg);
            pre_args.reverse();
            Ok(Value::Function(name, argc - 1, pre_args))
        }
        _ => Err(anyhow!("Invalid type passed to `pipe` function.")),
    }
}

pub fn pi(_rng: &mut ChaCha8Rng, _args: &[Value]) -> Result<Value> {
    Ok(Value::Float(PI))
}

pub fn sin(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).sin())),
        Value::Float(n) => Ok(Value::Float(n.sin())),
        _ => Err(anyhow!("Invalid type passed to `sin` function.")),
    }
}

pub fn cos(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).cos())),
        Value::Float(n) => Ok(Value::Float(n.cos())),
        _ => Err(anyhow!("Invalid type passed to `cos` function.")),
    }
}

pub fn tan(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).tan())),
        Value::Float(n) => Ok(Value::Float(n.tan())),
        _ => Err(anyhow!("Invalid type passed to `tan` function.")),
    }
}

pub fn asin(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).asin())),
        Value::Float(n) => Ok(Value::Float(n.asin())),
        _ => Err(anyhow!("Invalid type passed to `asin` function.")),
    }
}

pub fn acos(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).acos())),
        Value::Float(n) => Ok(Value::Float(n.acos())),
        _ => Err(anyhow!("Invalid type passed to `acos` function.")),
    }
}

pub fn atan(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).atan())),
        Value::Float(n) => Ok(Value::Float(n.atan())),
        _ => Err(anyhow!("Invalid type passed to `atan` function.")),
    }
}

pub fn atan2(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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
    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).sinh())),
        Value::Float(n) => Ok(Value::Float(n.sinh())),
        _ => Err(anyhow!("Invalid type passed to `sinh` function.")),
    }
}

pub fn cosh(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).cosh())),
        Value::Float(n) => Ok(Value::Float(n.cosh())),
        _ => Err(anyhow!("Invalid type passed to `cosh` function.")),
    }
}

pub fn tanh(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).tanh())),
        Value::Float(n) => Ok(Value::Float(n.tanh())),
        _ => Err(anyhow!("Invalid type passed to `tanh` function.")),
    }
}

pub fn asinh(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).asinh())),
        Value::Float(n) => Ok(Value::Float(n.asinh())),
        _ => Err(anyhow!("Invalid type passed to `asinh` function.")),
    }
}

pub fn acosh(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).acosh())),
        Value::Float(n) => Ok(Value::Float(n.acosh())),
        _ => Err(anyhow!("Invalid type passed to `acosh` function.")),
    }
}

pub fn atanh(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).atanh())),
        Value::Float(n) => Ok(Value::Float(n.atanh())),
        _ => Err(anyhow!("Invalid type passed to `atanh` function.")),
    }
}

pub fn ln(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).ln())),
        Value::Float(n) => Ok(Value::Float(n.ln())),
        _ => Err(anyhow!("Invalid type passed to `ln` function.")),
    }
}

pub fn log10(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).log10())),
        Value::Float(n) => Ok(Value::Float(n.log10())),
        _ => Err(anyhow!("Invalid type passed to `log10` function.")),
    }
}

pub fn log(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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
    match args[0] {
        Value::Integer(n) => Ok(Value::Integer(n.abs())),
        Value::Float(n) => Ok(Value::Float(n.abs())),
        _ => Err(anyhow!("Invalid type passed to `abs` function.")),
    }
}

pub fn floor(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => Ok(Value::Integer(n)),
        Value::Float(n) => Ok(Value::Integer(n.floor() as i32)),
        _ => Err(anyhow!("Invalid type passed to `floor` function.")),
    }
}

pub fn ceil(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => Ok(Value::Integer(n)),
        Value::Float(n) => Ok(Value::Integer(n.ceil() as i32)),
        _ => Err(anyhow!("Invalid type passed to `ceil` function.")),
    }
}

pub fn sqrt(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).sqrt())),
        Value::Float(n) => Ok(Value::Float(n.sqrt())),
        _ => Err(anyhow!("Invalid type passed to `sqrt` function.")),
    }
}

pub fn min(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(*a.max(b))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.max(*b))),
        (Value::Integer(a), Value::Float(b)) | (Value::Float(b), Value::Integer(a)) => {
            Ok(Value::Float((*a as f32).max(*b)))
        }
        _ => Err(anyhow!("Invalid type passed to `max` function.")),
    }
}

pub fn rand(rng: &mut ChaCha8Rng, _args: &[Value]) -> Result<Value> {
    Ok(Value::Float(rng.random()))
}

pub fn randi(rng: &mut ChaCha8Rng, _args: &[Value]) -> Result<Value> {
    if rng.random() {
        Ok(Value::Integer(1))
    } else {
        Ok(Value::Integer(0))
    }
}

pub fn rand_range(rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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
    let mut list = match &args[0] {
        Value::List(list) => list.clone(),
        _ => return Err(anyhow!("Invalid type passed to `shuffle` function.")),
    };

    list.shuffle(rng);
    Ok(Value::List(list))
}

pub fn choose(rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let list = match &args[0] {
        Value::List(list) => list,
        _ => return Err(anyhow!("Invalid type passed to `choose` function.")),
    };

    Ok(list.choose(rng).unwrap().clone())
}

pub fn translate(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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
    let shape = match &args[0] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `fliph` function.")),
    };

    shape.borrow_mut().fliph();
    Ok(Value::Shape(shape))
}

pub fn flipv(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let shape = match &args[0] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `flipv` function.")),
    };

    shape.borrow_mut().flipv();
    Ok(Value::Shape(shape))
}

pub fn flipd(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let shape = match &args[0] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `flipd` function.")),
    };

    shape.borrow_mut().flipd();
    Ok(Value::Shape(shape))
}

pub fn zindex(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
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
    let shape = Shape::Path {
        segments: vec![PathSegment::Close],
        transform: IDENTITY,
        zindex: None,
        color: WHITE,
    };
    Ok(Value::Shape(Rc::new(RefCell::new(shape))))
}
