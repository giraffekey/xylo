use crate::builtin_function;
use crate::error::{Error, Result};
use crate::interpreter::{Data, Value};

use rand_chacha::ChaCha8Rng;

builtin_function!(pipe => {
    [arg, Value::Function(name, argc, pre_args)] => {
        let mut pre_args = pre_args.clone();
        pre_args.push(arg.clone());
        pre_args.reverse();
        Value::Function(name.clone(), argc - 1, pre_args)
    },
});

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "no-std")]
    use alloc::vec;

    use rand::SeedableRng;

    #[test]
    fn test_pipe_basic() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // Create a simple function that adds 1
        let add_one = Value::Function(
            "add_one".into(),
            1,
            vec![Value::Integer(1)], // Pre-loaded argument
        );

        // Pipe 5 through add_one
        let result = pipe(&mut rng, &data, &[Value::Integer(5), add_one.clone()]).unwrap();

        // Should return a new function with arity reduced by 1
        if let Value::Function(name, argc, pre_args) = result {
            assert_eq!(name, "add_one");
            assert_eq!(argc, 0); // Original arity was 1, now 0 after piping
            assert_eq!(pre_args, vec![Value::Integer(5), Value::Integer(1)]);
        } else {
            panic!("pipe did not return a function");
        }
    }

    #[test]
    fn test_pipe_multiple_args() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // Create a function that adds two numbers (arity 2)
        let add = Value::Function(
            "add".into(),
            2,
            vec![], // No pre-loaded arguments
        );

        // Pipe 5 through add (partially applied)
        let result = pipe(&mut rng, &data, &[Value::Integer(5), add.clone()]).unwrap();

        // Should return a new function with arity reduced by 1
        if let Value::Function(name, argc, pre_args) = result {
            assert_eq!(name, "add");
            assert_eq!(argc, 1); // Original arity was 2, now 1 after piping
            assert_eq!(pre_args, vec![Value::Integer(5)]);
        } else {
            panic!("pipe did not return a function");
        }
    }

    #[test]
    fn test_pipe_with_pre_args() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // Create a function with pre-loaded arguments
        let func = Value::Function(
            "multiply".into(),
            3,                       // Total arity
            vec![Value::Integer(2)], // One pre-loaded argument
        );

        // Pipe 3 through the function
        let result = pipe(&mut rng, &data, &[Value::Integer(3), func.clone()]).unwrap();

        // Should return a new function with arity reduced by 1
        if let Value::Function(name, argc, pre_args) = result {
            assert_eq!(name, "multiply");
            assert_eq!(argc, 2); // Original arity was 3, now 2 after piping
            assert_eq!(pre_args, vec![Value::Integer(3), Value::Integer(2)]);
        } else {
            panic!("pipe did not return a function");
        }
    }

    #[test]
    fn test_pipe_invalid_args() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // Test with non-function as second argument
        assert!(pipe(&mut rng, &data, &[Value::Integer(1), Value::Integer(2)]).is_err());

        // Test with wrong number of arguments
        assert!(pipe(&mut rng, &data, &[Value::Integer(1)]).is_err());

        assert!(pipe(
            &mut rng,
            &data,
            &[
                Value::Integer(1),
                Value::Function("test".into(), 1, vec![]),
                Value::Integer(3)
            ]
        )
        .is_err());
    }

    #[test]
    fn test_pipe_chaining() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // Create a simple function that adds two numbers
        let add = Value::Function(
            "add".into(),
            2,
            vec![], // No pre-loaded arguments
        );

        // Pipe 5 through add (partial application)
        let partial = pipe(&mut rng, &data, &[Value::Integer(5), add.clone()]).unwrap();

        // Pipe 10 through the partially applied function
        let result = pipe(&mut rng, &data, &[Value::Integer(10), partial]).unwrap();

        // Should now have all arguments needed
        if let Value::Function(name, argc, pre_args) = result {
            assert_eq!(name, "add");
            assert_eq!(argc, 0); // All arguments provided
            assert_eq!(pre_args, vec![Value::Integer(10), Value::Integer(5)]);
        } else {
            panic!("pipe did not return a function");
        }
    }
}
