use crate::builtin_function;
use crate::error::{Error, Result};
use crate::interpreter::{Data, Value};

use core::f32::consts::{E, PI, TAU};
use factorial::{DoubleFactorial, Factorial};
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
    [Value::String(s)] => Value::Integer(s.parse().unwrap()),
});

builtin_function!(float => {
    [Value::Integer(n)] => Value::Float(*n as f32),
    [Value::String(s)] => Value::Float(s.parse().unwrap()),
});

builtin_function!(complex => {
    [Value::Float(re), Value::Float(im)] => Value::Complex(Complex::new(*re, *im)),
    [Value::String(s)] => Value::Complex(s.parse().unwrap()),
});

builtin_function!(real => {
    [Value::Complex(c)] => Value::Float(c.re),
});

builtin_function!(imag => {
    [Value::Complex(c)] => Value::Float(c.im),
});

builtin_function!(deg_to_rad => {
    [Value::Integer(n)] => Value::Float((*n as f32).to_radians()),
    [Value::Float(n)] => Value::Float(n.to_radians()),
});

builtin_function!(rad_to_deg => {
    [Value::Integer(n)] => Value::Float((*n as f32).to_degrees()),
    [Value::Float(n)] => Value::Float(n.to_degrees()),
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
            return Err(Error::NegativeNumber);
        }
        Value::Integer(((*n) as u32).factorial() as i32)
    },
    [Value::Float(n)] => {
         if *n < 0.0 {
            return Err(Error::NegativeNumber);
        }
        Value::Integer(((*n) as u32).factorial() as i32)
    },
});

builtin_function!(fact2 => {
    [Value::Integer(n)] => {
        if *n < 0 {
            return Err(Error::NegativeNumber);
        }
        Value::Integer(((*n) as u32).double_factorial() as i32)
    },
    [Value::Float(n)] => {
        if *n < 0.0 {
            return Err(Error::NegativeNumber);
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

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_neg() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        assert_eq!(
            neg(&mut rng, &data, &[Value::Integer(1)]).ok(),
            Some(Value::Integer(-1))
        );
        assert_eq!(
            neg(&mut rng, &data, &[Value::Integer(-1)]).ok(),
            Some(Value::Integer(1))
        );
        assert_eq!(
            neg(&mut rng, &data, &[Value::Float(2.5)]).ok(),
            Some(Value::Float(-2.5))
        );
        assert_eq!(
            neg(&mut rng, &data, &[Value::Float(-2.5)]).ok(),
            Some(Value::Float(2.5))
        );
        assert_eq!(
            neg(&mut rng, &data, &[Value::Complex(Complex::new(3.0, 1.0))]).ok(),
            Some(Value::Complex(Complex::new(-3.0, -1.0)))
        );
        assert_eq!(
            neg(&mut rng, &data, &[Value::Complex(Complex::new(-3.0, -1.0))]).ok(),
            Some(Value::Complex(Complex::new(3.0, 1.0)))
        );
    }

    #[test]
    fn test_bitnot() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        assert_eq!(
            bitnot(&mut rng, &data, &[Value::Integer(0b1010)]).ok(),
            Some(Value::Integer(!0b1010))
        );
        assert_eq!(
            bitnot(&mut rng, &data, &[Value::Float(5.0)]).ok(),
            Some(Value::Float(!5i32 as f32))
        );
    }

    #[test]
    fn test_add() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        assert_eq!(
            add(&mut rng, &data, &[Value::Integer(2), Value::Integer(3)]).ok(),
            Some(Value::Integer(5))
        );
        assert_eq!(
            add(&mut rng, &data, &[Value::Float(2.5), Value::Float(3.5)]).ok(),
            Some(Value::Float(6.0))
        );
        assert_eq!(
            add(
                &mut rng,
                &data,
                &[
                    Value::Complex(Complex::new(1.0, 2.0)),
                    Value::Complex(Complex::new(3.0, 4.0))
                ]
            )
            .ok(),
            Some(Value::Complex(Complex::new(4.0, 6.0)))
        );
        assert_eq!(
            add(&mut rng, &data, &[Value::Integer(2), Value::Float(3.5)]).ok(),
            Some(Value::Float(5.5))
        );
        assert_eq!(
            add(&mut rng, &data, &[Value::Float(2.5), Value::Integer(3)]).ok(),
            Some(Value::Float(5.5))
        );
        assert_eq!(
            add(
                &mut rng,
                &data,
                &[Value::Complex(Complex::new(1.0, 2.0)), Value::Integer(3)]
            )
            .ok(),
            Some(Value::Complex(Complex::new(4.0, 2.0)))
        );
    }

    #[test]
    fn test_sub() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        assert_eq!(
            sub(&mut rng, &data, &[Value::Integer(5), Value::Integer(3)]).ok(),
            Some(Value::Integer(2))
        );
        assert_eq!(
            sub(&mut rng, &data, &[Value::Float(5.5), Value::Float(3.5)]).ok(),
            Some(Value::Float(2.0))
        );
        assert_eq!(
            sub(
                &mut rng,
                &data,
                &[
                    Value::Complex(Complex::new(4.0, 6.0)),
                    Value::Complex(Complex::new(3.0, 4.0))
                ]
            )
            .ok(),
            Some(Value::Complex(Complex::new(1.0, 2.0)))
        );
        assert_eq!(
            sub(&mut rng, &data, &[Value::Integer(5), Value::Float(3.5)]).ok(),
            Some(Value::Float(1.5))
        );
        assert_eq!(
            sub(&mut rng, &data, &[Value::Float(5.5), Value::Integer(3)]).ok(),
            Some(Value::Float(2.5))
        );
    }

    #[test]
    fn test_mul() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        assert_eq!(
            mul(&mut rng, &data, &[Value::Integer(2), Value::Integer(3)]).ok(),
            Some(Value::Integer(6))
        );
        assert_eq!(
            mul(&mut rng, &data, &[Value::Float(2.5), Value::Float(4.0)]).ok(),
            Some(Value::Float(10.0))
        );
        assert_eq!(
            mul(
                &mut rng,
                &data,
                &[
                    Value::Complex(Complex::new(1.0, 2.0)),
                    Value::Complex(Complex::new(3.0, 4.0))
                ]
            )
            .ok(),
            Some(Value::Complex(Complex::new(-5.0, 10.0)))
        );
        assert_eq!(
            mul(&mut rng, &data, &[Value::Integer(2), Value::Float(3.5)]).ok(),
            Some(Value::Float(7.0))
        );
    }

    #[test]
    fn test_div() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        assert_eq!(
            div(&mut rng, &data, &[Value::Integer(6), Value::Integer(3)]).ok(),
            Some(Value::Integer(2))
        );
        assert_eq!(
            div(&mut rng, &data, &[Value::Float(10.0), Value::Float(4.0)]).ok(),
            Some(Value::Float(2.5))
        );
        assert_eq!(
            div(
                &mut rng,
                &data,
                &[
                    Value::Complex(Complex::new(-5.0, 10.0)),
                    Value::Complex(Complex::new(1.0, 2.0))
                ]
            )
            .ok(),
            Some(Value::Complex(Complex::new(3.0, 4.0)))
        );
        assert_eq!(
            div(&mut rng, &data, &[Value::Integer(7), Value::Float(2.0)]).ok(),
            Some(Value::Float(3.5))
        );
    }

    #[test]
    fn test_modulo() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        assert_eq!(
            modulo(&mut rng, &data, &[Value::Integer(7), Value::Integer(3)]).ok(),
            Some(Value::Integer(1))
        );
        assert_eq!(
            modulo(&mut rng, &data, &[Value::Float(7.5), Value::Float(3.0)]).ok(),
            Some(Value::Float(1.5))
        );
        assert_eq!(
            modulo(&mut rng, &data, &[Value::Integer(7), Value::Float(3.0)]).ok(),
            Some(Value::Float(1.0))
        );
    }

    #[test]
    fn test_pow() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        assert_eq!(
            pow(&mut rng, &data, &[Value::Integer(2), Value::Integer(3)]).ok(),
            Some(Value::Float(8.0))
        );
        assert_eq!(
            pow(&mut rng, &data, &[Value::Float(2.5), Value::Float(2.0)]).ok(),
            Some(Value::Float(6.25))
        );
        assert_eq!(
            pow(&mut rng, &data, &[Value::Integer(2), Value::Float(3.0)]).ok(),
            Some(Value::Float(8.0))
        );
    }

    #[test]
    fn test_bitand() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        assert_eq!(
            bitand(
                &mut rng,
                &data,
                &[Value::Integer(0b1010), Value::Integer(0b1100)]
            )
            .ok(),
            Some(Value::Integer(0b1000))
        );
    }

    #[test]
    fn test_bitor() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        assert_eq!(
            bitor(
                &mut rng,
                &data,
                &[Value::Integer(0b1010), Value::Integer(0b1100)]
            )
            .ok(),
            Some(Value::Integer(0b1110))
        );
    }

    #[test]
    fn test_bitxor() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        assert_eq!(
            bitxor(
                &mut rng,
                &data,
                &[Value::Integer(0b1010), Value::Integer(0b1100)]
            )
            .ok(),
            Some(Value::Integer(0b0110))
        );
    }

    #[test]
    fn test_bitleft() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        assert_eq!(
            bitleft(
                &mut rng,
                &data,
                &[Value::Integer(0b1010), Value::Integer(2)]
            )
            .ok(),
            Some(Value::Integer(0b101000))
        );
    }

    #[test]
    fn test_bitright() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        assert_eq!(
            bitright(
                &mut rng,
                &data,
                &[Value::Integer(0b101000), Value::Integer(2)]
            )
            .ok(),
            Some(Value::Integer(0b1010))
        );
    }

    #[test]
    fn test_int() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        assert_eq!(
            int(&mut rng, &data, &[Value::Float(3.7)]).ok(),
            Some(Value::Integer(3))
        );
        assert_eq!(
            int(&mut rng, &data, &[Value::String("42".into())]).ok(),
            Some(Value::Integer(42))
        );
    }

    #[test]
    fn test_float() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        assert_eq!(
            float(&mut rng, &data, &[Value::Integer(3)]).ok(),
            Some(Value::Float(3.0))
        );
        assert_eq!(
            float(&mut rng, &data, &[Value::String("3.14".into())]).ok(),
            Some(Value::Float(3.14))
        );
    }

    #[test]
    fn test_complex() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        assert_eq!(
            complex(&mut rng, &data, &[Value::Float(1.0), Value::Float(2.0)]).ok(),
            Some(Value::Complex(Complex::new(1.0, 2.0)))
        );
        assert_eq!(
            complex(&mut rng, &data, &[Value::String("1+2i".into())]).ok(),
            Some(Value::Complex(Complex::new(1.0, 2.0)))
        );
    }

    #[test]
    fn test_real_imag() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        let c = Value::Complex(Complex::new(1.0, 2.0));
        assert_eq!(
            real(&mut rng, &data, &[c.clone()]).ok(),
            Some(Value::Float(1.0))
        );
        assert_eq!(imag(&mut rng, &data, &[c]).ok(), Some(Value::Float(2.0)));
    }

    #[test]
    fn test_deg_rad_conversion() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        assert_eq!(
            deg_to_rad(&mut rng, &data, &[Value::Integer(180)]).ok(),
            Some(Value::Float(PI))
        );
        assert_eq!(
            rad_to_deg(&mut rng, &data, &[Value::Float(PI)]).ok(),
            Some(Value::Float(180.0))
        );
    }

    #[test]
    fn test_constants() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        assert_eq!(pi(&mut rng, &data, &[]).ok(), Some(Value::Float(PI)));
        assert_eq!(tau(&mut rng, &data, &[]).ok(), Some(Value::Float(TAU)));
        assert_eq!(e(&mut rng, &data, &[]).ok(), Some(Value::Float(E)));
        assert_eq!(phi(&mut rng, &data, &[]).ok(), Some(Value::Float(PHI)));
    }

    #[test]
    fn test_trig_functions() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        let angle = Value::Float(PI / 4.0);

        // sin(π/4) = √2/2 ≈ 0.7071067811865475
        assert!(
            if let Some(Value::Float(result)) = sin(&mut rng, &data, &[angle.clone()]).ok() {
                (result - 0.7071067811865475).abs() < 1e-6
            } else {
                false
            }
        );

        // cos(π/4) = √2/2 ≈ 0.7071067811865475
        assert!(
            if let Some(Value::Float(result)) = cos(&mut rng, &data, &[angle.clone()]).ok() {
                (result - 0.7071067811865475).abs() < 1e-6
            } else {
                false
            }
        );

        // tan(π/4) = 1
        assert!(
            if let Some(Value::Float(result)) = tan(&mut rng, &data, &[angle]).ok() {
                (result - 1.0).abs() < 1e-6
            } else {
                false
            }
        );
    }

    #[test]
    fn test_inverse_trig_functions() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        let value = Value::Float(0.5);

        // asin(0.5) = π/6 ≈ 0.5235987755982988
        assert!(
            if let Some(Value::Float(result)) = asin(&mut rng, &data, &[value.clone()]).ok() {
                (result - 0.5235987755982988).abs() < 1e-6
            } else {
                false
            }
        );

        // acos(0.5) = π/3 ≈ 1.0471975511965976
        assert!(
            if let Some(Value::Float(result)) = acos(&mut rng, &data, &[value.clone()]).ok() {
                (result - 1.0471975511965976).abs() < 1e-6
            } else {
                false
            }
        );

        // atan(0.5) ≈ 0.4636476090008061
        assert!(
            if let Some(Value::Float(result)) = atan(&mut rng, &data, &[value]).ok() {
                (result - 0.4636476090008061).abs() < 1e-6
            } else {
                false
            }
        );
    }

    #[test]
    fn test_hyperbolic_functions() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        let value = Value::Float(1.0);

        // sinh(1) ≈ 1.1752011936438014
        assert!(
            if let Some(Value::Float(result)) = sinh(&mut rng, &data, &[value.clone()]).ok() {
                (result - 1.1752011936438014).abs() < 1e-6
            } else {
                false
            }
        );

        // cosh(1) ≈ 1.543080634815244
        assert!(
            if let Some(Value::Float(result)) = cosh(&mut rng, &data, &[value.clone()]).ok() {
                (result - 1.543080634815244).abs() < 1e-6
            } else {
                false
            }
        );

        // tanh(1) ≈ 0.7615941559557649
        assert!(
            if let Some(Value::Float(result)) = tanh(&mut rng, &data, &[value]).ok() {
                (result - 0.7615941559557649).abs() < 1e-6
            } else {
                false
            }
        );
    }

    #[test]
    fn test_logarithmic_functions() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // ln(e) = 1
        assert!(
            if let Some(Value::Float(result)) = ln(&mut rng, &data, &[Value::Float(E)]).ok() {
                (result - 1.0).abs() < 1e-6
            } else {
                false
            }
        );

        // log10(100) = 2
        assert!(if let Some(Value::Float(result)) =
            log10(&mut rng, &data, &[Value::Integer(100)]).ok()
        {
            (result - 2.0).abs() < 1e-6
        } else {
            false
        });

        // log2(8) = 3 (using log with base 2)
        assert_eq!(
            log(&mut rng, &data, &[Value::Integer(2), Value::Integer(8)]).ok(),
            Some(Value::Integer(3))
        );
    }

    #[test]
    fn test_abs() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        assert_eq!(
            abs(&mut rng, &data, &[Value::Integer(-5)]).ok(),
            Some(Value::Integer(5))
        );
        assert_eq!(
            abs(&mut rng, &data, &[Value::Float(-3.5)]).ok(),
            Some(Value::Float(3.5))
        );
        assert!(if let Some(Value::Float(result)) =
            abs(&mut rng, &data, &[Value::Complex(Complex::new(3.0, 4.0))]).ok()
        {
            (result - 5.0).abs() < 1e-6
        } else {
            false
        });
    }

    #[test]
    fn test_floor_ceil() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        assert_eq!(
            floor(&mut rng, &data, &[Value::Float(3.7)]).ok(),
            Some(Value::Integer(3))
        );
        assert_eq!(
            ceil(&mut rng, &data, &[Value::Float(3.2)]).ok(),
            Some(Value::Integer(4))
        );
    }

    #[test]
    fn test_sqrt_cbrt() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        assert!(if let Some(Value::Float(result)) =
            sqrt(&mut rng, &data, &[Value::Integer(25)]).ok()
        {
            (result - 5.0).abs() < 1e-6
        } else {
            false
        });
        assert!(if let Some(Value::Float(result)) =
            cbrt(&mut rng, &data, &[Value::Integer(27)]).ok()
        {
            (result - 3.0).abs() < 1e-6
        } else {
            false
        });
    }

    #[test]
    fn test_factorials() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        assert_eq!(
            fact(&mut rng, &data, &[Value::Integer(5)]).ok(),
            Some(Value::Integer(120))
        );
        assert_eq!(
            fact2(&mut rng, &data, &[Value::Integer(6)]).ok(),
            Some(Value::Integer(48)) // 6!! = 6*4*2 = 48
        );

        // Test negative numbers (should return error)
        assert!(fact(&mut rng, &data, &[Value::Integer(-1)]).is_err());
        assert!(fact2(&mut rng, &data, &[Value::Float(-1.0)]).is_err());
    }

    #[test]
    fn test_min_max() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        assert_eq!(
            min(&mut rng, &data, &[Value::Integer(3), Value::Integer(5)]).ok(),
            Some(Value::Integer(3))
        );
        assert_eq!(
            max(&mut rng, &data, &[Value::Float(3.5), Value::Float(5.5)]).ok(),
            Some(Value::Float(5.5))
        );
        assert_eq!(
            min(&mut rng, &data, &[Value::Integer(3), Value::Float(5.5)]).ok(),
            Some(Value::Float(3.0))
        );
    }
}
