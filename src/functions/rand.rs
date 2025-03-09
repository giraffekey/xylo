use crate::interpreter::Value;

use anyhow::{anyhow, Result};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

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
