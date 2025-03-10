#[cfg(feature = "no-std")]
use alloc::vec::Vec;

use crate::builtin_function;
use crate::interpreter::Value;

use anyhow::{anyhow, Result};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

builtin_function!(rand rng => {
    [] => |rng: &mut ChaCha8Rng| Value::Float(rng.random()),
});

builtin_function!(randi rng => {
    [] => |rng: &mut ChaCha8Rng| {
        if rng.random() {
            Value::Integer(1)
        } else {
            Value::Integer(0)
        }
    },
});

builtin_function!(rand_range rng => {
    [Value::Integer(from), Value::Integer(to)] => |rng: &mut ChaCha8Rng| {
        let a = *from as f32;
        let b = *to as f32;
        Value::Float(rng.random_range(a..b))
    },
    [Value::Float(from), Value::Float(to)] => |rng: &mut ChaCha8Rng| {
        Value::Float(rng.random_range(*from..*to))
    },
    [Value::Integer(from), Value::Float(to)] => |rng: &mut ChaCha8Rng| {
        let a = *from as f32;
        Value::Float(rng.random_range(a..*to))
    },
    [Value::Float(from), Value::Integer(to)] => |rng: &mut ChaCha8Rng| {
        let b = *to as f32;
        Value::Float(rng.random_range(*from..b))
    }
});

builtin_function!(randi_range rng => {
    [Value::Integer(a), Value::Integer(b)] => |rng: &mut ChaCha8Rng| {
        Value::Integer(rng.random_range(*a..*b))
    },
    [Value::Float(a), Value::Float(b)] => |rng: &mut ChaCha8Rng| {
        Value::Integer(rng.random_range((*a as i32)..(*b as i32)))
    },
    [Value::Integer(a), Value::Float(b)] => |rng: &mut ChaCha8Rng| {
        Value::Integer(rng.random_range(*a..(*b as i32)))
    },
    [Value::Float(a), Value::Integer(b)] => |rng: &mut ChaCha8Rng| {
        Value::Integer(rng.random_range((*a as i32)..*b))
    }
});

builtin_function!(rand_rangei rng => {
    [Value::Integer(from), Value::Integer(to)] => |rng: &mut ChaCha8Rng| {
        let a = *from as f32;
        let b = *to as f32;
        Value::Float(rng.random_range(a..=b))
    },
    [Value::Float(from), Value::Float(to)] => |rng: &mut ChaCha8Rng| {
        Value::Float(rng.random_range(*from..=*to))
    },
    [Value::Integer(from), Value::Float(to)] => |rng: &mut ChaCha8Rng| {
        let a = *from as f32;
        Value::Float(rng.random_range(a..=*to))
    },
    [Value::Float(from), Value::Integer(to)] => |rng: &mut ChaCha8Rng| {
        let b = *to as f32;
        Value::Float(rng.random_range(*from..=b))
    }
});

builtin_function!(randi_rangei rng => {
    [Value::Integer(a), Value::Integer(b)] => |rng: &mut ChaCha8Rng| {
        Value::Integer(rng.random_range(*a..=*b))
    },
    [Value::Float(a), Value::Float(b)] => |rng: &mut ChaCha8Rng| {
        Value::Integer(rng.random_range((*a as i32)..=(*b as i32)))
    },
    [Value::Integer(a), Value::Float(b)] => |rng: &mut ChaCha8Rng| {
        Value::Integer(rng.random_range(*a..=(*b as i32)))
    },
    [Value::Float(a), Value::Integer(b)] => |rng: &mut ChaCha8Rng| {
        Value::Integer(rng.random_range((*a as i32)..=*b))
    }
});

builtin_function!(shuffle rng => {
    [Value::List(list)] => |rng: &mut ChaCha8Rng| {
        let mut list = list.clone();
        list.shuffle(rng);
        Value::List(list)
    }
});

builtin_function!(choose rng => {
    [Value::List(list)] => |rng: &mut ChaCha8Rng| {
        list.choose(rng).unwrap().clone()
    }
});
