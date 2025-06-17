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
        if a >= b {
            return Err(Error::InvalidRange);
        }
        Ok(Value::Float(rng.random_range(a..b)))
    },
    [Value::Float(from), Value::Float(to)] => |rng: &mut ChaCha8Rng| {
        if from >= to {
            return Err(Error::InvalidRange);
        }
        Ok(Value::Float(rng.random_range(*from..*to)))
    },
    [Value::Integer(from), Value::Float(to)] => |rng: &mut ChaCha8Rng| {
        let a = *from as f32;
        if a >= *to {
            return Err(Error::InvalidRange);
        }
        Ok(Value::Float(rng.random_range(a..*to)))
    },
    [Value::Float(from), Value::Integer(to)] => |rng: &mut ChaCha8Rng| {
        let b = *to as f32;
        if *from >= b {
            return Err(Error::InvalidRange);
        }
        Ok(Value::Float(rng.random_range(*from..b)))
    }
});

builtin_function!(rand_rangei rng => {
    [Value::Integer(from), Value::Integer(to)] => |rng: &mut ChaCha8Rng| {
        let a = *from as f32;
        let b = *to as f32;
        if a > b {
            return Err(Error::InvalidRange);
        }
        Ok(Value::Float(rng.random_range(a..=b)))
    },
    [Value::Float(from), Value::Float(to)] => |rng: &mut ChaCha8Rng| {
        if from > to {
            return Err(Error::InvalidRange);
        }
        Ok(Value::Float(rng.random_range(*from..=*to)))
    },
    [Value::Integer(from), Value::Float(to)] => |rng: &mut ChaCha8Rng| {
        let a = *from as f32;
        if a > *to {
            return Err(Error::InvalidRange);
        }
        Ok(Value::Float(rng.random_range(a..=*to)))
    },
    [Value::Float(from), Value::Integer(to)] => |rng: &mut ChaCha8Rng| {
        let b = *to as f32;
        if *from > b {
            return Err(Error::InvalidRange);
        }
        Ok(Value::Float(rng.random_range(*from..=b)))
    }
});

builtin_function!(randi_range rng => {
    [Value::Integer(a), Value::Integer(b)] => |rng: &mut ChaCha8Rng| {
        if a >= b {
            return Err(Error::InvalidRange);
        }
        Ok(Value::Integer(rng.random_range(*a..*b)))
    },
    [Value::Float(a), Value::Float(b)] => |rng: &mut ChaCha8Rng| {
        if a >= b {
            return Err(Error::InvalidRange);
        }
        Ok(Value::Integer(rng.random_range((*a as i32)..(*b as i32))))
    },
    [Value::Integer(a), Value::Float(b)] => |rng: &mut ChaCha8Rng| {
        if *a as f32 >= *b {
            return Err(Error::InvalidRange);
        }
        Ok(Value::Integer(rng.random_range(*a..(*b as i32))))
    },
    [Value::Float(a), Value::Integer(b)] => |rng: &mut ChaCha8Rng| {
        if *a >= *b as f32 {
            return Err(Error::InvalidRange);
        }
        Ok(Value::Integer(rng.random_range((*a as i32)..*b)))
    }
});

builtin_function!(randi_rangei rng => {
    [Value::Integer(a), Value::Integer(b)] => |rng: &mut ChaCha8Rng| {
        if a > b {
            return Err(Error::InvalidRange);
        }
        Ok(Value::Integer(rng.random_range(*a..=*b)))
    },
    [Value::Float(a), Value::Float(b)] => |rng: &mut ChaCha8Rng| {
        if a > b {
            return Err(Error::InvalidRange);
        }
        Ok(Value::Integer(rng.random_range((*a as i32)..=(*b as i32))))
    },
    [Value::Integer(a), Value::Float(b)] => |rng: &mut ChaCha8Rng| {
        if *a as f32 > *b {
            return Err(Error::InvalidRange);
        }
        Ok(Value::Integer(rng.random_range(*a..=(*b as i32))))
    },
    [Value::Float(a), Value::Integer(b)] => |rng: &mut ChaCha8Rng| {
        if *a > *b as f32 {
            return Err(Error::InvalidRange);
        }
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
        list.choose(rng).cloned().ok_or(Error::InvalidList)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "no-std")]
    use alloc::vec;

    use noise::Perlin;
    use rand::SeedableRng;

    #[test]
    fn test_random_functions() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // Test rand
        for _ in 0..100 {
            if let Ok(Value::Float(n)) = rand(&mut rng, &data, &[]) {
                assert!(n >= 0.0 && n < 1.0);
            } else {
                panic!("rand failed");
            }
        }

        // Test randi
        for _ in 0..100 {
            if let Ok(Value::Integer(n)) = randi(&mut rng, &data, &[]) {
                assert!(n == 0 || n == 1);
            } else {
                panic!("randi failed");
            }
        }

        // Test rand_range with various input types
        let range_tests = vec![
            (Value::Integer(1), Value::Integer(10)), // int, int
            (Value::Float(1.0), Value::Float(10.0)), // float, float
            (Value::Integer(1), Value::Float(10.0)), // int, float
            (Value::Float(1.0), Value::Integer(10)), // float, int
        ];

        for _ in 0..100 {
            for (from, to) in &range_tests {
                if let Ok(Value::Float(n)) =
                    rand_range(&mut rng, &data, &[from.clone(), to.clone()])
                {
                    let from_f = match from {
                        Value::Integer(i) => *i as f32,
                        Value::Float(f) => *f,
                        _ => unreachable!(),
                    };
                    let to_f = match to {
                        Value::Integer(i) => *i as f32,
                        Value::Float(f) => *f,
                        _ => unreachable!(),
                    };
                    assert!(n >= from_f && n < to_f);
                } else {
                    panic!("rand_range failed");
                }
            }
        }

        // Test rand_rangei
        for _ in 0..100 {
            if let Ok(Value::Float(n)) =
                rand_rangei(&mut rng, &data, &[Value::Integer(1), Value::Integer(10)])
            {
                assert!(n >= 1.0 && n <= 10.0);
            } else {
                panic!("rand_rangei failed");
            }
        }

        // Test randi_range with various input types
        for _ in 0..100 {
            for (from, to) in &range_tests {
                if let Ok(Value::Integer(n)) =
                    randi_range(&mut rng, &data, &[from.clone(), to.clone()])
                {
                    let from_i = match from {
                        Value::Integer(i) => *i,
                        Value::Float(f) => *f as i32,
                        _ => unreachable!(),
                    };
                    let to_i = match to {
                        Value::Integer(i) => *i,
                        Value::Float(f) => *f as i32,
                        _ => unreachable!(),
                    };
                    assert!(n >= from_i && n < to_i);
                } else {
                    panic!("randi_range failed");
                }
            }
        }

        // Test randi_rangei
        for _ in 0..100 {
            if let Ok(Value::Integer(n)) =
                randi_rangei(&mut rng, &data, &[Value::Integer(1), Value::Integer(10)])
            {
                assert!(n >= 1 && n <= 10);
            } else {
                panic!("randi_rangei failed");
            }
        }

        // Test shuffle
        let original = vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
        ];
        if let Ok(Value::List(shuffled)) =
            shuffle(&mut rng, &data, &[Value::List(original.clone())])
        {
            assert_eq!(shuffled.len(), 4);
            assert_ne!(shuffled, original);
            // Check all elements are still present
            for item in original {
                assert!(shuffled.contains(&item));
            }
        } else {
            panic!("shuffle failed");
        }

        // Test choose
        let list = Value::List(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        if let Ok(chosen) = choose(&mut rng, &data, &[list]) {
            assert!(matches!(chosen, Value::Integer(1 | 2 | 3)));
        } else {
            panic!("choose failed");
        }
    }

    #[test]
    fn test_noise_functions() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // Test noise1 with various input types
        let noise1_tests = vec![Value::Integer(42), Value::Float(3.14)];

        for input in noise1_tests {
            if let Ok(Value::Float(n)) = noise1(&mut rng, &data, &[input]) {
                assert!(n >= -1.0 && n <= 1.0);
            } else {
                panic!("noise1 failed");
            }
        }

        // Test noise2
        let inputs = vec![
            (Value::Integer(1), Value::Integer(2)),
            (Value::Float(1.0), Value::Float(2.0)),
            (Value::Integer(1), Value::Float(2.0)),
            (Value::Float(1.0), Value::Integer(2)),
        ];

        for (a, b) in inputs {
            if let Ok(Value::Float(n)) = noise2(&mut rng, &data, &[a, b]) {
                assert!(n >= -1.0 && n <= 1.0);
            } else {
                panic!("noise2 failed");
            }
        }

        // Test noise3
        if let Ok(Value::Float(n)) = noise3(
            &mut rng,
            &data,
            &[Value::Integer(1), Value::Integer(2), Value::Integer(3)],
        ) {
            assert!(n >= -1.0 && n <= 1.0);
        } else {
            panic!("noise3 failed");
        }

        // Test noise4
        if let Ok(Value::Float(n)) = noise4(
            &mut rng,
            &data,
            &[
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3),
                Value::Integer(4),
            ],
        ) {
            assert!(n >= -1.0 && n <= 1.0);
        } else {
            panic!("noise4 failed");
        }

        // Test invalid inputs
        assert!(noise1(&mut rng, &data, &[Value::String("invalid".into())]).is_err());
        assert!(noise2(
            &mut rng,
            &data,
            &[Value::Integer(1), Value::String("invalid".into())]
        )
        .is_err());
    }

    #[test]
    fn test_edge_cases() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data {
            perlin: Perlin::new(0),
            ..Default::default()
        };

        // Test empty list for choose
        assert!(choose(&mut rng, &data, &[Value::List(vec![])]).is_err());

        // Test equal bounds for ranges
        assert!(rand_range(&mut rng, &data, &[Value::Integer(5), Value::Integer(5)]).is_err());
        assert_eq!(
            rand_rangei(&mut rng, &data, &[Value::Integer(5), Value::Integer(5)]).ok(),
            Some(Value::Float(5.0))
        );
        assert!(randi_range(&mut rng, &data, &[Value::Integer(5), Value::Integer(5)]).is_err());
        assert_eq!(
            randi_rangei(&mut rng, &data, &[Value::Integer(5), Value::Integer(5)]).ok(),
            Some(Value::Integer(5))
        );

        // Test reverse bounds
        assert!(rand_range(&mut rng, &data, &[Value::Integer(10), Value::Integer(5)]).is_err());
        assert!(rand_rangei(&mut rng, &data, &[Value::Integer(10), Value::Integer(5)]).is_err());
        assert!(randi_range(&mut rng, &data, &[Value::Integer(10), Value::Integer(5)]).is_err());
        assert!(randi_rangei(&mut rng, &data, &[Value::Integer(10), Value::Integer(5)]).is_err());
    }
}
