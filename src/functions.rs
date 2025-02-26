use crate::compiler::{Shape, Value, IDENTITY, TRANSPARENT};

use anyhow::{anyhow, Result};
use core::f32::consts::PI;

pub const BUILTIN_FUNCTIONS: &[&str] = &[
    "pi",
    "π",
    "sin",
    "cos",
    "tan",
    "+",
    "add",
    "-",
    "sub",
    "*",
    "mul",
    "/",
    "div",
    "==",
    "eq",
    "!=",
    "neq",
    "<",
    "lt",
    "<=",
    "lte",
    ">",
    "gt",
    ">=",
    "gte",
    "&&",
    "and",
    "||",
    "or",
    ":",
    "compose",
    "collect",
    "..",
    "range",
    "..=",
    "rangei",
    "t",
    "translate",
    "tx",
    "translatex",
    "ty",
    "translatey",
    "tt",
    "translateb",
    "z",
    "zindex",
    "r",
    "rotate",
    "ra",
    "rotateat",
    "s",
    "scale",
    "sx",
    "scalex",
    "sy",
    "scaley",
    "ss",
    "scaleb",
    "k",
    "skew",
    "kx",
    "skewx",
    "ky",
    "skewy",
    "kk",
    "skewb",
    "f",
    "flip",
    "h",
    "hue",
    "sat",
    "saturation",
    "l",
    "lightness",
    "a",
    "alpha",
    "blend",
];

fn pi(args: &[Value]) -> Result<Value> {
    if args.len() != 0 {
        return Err(anyhow!("Invalid number of arguments to `pi` function."));
    }

    Ok(Value::Float(PI))
}

fn sin(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `sin` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).sin())),
        Value::Float(n) => Ok(Value::Float(n.sin())),
        _ => Err(anyhow!("Invalid type passed to `sin` function.")),
    }
}

fn cos(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `cos` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).cos())),
        Value::Float(n) => Ok(Value::Float(n.cos())),
        _ => Err(anyhow!("Invalid type passed to `cos` function.")),
    }
}

fn tan(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `tan` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).tan())),
        Value::Float(n) => Ok(Value::Float(n.tan())),
        _ => Err(anyhow!("Invalid type passed to `tan` function.")),
    }
}

fn abs(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(anyhow!("Invalid number of arguments to `abs` function."));
    }

    match args[0] {
        Value::Integer(n) => Ok(Value::Integer(n.abs())),
        Value::Float(n) => Ok(Value::Float(n.abs())),
        _ => Err(anyhow!("Invalid type passed to `abs` function.")),
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
        (Value::Integer(a), Value::Float(b)) | (Value::Float(b), Value::Integer(a)) => {
            Ok(Value::Float(*a as f32 - b))
        }
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
        (Value::Integer(a), Value::Float(b)) | (Value::Float(b), Value::Integer(a)) => {
            Ok(Value::Float(*a as f32 / b))
        }
        _ => Err(anyhow!("Invalid type passed to `div` function.")),
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
        (Value::Shape(a), Value::Shape(b)) => Ok(Value::Boolean(a == b)),
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
        (Value::Shape(a), Value::Shape(b)) => Ok(Value::Boolean(a != b)),
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
            let shape = Shape::Composite {
                shapes: vec![*a, *b].leak(),
                transform: IDENTITY,
                color: TRANSPARENT,
            };
            Ok(Value::Shape(shape))
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
            let mut shapes = Vec::new();
            for item in list {
                match item {
                    Value::Shape(shape) => shapes.push(*shape),
                    _ => return Err(anyhow!("Invalid type passed to `collect` function.")),
                }
            }
            let shape = Shape::Composite {
                shapes: shapes.leak(),
                transform: IDENTITY,
                color: TRANSPARENT,
            };
            Ok(Value::Shape(shape))
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

    let shape = match args[2] {
        Value::Shape(shape) => shape,
        _ => return Err(anyhow!("Invalid type passed to `translate` function.")),
    };

    Ok(Value::Shape(shape.translate(tx, ty)))
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

    let shape = match args[1] {
        Value::Shape(shape) => shape,
        _ => return Err(anyhow!("Invalid type passed to `translatex` function.")),
    };

    Ok(Value::Shape(shape.translate(tx, 0.0)))
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

    let shape = match args[1] {
        Value::Shape(shape) => shape,
        _ => return Err(anyhow!("Invalid type passed to `translatey` function.")),
    };

    Ok(Value::Shape(shape.translate(0.0, ty)))
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

    let shape = match args[1] {
        Value::Shape(shape) => shape,
        _ => return Err(anyhow!("Invalid type passed to `translateb` function.")),
    };

    Ok(Value::Shape(shape.translate(t, t)))
}

pub fn zindex(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `zindex` function."));
    }

    let z = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `zindex` function.")),
    };

    let shape = match args[1] {
        Value::Shape(shape) => shape,
        _ => return Err(anyhow!("Invalid type passed to `zindex` function.")),
    };

    // Ok(Value::Shape(shape.zindex(z)))
    todo!()
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

    let shape = match args[1] {
        Value::Shape(shape) => shape,
        _ => return Err(anyhow!("Invalid type passed to `rotate` function.")),
    };

    Ok(Value::Shape(shape.rotate(r)))
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

    let shape = match args[3] {
        Value::Shape(shape) => shape,
        _ => return Err(anyhow!("Invalid type passed to `rotate_at` function.")),
    };

    Ok(Value::Shape(shape.rotate_at(r, tx, ty)))
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

    let shape = match args[2] {
        Value::Shape(shape) => shape,
        _ => return Err(anyhow!("Invalid type passed to `scale` function.")),
    };

    Ok(Value::Shape(shape.scale(sx, sy)))
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

    let shape = match args[1] {
        Value::Shape(shape) => shape,
        _ => return Err(anyhow!("Invalid type passed to `scalex` function.")),
    };

    Ok(Value::Shape(shape.scale(sx, 1.0)))
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

    let shape = match args[1] {
        Value::Shape(shape) => shape,
        _ => return Err(anyhow!("Invalid type passed to `scaley` function.")),
    };

    Ok(Value::Shape(shape.scale(1.0, sy)))
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

    let shape = match args[1] {
        Value::Shape(shape) => shape,
        _ => return Err(anyhow!("Invalid type passed to `scaleb` function.")),
    };

    Ok(Value::Shape(shape.scale(s, s)))
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

    let shape = match args[2] {
        Value::Shape(shape) => shape,
        _ => return Err(anyhow!("Invalid type passed to `skew` function.")),
    };

    Ok(Value::Shape(shape.skew(kx, ky)))
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

    let shape = match args[1] {
        Value::Shape(shape) => shape,
        _ => return Err(anyhow!("Invalid type passed to `skewx` function.")),
    };

    Ok(Value::Shape(shape.scale(kx, 0.0)))
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

    let shape = match args[1] {
        Value::Shape(shape) => shape,
        _ => return Err(anyhow!("Invalid type passed to `skewy` function.")),
    };

    Ok(Value::Shape(shape.scale(0.0, ky)))
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

    let shape = match args[1] {
        Value::Shape(shape) => shape,
        _ => return Err(anyhow!("Invalid type passed to `skewb` function.")),
    };

    Ok(Value::Shape(shape.skew(k, k)))
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

    let shape = match args[1] {
        Value::Shape(shape) => shape,
        _ => return Err(anyhow!("Invalid type passed to `flip` function.")),
    };

    Ok(Value::Shape(shape.flip(f)))
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

    let shape = match args[1] {
        Value::Shape(shape) => shape,
        _ => return Err(anyhow!("Invalid type passed to `hue` function.")),
    };

    Ok(Value::Shape(shape.hue(h)))
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

    let shape = match args[1] {
        Value::Shape(shape) => shape,
        _ => return Err(anyhow!("Invalid type passed to `saturation` function.")),
    };

    Ok(Value::Shape(shape.saturation(s)))
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

    let shape = match args[1] {
        Value::Shape(shape) => shape,
        _ => return Err(anyhow!("Invalid type passed to `lightness` function.")),
    };

    Ok(Value::Shape(shape.lightness(l)))
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

    let shape = match args[1] {
        Value::Shape(shape) => shape,
        _ => return Err(anyhow!("Invalid type passed to `alpha` function.")),
    };

    Ok(Value::Shape(shape.alpha(a)))
}

pub fn blend(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(anyhow!("Invalid number of arguments to `blend` function."));
    }

    // let blend = match args[0] {
    // 	Value::Blend(blend) => blend,
    //     _ => return Err(anyhow!("Invalid type passed to `blend` function.")),
    // };

    // let shape = match args[1] {
    //     Value::Shape(shape) => shape,
    //     _ => return Err(anyhow!("Invalid type passed to `blend` function.")),
    // };

    // Ok(Value::Shape(shape.blend(blend)))

    todo!()
}

pub fn handle_builtin(name: &str, args: &[Value]) -> Result<Value> {
    match name {
        "pi" | "π" => pi(args),
        "sin" => sin(args),
        "cos" => cos(args),
        "tan" => tan(args),
        "abs" => abs(args),
        "+" | "add" => add(args),
        "-" | "sub" => sub(args),
        "*" | "mul" => mul(args),
        "/" | "div" => div(args),
        "==" | "eq" => eq(args),
        "!=" | "neq" => neq(args),
        "<" | "lt" => lt(args),
        "<=" | "lte" => lte(args),
        ">" | "gt" => gt(args),
        ">=" | "gte" => gte(args),
        "&&" | "and" => and(args),
        "||" | "or" => or(args),
        ":" | "compose" => compose(args),
        "collect" => collect(args),
        ".." | "range" => range(args),
        "..=" | "rangei" => rangei(args),
        "t" | "translate" => translate(args),
        "tx" | "translatex" => translatex(args),
        "ty" | "translatey" => translatey(args),
        "tt" | "translateb" => translateb(args),
        "z" | "zindex" => zindex(args),
        "r" | "rotate" => rotate(args),
        "ra" | "rotate_at" => rotate_at(args),
        "s" | "scale" => scale(args),
        "sx" | "scalex" => scalex(args),
        "sy" | "scaley" => scaley(args),
        "ss" | "scaleb" => scaleb(args),
        "k" | "skew" => skew(args),
        "kx" | "skewx" => skewx(args),
        "ky" | "skewy" => skewy(args),
        "kk" | "skewb" => skewb(args),
        "f" | "flip" => flip(args),
        // "hsl" => hsl(args),
        // "hsla" => hsla(args),
        "h" | "hue" => hue(args),
        "sat" | "saturation" => saturation(args),
        "l" | "lightness" => lightness(args),
        "a" | "alpha" => alpha(args),
        "blend" => blend(args),
        _ => unreachable!(),
    }
}
