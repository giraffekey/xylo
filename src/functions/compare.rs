#[cfg(feature = "no-std")]
use alloc::vec::Vec;

use crate::builtin_function;
use crate::interpreter::Value;

use anyhow::{anyhow, Result};
use rand_chacha::ChaCha8Rng;

builtin_function!(not => {
    [Value::Boolean(b)] => Value::Boolean(!b)
});

builtin_function!(eq => {
    [Value::Integer(a), Value::Integer(b)] => Value::Boolean(a == b),
    [Value::Float(a), Value::Float(b)] => Value::Boolean(a == b),
    [Value::Complex(a), Value::Complex(b)] => Value::Boolean(a == b),
    [Value::Integer(a), Value::Float(b)] | [Value::Float(b), Value::Integer(a)] => Value::Boolean((*a as f32) == *b),
    [Value::Boolean(a), Value::Boolean(b)] => Value::Boolean(a == b),
});

builtin_function!(neq => {
    [Value::Integer(a), Value::Integer(b)] => Value::Boolean(a != b),
    [Value::Float(a), Value::Float(b)] => Value::Boolean(a != b),
    [Value::Complex(a), Value::Complex(b)] => Value::Boolean(a != b),
    [Value::Integer(a), Value::Float(b)] | [Value::Float(b), Value::Integer(a)] => Value::Boolean((*a as f32) != *b),
});

builtin_function!(lt => {
    [Value::Integer(a), Value::Integer(b)] => Value::Boolean(a < b),
    [Value::Float(a), Value::Float(b)] => Value::Boolean(a < b),
    [Value::Integer(a), Value::Float(b)] => Value::Boolean((*a as f32) < *b),
    [Value::Float(a), Value::Integer(b)] => Value::Boolean(*a < *b as f32)
});

builtin_function!(lte => {
    [Value::Integer(a), Value::Integer(b)] => Value::Boolean(a <= b),
    [Value::Float(a), Value::Float(b)] => Value::Boolean(a <= b),
    [Value::Integer(a), Value::Float(b)] => Value::Boolean((*a as f32) <= *b),
    [Value::Float(a), Value::Integer(b)] => Value::Boolean(*a <= *b as f32)
});

builtin_function!(gt => {
    [Value::Integer(a), Value::Integer(b)] => Value::Boolean(a > b),
    [Value::Float(a), Value::Float(b)] => Value::Boolean(a > b),
    [Value::Integer(a), Value::Float(b)] => Value::Boolean((*a as f32) > *b),
    [Value::Float(a), Value::Integer(b)] => Value::Boolean(*a > *b as f32)
});

builtin_function!(gte => {
    [Value::Integer(a), Value::Integer(b)] => Value::Boolean(a >= b),
    [Value::Float(a), Value::Float(b)] => Value::Boolean(a >= b),
    [Value::Integer(a), Value::Float(b)] => Value::Boolean((*a as f32) >= *b),
    [Value::Float(a), Value::Integer(b)] => Value::Boolean(*a >= *b as f32)
});

builtin_function!(and => {
    [Value::Boolean(a), Value::Boolean(b)] => Value::Boolean(*a && *b)
});

builtin_function!(or => {
    [Value::Boolean(a), Value::Boolean(b)] => Value::Boolean(*a || *b)
});
