#[cfg(feature = "no-std")]
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
