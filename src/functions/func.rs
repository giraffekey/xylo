use crate::interpreter::Value;

use anyhow::{anyhow, Result};
use rand_chacha::ChaCha8Rng;

pub fn pipe(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let arg = args[0].clone();

    match args[1].clone() {
        Value::Function(name, argc, mut pre_args) => {
            pre_args.push(arg);
            pre_args.reverse();
            Ok(Value::Function(name, argc - 1, pre_args))
        }
        _ => Err(anyhow!("Invalid type passed to `pipe` function.")),
    }
}
