use crate::builtin_function;
use crate::error::{Error, Result};
use crate::interpreter::Value;

use noise::Perlin;
use rand_chacha::ChaCha8Rng;

builtin_function!(pipe => {
    [arg, Value::Function(name, argc, pre_args)] => {
        let mut pre_args = pre_args.clone();
        pre_args.push(arg.clone());
        pre_args.reverse();
        Value::Function(name.clone(), argc - 1, pre_args)
    },
});
