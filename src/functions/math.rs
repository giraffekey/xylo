use crate::builtin_function;
use crate::error::{Error, Result};
use crate::interpreter::Value;

use core::f32::consts::{E, PI, TAU};
use factorial::{DoubleFactorial, Factorial};
use noise::Perlin;
use num::complex::{Complex, ComplexFloat};
use rand_chacha::ChaCha8Rng;

// Defined manually until more_float_constants is stable
const PHI: f32 = 1.618033988749894848204586834365638118_f32;

builtin_function!(neg => {
    [Value::Integer(n)] => Value::Integer(-n),
    [Value::Float(n)] => Value::Float(-n),
    [Value::Complex(n)] => Value::Complex(-n),
});

builtin_function!(bitnot => {
    [Value::Integer(n)] => Value::Integer(!n),
    [Value::Float(n)] => Value::Float(!(*n as i32) as f32),
});

builtin_function!(add => {
    [Value::Integer(a), Value::Integer(b)] => Value::Integer(a + b),
    [Value::Float(a), Value::Float(b)] => Value::Float(a + b),
    [Value::Complex(a), Value::Complex(b)] => Value::Complex(a + b),
    [Value::Integer(a), Value::Float(b)] | [Value::Float(b), Value::Integer(a)] => Value::Float(*a as f32 + b),
    [Value::Complex(a), Value::Integer(b)] | [Value::Integer(b), Value::Complex(a)] => Value::Complex(a + (*b as f32)),
    [Value::Complex(a), Value::Float(b)] | [Value::Float(b), Value::Complex(a)] => Value::Complex(a + b)
});

builtin_function!(sub => {
    [Value::Integer(a), Value::Integer(b)] => Value::Integer(a - b),
    [Value::Float(a), Value::Float(b)] => Value::Float(a - b),
    [Value::Complex(a), Value::Complex(b)] => Value::Complex(a - b),
    [Value::Integer(a), Value::Float(b)] => Value::Float(*a as f32 - b),
    [Value::Float(a), Value::Integer(b)] => Value::Float(a - *b as f32),
    [Value::Complex(a), Value::Integer(b)] => Value::Complex(a - *b as f32),
    [Value::Integer(a), Value::Complex(b)] => Value::Complex(*a as f32 - b),
    [Value::Complex(a), Value::Float(b)] => Value::Complex(a - b),
    [Value::Float(a), Value::Complex(b)] => Value::Complex(a - b)
});

builtin_function!(mul => {
    [Value::Integer(a), Value::Integer(b)] => Value::Integer(a * b),
    [Value::Float(a), Value::Float(b)] => Value::Float(a * b),
    [Value::Complex(a), Value::Complex(b)] => Value::Complex(a * b),
    [Value::Integer(a), Value::Float(b)] | [Value::Float(b), Value::Integer(a)] => Value::Float(*a as f32 * b),
    [Value::Complex(a), Value::Integer(b)] | [Value::Integer(b), Value::Complex(a)] => Value::Complex(a * (*b as f32)),
    [Value::Complex(a), Value::Float(b)] | [Value::Float(b), Value::Complex(a)] => Value::Complex(a * b)
});

builtin_function!(div => {
    [Value::Integer(a), Value::Integer(b)] => Value::Integer(a / b),
    [Value::Float(a), Value::Float(b)] => Value::Float(a / b),
    [Value::Complex(a), Value::Complex(b)] => Value::Complex(a / b),
    [Value::Integer(a), Value::Float(b)] => Value::Float(*a as f32 / b),
    [Value::Float(a), Value::Integer(b)] => Value::Float(a / *b as f32),
    [Value::Complex(a), Value::Integer(b)] => Value::Complex(a / *b as f32),
    [Value::Integer(a), Value::Complex(b)] => Value::Complex((*a as f32) / b),
    [Value::Complex(a), Value::Float(b)] => Value::Complex(a / b),
    [Value::Float(a), Value::Complex(b)] => Value::Complex((*a) / b)
});

builtin_function!(modulo => {
    [Value::Integer(a), Value::Integer(b)] => Value::Integer(a % b),
    [Value::Float(a), Value::Float(b)] => Value::Float(a % b),
    [Value::Complex(a), Value::Complex(b)] => Value::Complex(a % b),
    [Value::Integer(a), Value::Float(b)] | [Value::Float(b), Value::Integer(a)] => Value::Float(*a as f32 % b),
    [Value::Complex(a), Value::Integer(b)] | [Value::Integer(b), Value::Complex(a)] => Value::Complex(a % (*b as f32)),
    [Value::Complex(a), Value::Float(b)] | [Value::Float(b), Value::Complex(a)] => Value::Complex(a % b)
});

builtin_function!(pow => {
    [Value::Integer(a), Value::Integer(b)] => Value::Float((*a as f32).powf(*b as f32)),
    [Value::Float(a), Value::Float(b)] => Value::Float(a.powf(*b)),
    [Value::Complex(a), Value::Complex(b)] => Value::Complex(a.powc(*b)),
    [Value::Integer(a), Value::Float(b)] | [Value::Float(b), Value::Integer(a)] => Value::Float((*a as f32).powf(*b)),
    [Value::Complex(a), Value::Integer(b)] | [Value::Integer(b), Value::Complex(a)] => Value::Complex(a.powf(*b as f32)),
    [Value::Complex(a), Value::Float(b)] | [Value::Float(b), Value::Complex(a)] => Value::Complex(a.powf(*b))
});

builtin_function!(bitand => {
    [Value::Integer(a), Value::Integer(b)] => Value::Integer(a & b),
});

builtin_function!(bitor => {
    [Value::Integer(a), Value::Integer(b)] => Value::Integer(a | b),
});

builtin_function!(bitxor => {
    [Value::Integer(a), Value::Integer(b)] => Value::Integer(a ^ b),
});

builtin_function!(bitleft => {
    [Value::Integer(a), Value::Integer(b)] => Value::Integer(a << b),
});

builtin_function!(bitright => {
    [Value::Integer(a), Value::Integer(b)] => Value::Integer(a >> b),
});

builtin_function!(int => {
    [Value::Float(n)] => Value::Integer(*n as i32),
});

builtin_function!(float => {
    [Value::Integer(n)] => Value::Float(*n as f32),
});

builtin_function!(complex => {
    [Value::Float(re), Value::Float(im)] => Value::Complex(Complex::new(*re, *im)),
});

builtin_function!(real => {
    [Value::Complex(c)] => Value::Float(c.re),
});

builtin_function!(imag => {
    [Value::Complex(c)] => Value::Float(c.im),
});

builtin_function!(pi => {
    [] => Value::Float(PI),
});

builtin_function!(tau => {
    [] => Value::Float(TAU),
});

builtin_function!(e => {
    [] => Value::Float(E),
});

builtin_function!(phi => {
    [] => Value::Float(PHI),
});

builtin_function!(sin => {
    [Value::Integer(n)] => Value::Float((*n as f32).sin()),
    [Value::Float(n)] => Value::Float(n.sin()),
    [Value::Complex(n)] => Value::Complex(n.sin())
});

builtin_function!(cos => {
    [Value::Integer(n)] => Value::Float((*n as f32).cos()),
    [Value::Float(n)] => Value::Float(n.cos()),
    [Value::Complex(n)] => Value::Complex(n.cos())
});

builtin_function!(tan => {
    [Value::Integer(n)] => Value::Float((*n as f32).tan()),
    [Value::Float(n)] => Value::Float(n.tan()),
    [Value::Complex(n)] => Value::Complex(n.tan())
});

builtin_function!(asin => {
    [Value::Integer(n)] => Value::Float((*n as f32).asin()),
    [Value::Float(n)] => Value::Float(n.asin()),
    [Value::Complex(n)] => Value::Complex(n.asin())
});

builtin_function!(acos => {
    [Value::Integer(n)] => Value::Float((*n as f32).acos()),
    [Value::Float(n)] => Value::Float(n.acos()),
    [Value::Complex(n)] => Value::Complex(n.acos())
});

builtin_function!(atan => {
    [Value::Integer(n)] => Value::Float((*n as f32).atan()),
    [Value::Float(n)] => Value::Float(n.atan()),
    [Value::Complex(n)] => Value::Complex(n.atan())
});

builtin_function!(atan2 => {
    [Value::Integer(y), Value::Integer(x)] => Value::Float((*y as f32).atan2(*x as f32)),
    [Value::Float(y), Value::Float(x)] => Value::Float(y.atan2(*x)),
    [Value::Integer(y), Value::Float(x)] => Value::Float((*y as f32).atan2(*x)),
    [Value::Float(y), Value::Integer(x)] => Value::Float(y.atan2(*x as f32))
});

builtin_function!(sinh => {
    [Value::Integer(n)] => Value::Float((*n as f32).sinh()),
    [Value::Float(n)] => Value::Float(n.sinh()),
    [Value::Complex(n)] => Value::Complex(n.sinh())
});

builtin_function!(cosh => {
    [Value::Integer(n)] => Value::Float((*n as f32).cosh()),
    [Value::Float(n)] => Value::Float(n.cosh()),
    [Value::Complex(n)] => Value::Complex(n.cosh())
});

builtin_function!(tanh => {
    [Value::Integer(n)] => Value::Float((*n as f32).tanh()),
    [Value::Float(n)] => Value::Float(n.tanh()),
    [Value::Complex(n)] => Value::Complex(n.tanh())
});

builtin_function!(asinh => {
    [Value::Integer(n)] => Value::Float((*n as f32).asinh()),
    [Value::Float(n)] => Value::Float(n.asinh()),
    [Value::Complex(n)] => Value::Complex(n.asinh())
});

builtin_function!(acosh => {
    [Value::Integer(n)] => Value::Float((*n as f32).acosh()),
    [Value::Float(n)] => Value::Float(n.acosh()),
    [Value::Complex(n)] => Value::Complex(n.acosh())
});

builtin_function!(atanh => {
    [Value::Integer(n)] => Value::Float((*n as f32).atanh()),
    [Value::Float(n)] => Value::Float(n.atanh()),
    [Value::Complex(n)] => Value::Complex(n.atanh())
});

builtin_function!(ln => {
    [Value::Integer(n)] => Value::Float((*n as f32).ln()),
    [Value::Float(n)] => Value::Float(n.ln()),
    [Value::Complex(n)] => Value::Complex(n.ln())
});

builtin_function!(log10 => {
    [Value::Integer(n)] => Value::Float((*n as f32).log10()),
    [Value::Float(n)] => Value::Float(n.log10()),
    [Value::Complex(n)] => Value::Complex(n.log10())
});

builtin_function!(log => {
    [Value::Integer(b), Value::Integer(n)] => Value::Integer(n.ilog(*b) as i32),
    [Value::Float(b), Value::Integer(n)] => Value::Float((*n as f32).log(*b)),
    [Value::Integer(b), Value::Float(n)] => Value::Float(n.log(*b as f32)),
    [Value::Float(b), Value::Float(n)] => Value::Float(n.log(*b)),
    [Value::Integer(b), Value::Complex(n)] => Value::Complex(n.log(*b as f32)),
    [Value::Float(b), Value::Complex(n)] => Value::Complex(n.log(*b))
});

builtin_function!(abs => {
    [Value::Integer(n)] => Value::Integer(n.abs()),
    [Value::Float(n)] => Value::Float(n.abs()),
    [Value::Complex(n)] => Value::Float(n.abs())
});

builtin_function!(floor => {
    [Value::Integer(n)] => Value::Integer(*n),
    [Value::Float(n)] => Value::Integer(n.floor() as i32)
});

builtin_function!(ceil => {
    [Value::Integer(n)] => Value::Integer(*n),
    [Value::Float(n)] => Value::Integer(n.ceil() as i32)
});

builtin_function!(sqrt => {
    [Value::Integer(n)] => Value::Float((*n as f32).sqrt()),
    [Value::Float(n)] => Value::Float(n.sqrt()),
    [Value::Complex(n)] => Value::Complex(n.sqrt())
});

builtin_function!(cbrt => {
    [Value::Integer(n)] => Value::Float((*n as f32).cbrt()),
    [Value::Float(n)] => Value::Float(n.cbrt()),
    [Value::Complex(n)] => Value::Complex(n.cbrt())
});

builtin_function!(fact => {
    [Value::Integer(n)] => {
        if *n < 0 {
            return Err(Error::NegativeFactorial);
        }
        Value::Integer(((*n) as u32).factorial() as i32)
    },
    [Value::Float(n)] => {
         if *n < 0.0 {
            return Err(Error::NegativeFactorial);
        }
        Value::Integer(((*n) as u32).factorial() as i32)
    },
});

builtin_function!(fact2 => {
    [Value::Integer(n)] => {
        if *n < 0 {
            return Err(Error::NegativeFactorial);
        }
        Value::Integer(((*n) as u32).double_factorial() as i32)
    },
    [Value::Float(n)] => {
        if *n < 0.0 {
            return Err(Error::NegativeFactorial);
        }
        Value::Integer(((*n) as u32).double_factorial() as i32)
    },
});

builtin_function!(min => {
    [Value::Integer(a), Value::Integer(b)] => Value::Integer(*a.min(b)),
    [Value::Float(a), Value::Float(b)] => Value::Float(a.min(*b)),
    [Value::Integer(a), Value::Float(b)] => Value::Float((*a as f32).min(*b)),
    [Value::Float(a), Value::Integer(b)] => Value::Float((*b as f32).min(*a)),
});

builtin_function!(max => {
    [Value::Integer(a), Value::Integer(b)] => Value::Integer(*a.max(b)),
    [Value::Float(a), Value::Float(b)] => Value::Float(a.max(*b)),
    [Value::Integer(a), Value::Float(b)] => Value::Float((*a as f32).max(*b)),
    [Value::Float(a), Value::Integer(b)] => Value::Float((*b as f32).max(*a))
});
