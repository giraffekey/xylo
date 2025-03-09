use crate::interpreter::Value;

use anyhow::{anyhow, Result};
use core::f32::consts::{E, PI, TAU};
use factorial::{DoubleFactorial, Factorial};
use num::complex::{Complex, ComplexFloat};
use rand_chacha::ChaCha8Rng;

// Defined manually until more_float_constants is stable
const PHI: f32 = 1.618033988749894848204586834365638118_f32;

pub fn neg(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match &args[0] {
        Value::Integer(n) => Ok(Value::Integer(-n)),
        Value::Float(n) => Ok(Value::Float(-n)),
        Value::Complex(n) => Ok(Value::Complex(-n)),
        _ => Err(anyhow!("Invalid type passed to `neg` function.")),
    }
}

pub fn bitnot(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match &args[0] {
        Value::Integer(n) => Ok(Value::Integer(!n)),
        Value::Float(n) => Ok(Value::Float(!(*n as i32) as f32)),
        _ => Err(anyhow!("Invalid type passed to `neg` function.")),
    }
}

pub fn add(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
        (Value::Complex(a), Value::Complex(b)) => Ok(Value::Complex(a + b)),
        (Value::Integer(a), Value::Float(b)) | (Value::Float(b), Value::Integer(a)) => {
            Ok(Value::Float(*a as f32 + b))
        }
        (Value::Complex(a), Value::Integer(b)) | (Value::Integer(b), Value::Complex(a)) => {
            Ok(Value::Complex(a + (*b as f32)))
        }
        (Value::Complex(a), Value::Float(b)) | (Value::Float(b), Value::Complex(a)) => {
            Ok(Value::Complex(a + b))
        }
        _ => Err(anyhow!("Invalid type passed to `add` function.")),
    }
}

pub fn sub(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
        (Value::Complex(a), Value::Complex(b)) => Ok(Value::Complex(a - b)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f32 - b)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a - *b as f32)),
        (Value::Complex(a), Value::Integer(b)) => Ok(Value::Complex(a - *b as f32)),
        (Value::Integer(a), Value::Complex(b)) => Ok(Value::Complex(*a as f32 - b)),
        (Value::Complex(a), Value::Float(b)) => Ok(Value::Complex(a - b)),
        (Value::Float(a), Value::Complex(b)) => Ok(Value::Complex(a - b)),
        _ => Err(anyhow!("Invalid type passed to `sub` function.")),
    }
}

pub fn mul(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
        (Value::Complex(a), Value::Complex(b)) => Ok(Value::Complex(a * b)),
        (Value::Integer(a), Value::Float(b)) | (Value::Float(b), Value::Integer(a)) => {
            Ok(Value::Float(*a as f32 * b))
        }
        (Value::Complex(a), Value::Integer(b)) | (Value::Integer(b), Value::Complex(a)) => {
            Ok(Value::Complex(a * (*b as f32)))
        }
        (Value::Complex(a), Value::Float(b)) | (Value::Float(b), Value::Complex(a)) => {
            Ok(Value::Complex(a * b))
        }
        _ => Err(anyhow!("Invalid type passed to `mul` function.")),
    }
}

pub fn div(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a / b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
        (Value::Complex(a), Value::Complex(b)) => Ok(Value::Complex(a / b)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f32 / b)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a / *b as f32)),
        (Value::Complex(a), Value::Integer(b)) => Ok(Value::Complex(a / *b as f32)),
        (Value::Integer(a), Value::Complex(b)) => Ok(Value::Complex(*a as f32 / b)),
        (Value::Complex(a), Value::Float(b)) => Ok(Value::Complex(a / b)),
        (Value::Float(a), Value::Complex(b)) => Ok(Value::Complex(a / b)),
        _ => Err(anyhow!("Invalid type passed to `div` function.")),
    }
}

pub fn modulo(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a % b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a % b)),
        (Value::Complex(a), Value::Complex(b)) => Ok(Value::Complex(a % b)),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f32 % b)),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a % *b as f32)),
        (Value::Complex(a), Value::Integer(b)) => Ok(Value::Complex(a % *b as f32)),
        (Value::Integer(a), Value::Complex(b)) => Ok(Value::Complex(*a as f32 % b)),
        (Value::Complex(a), Value::Float(b)) => Ok(Value::Complex(a % b)),
        (Value::Float(a), Value::Complex(b)) => Ok(Value::Complex(a % b)),
        _ => Err(anyhow!("Invalid type passed to `mod` function.")),
    }
}

pub fn pow(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Float((*a as f32).powf(*b as f32))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.powf(*b))),
        (Value::Complex(a), Value::Complex(b)) => Ok(Value::Complex(a.powc(*b))),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Float((*a as f32).powf(*b))),
        (Value::Integer(a), Value::Complex(b)) => Ok(Value::Complex((*a as f32).powc(*b))),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a.powf(*b as f32))),
        (Value::Float(a), Value::Complex(b)) => Ok(Value::Complex(a.powc(*b))),
        (Value::Complex(a), Value::Integer(b)) => Ok(Value::Complex(a.powf(*b as f32))),
        (Value::Complex(a), Value::Float(b)) => Ok(Value::Complex(a.powf(*b))),
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

pub fn complex(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match (&args[0], &args[1]) {
        (Value::Float(re), Value::Float(im)) => Ok(Value::Complex(Complex::new(*re, *im))),
        _ => Err(anyhow!("Invalid type passed to `complex` function.")),
    }
}

pub fn pi(_rng: &mut ChaCha8Rng, _args: &[Value]) -> Result<Value> {
    Ok(Value::Float(PI))
}

pub fn tau(_rng: &mut ChaCha8Rng, _args: &[Value]) -> Result<Value> {
    Ok(Value::Float(TAU))
}

pub fn e(_rng: &mut ChaCha8Rng, _args: &[Value]) -> Result<Value> {
    Ok(Value::Float(E))
}

pub fn phi(_rng: &mut ChaCha8Rng, _args: &[Value]) -> Result<Value> {
    Ok(Value::Float(PHI))
}

pub fn sin(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).sin())),
        Value::Float(n) => Ok(Value::Float(n.sin())),
        Value::Complex(n) => Ok(Value::Complex(n.sin())),
        _ => Err(anyhow!("Invalid type passed to `sin` function.")),
    }
}

pub fn cos(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).cos())),
        Value::Float(n) => Ok(Value::Float(n.cos())),
        Value::Complex(n) => Ok(Value::Complex(n.cos())),
        _ => Err(anyhow!("Invalid type passed to `cos` function.")),
    }
}

pub fn tan(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).tan())),
        Value::Float(n) => Ok(Value::Float(n.tan())),
        Value::Complex(n) => Ok(Value::Complex(n.tan())),
        _ => Err(anyhow!("Invalid type passed to `tan` function.")),
    }
}

pub fn asin(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).asin())),
        Value::Float(n) => Ok(Value::Float(n.asin())),
        Value::Complex(n) => Ok(Value::Complex(n.asin())),
        _ => Err(anyhow!("Invalid type passed to `asin` function.")),
    }
}

pub fn acos(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).acos())),
        Value::Float(n) => Ok(Value::Float(n.acos())),
        Value::Complex(n) => Ok(Value::Complex(n.acos())),
        _ => Err(anyhow!("Invalid type passed to `acos` function.")),
    }
}

pub fn atan(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).atan())),
        Value::Float(n) => Ok(Value::Float(n.atan())),
        Value::Complex(n) => Ok(Value::Complex(n.atan())),
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
        Value::Complex(n) => Ok(Value::Complex(n.sinh())),
        _ => Err(anyhow!("Invalid type passed to `sinh` function.")),
    }
}

pub fn cosh(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).cosh())),
        Value::Float(n) => Ok(Value::Float(n.cosh())),
        Value::Complex(n) => Ok(Value::Complex(n.cosh())),
        _ => Err(anyhow!("Invalid type passed to `cosh` function.")),
    }
}

pub fn tanh(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).tanh())),
        Value::Float(n) => Ok(Value::Float(n.tanh())),
        Value::Complex(n) => Ok(Value::Complex(n.tanh())),
        _ => Err(anyhow!("Invalid type passed to `tanh` function.")),
    }
}

pub fn asinh(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).asinh())),
        Value::Float(n) => Ok(Value::Float(n.asinh())),
        Value::Complex(n) => Ok(Value::Complex(n.asinh())),
        _ => Err(anyhow!("Invalid type passed to `asinh` function.")),
    }
}

pub fn acosh(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).acosh())),
        Value::Float(n) => Ok(Value::Float(n.acosh())),
        Value::Complex(n) => Ok(Value::Complex(n.acosh())),
        _ => Err(anyhow!("Invalid type passed to `acosh` function.")),
    }
}

pub fn atanh(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).atanh())),
        Value::Float(n) => Ok(Value::Float(n.atanh())),
        Value::Complex(n) => Ok(Value::Complex(n.atanh())),
        _ => Err(anyhow!("Invalid type passed to `atanh` function.")),
    }
}

pub fn ln(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).ln())),
        Value::Float(n) => Ok(Value::Float(n.ln())),
        Value::Complex(n) => Ok(Value::Complex(n.ln())),
        _ => Err(anyhow!("Invalid type passed to `ln` function.")),
    }
}

pub fn log10(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).log10())),
        Value::Float(n) => Ok(Value::Float(n.log10())),
        Value::Complex(n) => Ok(Value::Complex(n.log10())),
        _ => Err(anyhow!("Invalid type passed to `log10` function.")),
    }
}

pub fn log(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match (&args[0], &args[1]) {
        (Value::Integer(n), Value::Integer(b)) => Ok(Value::Integer(n.ilog(*b) as i32)),
        (Value::Float(n), Value::Float(b)) => Ok(Value::Float(n.log(*b))),
        (Value::Integer(n), Value::Float(b)) => Ok(Value::Float((*n as f32).log(*b))),
        (Value::Float(n), Value::Integer(b)) => Ok(Value::Float(n.log(*b as f32))),
        (Value::Complex(n), Value::Integer(b)) => Ok(Value::Complex(n.log(*b as f32))),
        (Value::Complex(n), Value::Float(b)) => Ok(Value::Complex(n.log(*b))),
        _ => return Err(anyhow!("Invalid type passed to `log` function.")),
    }
}

pub fn abs(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => Ok(Value::Integer(n.abs())),
        Value::Float(n) => Ok(Value::Float(n.abs())),
        Value::Complex(n) => Ok(Value::Float(n.abs())),
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
        Value::Complex(n) => Ok(Value::Complex(n.sqrt())),
        _ => Err(anyhow!("Invalid type passed to `sqrt` function.")),
    }
}

pub fn cbrt(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => Ok(Value::Float((n as f32).cbrt())),
        Value::Float(n) => Ok(Value::Float(n.cbrt())),
        Value::Complex(n) => Ok(Value::Complex(n.cbrt())),
        _ => Err(anyhow!("Invalid type passed to `cbrt` function.")),
    }
}

pub fn fact(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => {
            if n < 0 {
                return Err(anyhow!("Cannot get factorial of negative number."));
            }
            Ok(Value::Integer((n as u32).factorial() as i32))
        }
        Value::Float(n) => {
            if n < 0.0 {
                return Err(anyhow!("Cannot get factorial of negative number."));
            }
            Ok(Value::Integer((n as u32).factorial() as i32))
        }
        _ => Err(anyhow!("Invalid type passed to `fact` function.")),
    }
}

pub fn fact2(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match args[0] {
        Value::Integer(n) => {
            if n < 0 {
                return Err(anyhow!("Cannot get factorial of negative number."));
            }
            Ok(Value::Integer((n as u32).double_factorial() as i32))
        }
        Value::Float(n) => {
            if n < 0.0 {
                return Err(anyhow!("Cannot get factorial of negative number."));
            }
            Ok(Value::Integer((n as u32).double_factorial() as i32))
        }
        _ => Err(anyhow!("Invalid type passed to `fact2` function.")),
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
