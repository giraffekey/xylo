#[cfg(feature = "no-std")]
use alloc::vec::Vec;

use crate::interpreter::Value;

use anyhow::{anyhow, Result};
use rand_chacha::ChaCha8Rng;

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

pub fn concat(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match (&args[0], &args[1]) {
        (Value::List(a), Value::List(b)) => {
            let mut list = Vec::with_capacity(a.len() + b.len());
            list.extend(a.clone());
            list.extend(b.clone());

            let value = Value::List(list);
            value.kind()?;
            Ok(value)
        }
        _ => Err(anyhow!("Invalid type passed to `concat` function.")),
    }
}

pub fn prepend(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match (&args[0], &args[1]) {
        (a, Value::List(b)) => {
            let mut list = Vec::with_capacity(b.len() + 1);
            list.push(a.clone());
            list.extend(b.clone());

            let value = Value::List(list);
            value.kind()?;
            Ok(value)
        }
        _ => Err(anyhow!("Invalid type passed to `prepend` function.")),
    }
}

pub fn append(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match (&args[0], &args[1]) {
        (Value::List(a), b) => {
            let mut list = Vec::with_capacity(a.len() + 1);
            list.extend(a.clone());
            list.push(b.clone());

            let value = Value::List(list);
            value.kind()?;
            Ok(value)
        }
        _ => Err(anyhow!("Invalid type passed to `append` function.")),
    }
}
