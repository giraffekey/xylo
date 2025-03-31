use crate::builtin_function;
use crate::error::{Error, Result};
use crate::interpreter::Value;

use rand_chacha::ChaCha8Rng;

builtin_function!(not => {
    [Value::Boolean(b)] => Value::Boolean(!b)
});

builtin_function!(eq => {
    [a, b] => Value::Boolean(a == b),
});

builtin_function!(neq => {
    [a, b] => Value::Boolean(a != b),
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
