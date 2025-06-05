use crate::builtin_function;
use crate::error::{Error, Result};
use crate::interpreter::{Data, Value};

use noise::NoiseFn;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

builtin_function!(rand rng => {
    [] => |rng: &mut ChaCha8Rng| Ok(Value::Float(rng.random())),
});

builtin_function!(randi rng => {
    [] => |rng: &mut ChaCha8Rng| {
        if rng.random() {
            Ok(Value::Integer(1))
        } else {
            Ok(Value::Integer(0))
        }
    },
});

builtin_function!(rand_range rng => {
    [Value::Integer(from), Value::Integer(to)] => |rng: &mut ChaCha8Rng| {
        let a = *from as f32;
        let b = *to as f32;
        Ok(Value::Float(rng.random_range(a..b)))
    },
    [Value::Float(from), Value::Float(to)] => |rng: &mut ChaCha8Rng| {
        Ok(Value::Float(rng.random_range(*from..*to)))
    },
    [Value::Integer(from), Value::Float(to)] => |rng: &mut ChaCha8Rng| {
        let a = *from as f32;
        Ok(Value::Float(rng.random_range(a..*to)))
    },
    [Value::Float(from), Value::Integer(to)] => |rng: &mut ChaCha8Rng| {
        let b = *to as f32;
        Ok(Value::Float(rng.random_range(*from..b)))
    }
});

builtin_function!(rand_rangei rng => {
    [Value::Integer(from), Value::Integer(to)] => |rng: &mut ChaCha8Rng| {
        let a = *from as f32;
        let b = *to as f32;
        Ok(Value::Float(rng.random_range(a..=b)))
    },
    [Value::Float(from), Value::Float(to)] => |rng: &mut ChaCha8Rng| {
        Ok(Value::Float(rng.random_range(*from..=*to)))
    },
    [Value::Integer(from), Value::Float(to)] => |rng: &mut ChaCha8Rng| {
        let a = *from as f32;
        Ok(Value::Float(rng.random_range(a..=*to)))
    },
    [Value::Float(from), Value::Integer(to)] => |rng: &mut ChaCha8Rng| {
        let b = *to as f32;
        Ok(Value::Float(rng.random_range(*from..=b)))
    }
});

builtin_function!(randi_range rng => {
    [Value::Integer(a), Value::Integer(b)] => |rng: &mut ChaCha8Rng| {
        Ok(Value::Integer(rng.random_range(*a..*b)))
    },
    [Value::Float(a), Value::Float(b)] => |rng: &mut ChaCha8Rng| {
        Ok(Value::Integer(rng.random_range((*a as i32)..(*b as i32))))
    },
    [Value::Integer(a), Value::Float(b)] => |rng: &mut ChaCha8Rng| {
        Ok(Value::Integer(rng.random_range(*a..(*b as i32))))
    },
    [Value::Float(a), Value::Integer(b)] => |rng: &mut ChaCha8Rng| {
        Ok(Value::Integer(rng.random_range((*a as i32)..*b)))
    }
});

builtin_function!(randi_rangei rng => {
    [Value::Integer(a), Value::Integer(b)] => |rng: &mut ChaCha8Rng| {
        Ok(Value::Integer(rng.random_range(*a..=*b)))
    },
    [Value::Float(a), Value::Float(b)] => |rng: &mut ChaCha8Rng| {
        Ok(Value::Integer(rng.random_range((*a as i32)..=(*b as i32))))
    },
    [Value::Integer(a), Value::Float(b)] => |rng: &mut ChaCha8Rng| {
        Ok(Value::Integer(rng.random_range(*a..=(*b as i32))))
    },
    [Value::Float(a), Value::Integer(b)] => |rng: &mut ChaCha8Rng| {
        Ok(Value::Integer(rng.random_range((*a as i32)..=*b)))
    }
});

builtin_function!(shuffle rng => {
    [Value::List(list)] => |rng: &mut ChaCha8Rng| {
        let mut list = list.clone();
        list.shuffle(rng);
        Ok(Value::List(list))
    }
});

builtin_function!(choose rng => {
    [Value::List(list)] => |rng: &mut ChaCha8Rng| {
        Ok(list.choose(rng).unwrap().clone())
    }
});

builtin_function!(noise1 data => {
    [a] => |data: &Data| {
        let a = match a {
            Value::Integer(a) => *a as f64,
            Value::Float(a) => *a as f64,
            _ => return Err(Error::InvalidArgument("noise1".into())),
        };

        Ok(Value::Float(data.perlin.get([a]) as f32))
    }
});

builtin_function!(noise2 data => {
    [a, b] => |data: &Data| {
        let a = match a {
            Value::Integer(a) => *a as f64,
            Value::Float(a) => *a as f64,
            _ => return Err(Error::InvalidArgument("noise2".into())),
        };

        let b = match b {
            Value::Integer(b) => *b as f64,
            Value::Float(b) => *b as f64,
            _ => return Err(Error::InvalidArgument("noise2".into())),
        };

        Ok(Value::Float(data.perlin.get([a, b]) as f32))
    }
});

builtin_function!(noise3 data => {
    [a, b, c] => |data: &Data| {
        let a = match a {
            Value::Integer(a) => *a as f64,
            Value::Float(a) => *a as f64,
            _ => return Err(Error::InvalidArgument("noise3".into())),
        };

        let b = match b {
            Value::Integer(b) => *b as f64,
            Value::Float(b) => *b as f64,
            _ => return Err(Error::InvalidArgument("noise3".into())),
        };

        let c = match c {
            Value::Integer(c) => *c as f64,
            Value::Float(c) => *c as f64,
            _ => return Err(Error::InvalidArgument("noise3".into())),
        };

        Ok(Value::Float(data.perlin.get([a, b, c]) as f32))
    }
});

builtin_function!(noise4 data => {
    [a, b, c, d] => |data: &Data| {
        let a = match a {
            Value::Integer(a) => *a as f64,
            Value::Float(a) => *a as f64,
            _ => return Err(Error::InvalidArgument("noise4".into())),
        };

        let b = match b {
            Value::Integer(b) => *b as f64,
            Value::Float(b) => *b as f64,
            _ => return Err(Error::InvalidArgument("noise4".into())),
        };

        let c = match c {
            Value::Integer(c) => *c as f64,
            Value::Float(c) => *c as f64,
            _ => return Err(Error::InvalidArgument("noise4".into())),
        };

        let d = match d {
            Value::Integer(d) => *d as f64,
            Value::Float(d) => *d as f64,
            _ => return Err(Error::InvalidArgument("noise4".into())),
        };

        Ok(Value::Float(data.perlin.get([a, b, c, d]) as f32))
    }
});
