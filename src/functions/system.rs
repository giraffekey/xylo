use crate::builtin_function;
use crate::error::{Error, Result};
use crate::interpreter::{Data, Value};

use rand_chacha::ChaCha8Rng;

builtin_function!(width data => {
    [] => |data: &Data| Ok(Value::Integer(data.dimensions.0 as i32)),
});

builtin_function!(height data => {
    [] => |data: &Data| Ok(Value::Integer(data.dimensions.1 as i32)),
});
