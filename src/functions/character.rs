#[cfg(feature = "alloc")]
use alloc::string::ToString;

use crate::builtin_function;
use crate::error::{Error, Result};
use crate::interpreter::{Data, Value};

use rand_chacha::ChaCha8Rng;

builtin_function!(is_control => {
    [Value::Char(c)] => {
        Value::Boolean(c.is_ascii_control())
    }
});

builtin_function!(is_uppercase => {
    [Value::Char(c)] => {
        Value::Boolean(c.is_ascii_uppercase())
    }
});

builtin_function!(is_lowercase => {
    [Value::Char(c)] => {
        Value::Boolean(c.is_ascii_lowercase())
    }
});

builtin_function!(is_alpha => {
    [Value::Char(c)] => {
        Value::Boolean(c.is_ascii_alphabetic())
    }
});

builtin_function!(is_alphanumeric => {
    [Value::Char(c)] => {
        Value::Boolean(c.is_ascii_alphanumeric())
    }
});

builtin_function!(is_digit => {
    [Value::Char(c)] => {
        Value::Boolean(c.is_ascii_digit())
    }
});

// builtin_function!(is_oct_digit => {
//     [Value::Char(c)] => {
//         Value::Boolean(c.is_ascii_octdigit())
//     }
// });

builtin_function!(is_hex_digit => {
    [Value::Char(c)] => {
        Value::Boolean(c.is_ascii_hexdigit())
    }
});

builtin_function!(is_numeric => {
    [Value::Char(c)] => {
        Value::Boolean(c.is_numeric())
    }
});

builtin_function!(is_punctuation => {
    [Value::Char(c)] => {
        Value::Boolean(c.is_ascii_punctuation())
    }
});

builtin_function!(is_whitespace => {
    [Value::Char(c)] => {
        Value::Boolean(c.is_ascii_whitespace())
    }
});

builtin_function!(is_ascii => {
    [Value::Char(c)] => {
        Value::Boolean(c.is_ascii())
    }
});

builtin_function!(to_uppercase => {
    [Value::Char(c)] => {
        Value::Char(c.to_ascii_uppercase())
    }
});

builtin_function!(to_lowercase => {
    [Value::Char(c)] => {
        Value::Char(c.to_ascii_lowercase())
    }
});

builtin_function!(digit_to_int => {
    [Value::Char(c)] => {
        match c.to_digit(10) {
            Some(n) => Value::Integer(n as i32),
            None => return Err(Error::NotDigit("digit_to_int".into()))
        }
    }
});

builtin_function!(int_to_digit => {
    [Value::Integer(n)] => {
        match char::from_digit(*n as u32, 10) {
            Some(c) => Value::Char(c),
            None => return Err(Error::NotDigit("int_to_digit".into()))
        }
    }
});

builtin_function!(words => {
    [Value::String(s)] => {
        Value::List(s.split_whitespace().map(|word| Value::String(word.into())).collect())
    }
});

builtin_function!(lines => {
    [Value::String(s)] => {
        Value::List(s.lines().map(|line| Value::String(line.into())).collect())
    }
});

builtin_function!(string => {
    [Value::Integer(n)] => Value::String(n.to_string()),
    [Value::Float(n)] => Value::String(n.to_string()),
    [Value::Complex(n)] => Value::String(n.to_string()),
    [Value::Char(c)] => Value::String(c.to_string()),
});

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "alloc")]
    use alloc::vec;

    use num::Complex;
    use rand::SeedableRng;

    #[test]
    fn test_char_predicates() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // is_control
        assert_eq!(
            is_control(&mut rng, &data, &[Value::Char('\n')]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            is_control(&mut rng, &data, &[Value::Char('a')]).ok(),
            Some(Value::Boolean(false))
        );

        // is_uppercase
        assert_eq!(
            is_uppercase(&mut rng, &data, &[Value::Char('A')]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            is_uppercase(&mut rng, &data, &[Value::Char('a')]).ok(),
            Some(Value::Boolean(false))
        );

        // is_lowercase
        assert_eq!(
            is_lowercase(&mut rng, &data, &[Value::Char('a')]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            is_lowercase(&mut rng, &data, &[Value::Char('A')]).ok(),
            Some(Value::Boolean(false))
        );

        // is_alpha
        assert_eq!(
            is_alpha(&mut rng, &data, &[Value::Char('a')]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            is_alpha(&mut rng, &data, &[Value::Char('1')]).ok(),
            Some(Value::Boolean(false))
        );

        // is_alphanumeric
        assert_eq!(
            is_alphanumeric(&mut rng, &data, &[Value::Char('a')]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            is_alphanumeric(&mut rng, &data, &[Value::Char('1')]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            is_alphanumeric(&mut rng, &data, &[Value::Char('!')]).ok(),
            Some(Value::Boolean(false))
        );

        // is_digit
        assert_eq!(
            is_digit(&mut rng, &data, &[Value::Char('9')]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            is_digit(&mut rng, &data, &[Value::Char('a')]).ok(),
            Some(Value::Boolean(false))
        );

        // is_hex_digit
        assert_eq!(
            is_hex_digit(&mut rng, &data, &[Value::Char('a')]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            is_hex_digit(&mut rng, &data, &[Value::Char('F')]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            is_hex_digit(&mut rng, &data, &[Value::Char('g')]).ok(),
            Some(Value::Boolean(false))
        );

        // is_numeric
        assert_eq!(
            is_numeric(&mut rng, &data, &[Value::Char('Ⅷ')]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            is_numeric(&mut rng, &data, &[Value::Char('a')]).ok(),
            Some(Value::Boolean(false))
        );

        // is_punctuation
        assert_eq!(
            is_punctuation(&mut rng, &data, &[Value::Char('!')]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            is_punctuation(&mut rng, &data, &[Value::Char('a')]).ok(),
            Some(Value::Boolean(false))
        );

        // is_whitespace
        assert_eq!(
            is_whitespace(&mut rng, &data, &[Value::Char(' ')]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            is_whitespace(&mut rng, &data, &[Value::Char('a')]).ok(),
            Some(Value::Boolean(false))
        );

        // is_ascii
        assert_eq!(
            is_ascii(&mut rng, &data, &[Value::Char('a')]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            is_ascii(&mut rng, &data, &[Value::Char('ñ')]).ok(),
            Some(Value::Boolean(false))
        );
    }

    #[test]
    fn test_char_transforms() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // to_uppercase
        assert_eq!(
            to_uppercase(&mut rng, &data, &[Value::Char('a')]).ok(),
            Some(Value::Char('A'))
        );
        assert_eq!(
            to_uppercase(&mut rng, &data, &[Value::Char('A')]).ok(),
            Some(Value::Char('A'))
        );

        // to_lowercase
        assert_eq!(
            to_lowercase(&mut rng, &data, &[Value::Char('A')]).ok(),
            Some(Value::Char('a'))
        );
        assert_eq!(
            to_lowercase(&mut rng, &data, &[Value::Char('a')]).ok(),
            Some(Value::Char('a'))
        );
    }

    #[test]
    fn test_digit_conversions() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // digit_to_int
        assert_eq!(
            digit_to_int(&mut rng, &data, &[Value::Char('5')]).ok(),
            Some(Value::Integer(5))
        );
        assert_eq!(
            digit_to_int(&mut rng, &data, &[Value::Char('a')]).ok(),
            None
        );

        // int_to_digit
        assert_eq!(
            int_to_digit(&mut rng, &data, &[Value::Integer(7)]).ok(),
            Some(Value::Char('7'))
        );
        assert_eq!(
            int_to_digit(&mut rng, &data, &[Value::Integer(10)]).ok(),
            None
        );
    }

    #[test]
    fn test_string_operations() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // words
        assert_eq!(
            words(&mut rng, &data, &[Value::String("hello world".into())]).ok(),
            Some(Value::List(vec![
                Value::String("hello".into()),
                Value::String("world".into())
            ]))
        );
        assert_eq!(
            words(&mut rng, &data, &[Value::String("  ".into())]).ok(),
            Some(Value::List(vec![]))
        );

        // lines
        assert_eq!(
            lines(&mut rng, &data, &[Value::String("a\nb\nc".into())]).ok(),
            Some(Value::List(vec![
                Value::String("a".into()),
                Value::String("b".into()),
                Value::String("c".into())
            ]))
        );
        assert_eq!(
            lines(&mut rng, &data, &[Value::String("".into())]).ok(),
            Some(Value::List(vec![]))
        );

        // string conversion
        assert_eq!(
            string(&mut rng, &data, &[Value::Integer(42)]).ok(),
            Some(Value::String("42".into()))
        );
        assert_eq!(
            string(&mut rng, &data, &[Value::Float(3.14)]).ok(),
            Some(Value::String("3.14".into()))
        );
        assert_eq!(
            string(&mut rng, &data, &[Value::Char('x')]).ok(),
            Some(Value::String("x".into()))
        );
        assert_eq!(
            string(&mut rng, &data, &[Value::Complex(Complex::new(1.0, 2.0))]).ok(),
            Some(Value::String("1+2i".into()))
        );
    }
}
