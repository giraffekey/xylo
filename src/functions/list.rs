#[cfg(feature = "no-std")]
use alloc::vec::Vec;

use crate::builtin_function;
use crate::interpreter::Value;

use anyhow::{anyhow, Result};
use rand_chacha::ChaCha8Rng;

builtin_function!(range => {
    [Value::Integer(from), Value::Integer(to)] => {
        Value::List((*from..*to).map(Value::Integer).collect())
    },
    [Value::Float(from), Value::Float(to)] => {
        Value::List((*from as i32..*to as i32).map(|i| Value::Float(i as f32)).collect())
    }
});

builtin_function!(rangei => {
    [Value::Integer(from), Value::Integer(to)] => {
        Value::List((*from..=*to).map(Value::Integer).collect())
    },
    [Value::Float(from), Value::Float(to)] => {
        Value::List((*from as i32..=*to as i32).map(|i| Value::Float(i as f32)).collect())
    }
});

builtin_function!(concat => {
    [Value::List(a), Value::List(b)] => {
        {
            let mut list = Vec::with_capacity(a.len() + b.len());
            list.extend(a.clone());
            list.extend(b.clone());

            let value = Value::List(list);
            value.kind()?;
            value
        }
    }
});

builtin_function!(prepend => {
    [a, Value::List(b)] => {
        {
            let mut list = Vec::with_capacity(b.len() + 1);
            list.push(a.clone());
            list.extend(b.clone());

            let value = Value::List(list);
            value.kind()?;
            value
        }
    }
});

builtin_function!(append => {
    [Value::List(a), b] => {
        {
            let mut list = Vec::with_capacity(a.len() + 1);
            list.extend(a.clone());
            list.push(b.clone());

            let value = Value::List(list);
            value.kind()?;
            value
        }
    }
});
