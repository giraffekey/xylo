use crate::builtin_function;
use crate::error::{Error, Result};
use crate::interpreter::{Data, Value};

use rand_chacha::ChaCha8Rng;

builtin_function!(not => {
    [Value::Boolean(b)] => Value::Boolean(!b)
});

builtin_function!(eq => {
    [a, b] => Value::Boolean(a == b),
});

builtin_function!(neq => {
    [a, b] => Value::Boolean(a != b),
});

builtin_function!(lt => {
    [Value::Integer(a), Value::Integer(b)] => Value::Boolean(a < b),
    [Value::Float(a), Value::Float(b)] => Value::Boolean(a < b),
    [Value::Integer(a), Value::Float(b)] => Value::Boolean((*a as f32) < *b),
    [Value::Float(a), Value::Integer(b)] => Value::Boolean(*a < *b as f32),
    [Value::Char(a), Value::Char(b)] => Value::Boolean(a < b),
});

builtin_function!(lte => {
    [Value::Integer(a), Value::Integer(b)] => Value::Boolean(a <= b),
    [Value::Float(a), Value::Float(b)] => Value::Boolean(a <= b),
    [Value::Integer(a), Value::Float(b)] => Value::Boolean((*a as f32) <= *b),
    [Value::Float(a), Value::Integer(b)] => Value::Boolean(*a <= *b as f32),
    [Value::Char(a), Value::Char(b)] => Value::Boolean(a <= b),
});

builtin_function!(gt => {
    [Value::Integer(a), Value::Integer(b)] => Value::Boolean(a > b),
    [Value::Float(a), Value::Float(b)] => Value::Boolean(a > b),
    [Value::Integer(a), Value::Float(b)] => Value::Boolean((*a as f32) > *b),
    [Value::Float(a), Value::Integer(b)] => Value::Boolean(*a > *b as f32),
    [Value::Char(a), Value::Char(b)] => Value::Boolean(a > b),
});

builtin_function!(gte => {
    [Value::Integer(a), Value::Integer(b)] => Value::Boolean(a >= b),
    [Value::Float(a), Value::Float(b)] => Value::Boolean(a >= b),
    [Value::Integer(a), Value::Float(b)] => Value::Boolean((*a as f32) >= *b),
    [Value::Float(a), Value::Integer(b)] => Value::Boolean(*a >= *b as f32),
    [Value::Char(a), Value::Char(b)] => Value::Boolean(a >= b),
});

builtin_function!(and => {
    [Value::Boolean(a), Value::Boolean(b)] => Value::Boolean(*a && *b)
});

builtin_function!(or => {
    [Value::Boolean(a), Value::Boolean(b)] => Value::Boolean(*a || *b)
});

#[cfg(test)]
mod tests {
    use super::*;
    use num::Complex;
    use rand::SeedableRng;

    #[test]
    fn test_not() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        assert_eq!(
            not(&mut rng, &data, &[Value::Boolean(true)]).ok(),
            Some(Value::Boolean(false))
        );
        assert_eq!(
            not(&mut rng, &data, &[Value::Boolean(false)]).ok(),
            Some(Value::Boolean(true))
        );
    }

    #[test]
    fn test_eq() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // Integer comparisons
        assert_eq!(
            eq(&mut rng, &data, &[Value::Integer(3), Value::Integer(3)]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            eq(&mut rng, &data, &[Value::Integer(3), Value::Integer(2)]).ok(),
            Some(Value::Boolean(false))
        );

        // Float comparisons
        assert_eq!(
            eq(&mut rng, &data, &[Value::Float(3.14), Value::Float(3.14)]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            eq(&mut rng, &data, &[Value::Float(3.14), Value::Float(2.71)]).ok(),
            Some(Value::Boolean(false))
        );

        // Mixed numeric comparisons
        assert_eq!(
            eq(&mut rng, &data, &[Value::Integer(3), Value::Float(3.0)]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            eq(&mut rng, &data, &[Value::Float(3.0), Value::Integer(3)]).ok(),
            Some(Value::Boolean(true))
        );

        // Boolean comparisons
        assert_eq!(
            eq(
                &mut rng,
                &data,
                &[Value::Boolean(true), Value::Boolean(true)]
            )
            .ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            eq(
                &mut rng,
                &data,
                &[Value::Boolean(true), Value::Boolean(false)]
            )
            .ok(),
            Some(Value::Boolean(false))
        );

        // Character comparisons
        assert_eq!(
            eq(&mut rng, &data, &[Value::Char('a'), Value::Char('a')]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            eq(&mut rng, &data, &[Value::Char('a'), Value::Char('b')]).ok(),
            Some(Value::Boolean(false))
        );

        // String comparisons
        assert_eq!(
            eq(
                &mut rng,
                &data,
                &[Value::String("hello".into()), Value::String("hello".into())]
            )
            .ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            eq(
                &mut rng,
                &data,
                &[Value::String("hello".into()), Value::String("world".into())]
            )
            .ok(),
            Some(Value::Boolean(false))
        );

        // Complex comparisons
        let complex1 = Value::Complex(Complex::new(1.0, 2.0));
        let complex2 = Value::Complex(Complex::new(1.0, 2.0));
        let complex3 = Value::Complex(Complex::new(3.0, 4.0));
        assert_eq!(
            eq(&mut rng, &data, &[complex1.clone(), complex2.clone()]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            eq(&mut rng, &data, &[complex1.clone(), complex3.clone()]).ok(),
            Some(Value::Boolean(false))
        );

        // Invalid comparisons
        assert_eq!(
            eq(
                &mut rng,
                &data,
                &[Value::Integer(42), Value::String("42".into())]
            )
            .ok(),
            Some(Value::Boolean(false))
        );
        assert_eq!(
            eq(&mut rng, &data, &[Value::Boolean(true), Value::Integer(1)]).ok(),
            Some(Value::Boolean(false))
        );
    }

    #[test]
    fn test_neq() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // Integer comparisons
        assert_eq!(
            neq(&mut rng, &data, &[Value::Integer(3), Value::Integer(3)]).ok(),
            Some(Value::Boolean(false))
        );
        assert_eq!(
            neq(&mut rng, &data, &[Value::Integer(3), Value::Integer(2)]).ok(),
            Some(Value::Boolean(true))
        );

        // Float comparisons
        assert_eq!(
            neq(&mut rng, &data, &[Value::Float(3.14), Value::Float(3.14)]).ok(),
            Some(Value::Boolean(false))
        );
        assert_eq!(
            neq(&mut rng, &data, &[Value::Float(3.14), Value::Float(2.71)]).ok(),
            Some(Value::Boolean(true))
        );

        // Mixed numeric comparisons
        assert_eq!(
            neq(&mut rng, &data, &[Value::Integer(3), Value::Float(3.0)]).ok(),
            Some(Value::Boolean(false))
        );
        assert_eq!(
            neq(&mut rng, &data, &[Value::Float(3.0), Value::Integer(3)]).ok(),
            Some(Value::Boolean(false))
        );

        // Boolean comparisons
        assert_eq!(
            neq(
                &mut rng,
                &data,
                &[Value::Boolean(true), Value::Boolean(true)]
            )
            .ok(),
            Some(Value::Boolean(false))
        );
        assert_eq!(
            neq(
                &mut rng,
                &data,
                &[Value::Boolean(true), Value::Boolean(false)]
            )
            .ok(),
            Some(Value::Boolean(true))
        );

        // Character comparisons
        assert_eq!(
            neq(&mut rng, &data, &[Value::Char('a'), Value::Char('a')]).ok(),
            Some(Value::Boolean(false))
        );
        assert_eq!(
            neq(&mut rng, &data, &[Value::Char('a'), Value::Char('b')]).ok(),
            Some(Value::Boolean(true))
        );

        // String comparisons
        assert_eq!(
            neq(
                &mut rng,
                &data,
                &[Value::String("hello".into()), Value::String("hello".into())]
            )
            .ok(),
            Some(Value::Boolean(false))
        );
        assert_eq!(
            neq(
                &mut rng,
                &data,
                &[Value::String("hello".into()), Value::String("world".into())]
            )
            .ok(),
            Some(Value::Boolean(true))
        );

        // Complex comparisons
        let complex1 = Value::Complex(Complex::new(1.0, 2.0));
        let complex2 = Value::Complex(Complex::new(1.0, 2.0));
        let complex3 = Value::Complex(Complex::new(3.0, 4.0));
        assert_eq!(
            neq(&mut rng, &data, &[complex1.clone(), complex2.clone()]).ok(),
            Some(Value::Boolean(false))
        );
        assert_eq!(
            neq(&mut rng, &data, &[complex1.clone(), complex3.clone()]).ok(),
            Some(Value::Boolean(true))
        );

        // Invalid comparisons
        assert_eq!(
            neq(
                &mut rng,
                &data,
                &[Value::Integer(42), Value::String("42".into())]
            )
            .ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            neq(&mut rng, &data, &[Value::Boolean(true), Value::Integer(1)]).ok(),
            Some(Value::Boolean(true))
        );
    }

    #[test]
    fn test_lt() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // Integer comparisons
        assert_eq!(
            lt(&mut rng, &data, &[Value::Integer(2), Value::Integer(3)]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            lt(&mut rng, &data, &[Value::Integer(3), Value::Integer(3)]).ok(),
            Some(Value::Boolean(false))
        );
        assert_eq!(
            lt(&mut rng, &data, &[Value::Integer(4), Value::Integer(3)]).ok(),
            Some(Value::Boolean(false))
        );

        // Float comparisons
        assert_eq!(
            lt(&mut rng, &data, &[Value::Float(2.5), Value::Float(3.5)]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            lt(&mut rng, &data, &[Value::Float(3.5), Value::Float(3.5)]).ok(),
            Some(Value::Boolean(false))
        );

        // Mixed integer/float comparisons
        assert_eq!(
            lt(&mut rng, &data, &[Value::Integer(2), Value::Float(3.5)]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            lt(&mut rng, &data, &[Value::Float(2.5), Value::Integer(3)]).ok(),
            Some(Value::Boolean(true))
        );

        // Char comparisons
        assert_eq!(
            lt(&mut rng, &data, &[Value::Char('a'), Value::Char('b')]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            lt(&mut rng, &data, &[Value::Char('b'), Value::Char('a')]).ok(),
            Some(Value::Boolean(false))
        );
    }

    #[test]
    fn test_lte() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // Integer comparisons
        assert_eq!(
            lte(&mut rng, &data, &[Value::Integer(2), Value::Integer(3)]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            lte(&mut rng, &data, &[Value::Integer(3), Value::Integer(3)]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            lte(&mut rng, &data, &[Value::Integer(4), Value::Integer(3)]).ok(),
            Some(Value::Boolean(false))
        );

        // Float comparisons
        assert_eq!(
            lte(&mut rng, &data, &[Value::Float(2.5), Value::Float(3.5)]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            lte(&mut rng, &data, &[Value::Float(3.5), Value::Float(3.5)]).ok(),
            Some(Value::Boolean(true))
        );

        // Mixed integer/float comparisons
        assert_eq!(
            lte(&mut rng, &data, &[Value::Integer(3), Value::Float(3.0)]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            lte(&mut rng, &data, &[Value::Float(3.0), Value::Integer(3)]).ok(),
            Some(Value::Boolean(true))
        );

        // Char comparisons
        assert_eq!(
            lte(&mut rng, &data, &[Value::Char('a'), Value::Char('b')]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            lte(&mut rng, &data, &[Value::Char('b'), Value::Char('a')]).ok(),
            Some(Value::Boolean(false))
        );
        assert_eq!(
            lte(&mut rng, &data, &[Value::Char('a'), Value::Char('a')]).ok(),
            Some(Value::Boolean(true))
        );
    }

    #[test]
    fn test_gt() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // Integer comparisons
        assert_eq!(
            gt(&mut rng, &data, &[Value::Integer(2), Value::Integer(3)]).ok(),
            Some(Value::Boolean(false))
        );
        assert_eq!(
            gt(&mut rng, &data, &[Value::Integer(3), Value::Integer(3)]).ok(),
            Some(Value::Boolean(false))
        );
        assert_eq!(
            gt(&mut rng, &data, &[Value::Integer(4), Value::Integer(3)]).ok(),
            Some(Value::Boolean(true))
        );

        // Float comparisons
        assert_eq!(
            gt(&mut rng, &data, &[Value::Float(2.5), Value::Float(3.5)]).ok(),
            Some(Value::Boolean(false))
        );
        assert_eq!(
            gt(&mut rng, &data, &[Value::Float(3.5), Value::Float(3.5)]).ok(),
            Some(Value::Boolean(false))
        );
        assert_eq!(
            gt(&mut rng, &data, &[Value::Float(4.5), Value::Float(3.5)]).ok(),
            Some(Value::Boolean(true))
        );

        // Mixed integer/float comparisons
        assert_eq!(
            gt(&mut rng, &data, &[Value::Integer(4), Value::Float(3.5)]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            gt(&mut rng, &data, &[Value::Float(4.5), Value::Integer(4)]).ok(),
            Some(Value::Boolean(true))
        );

        // Char comparisons
        assert_eq!(
            gt(&mut rng, &data, &[Value::Char('b'), Value::Char('a')]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            gt(&mut rng, &data, &[Value::Char('a'), Value::Char('b')]).ok(),
            Some(Value::Boolean(false))
        );
    }

    #[test]
    fn test_gte() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // Integer comparisons
        assert_eq!(
            gte(&mut rng, &data, &[Value::Integer(2), Value::Integer(3)]).ok(),
            Some(Value::Boolean(false))
        );
        assert_eq!(
            gte(&mut rng, &data, &[Value::Integer(3), Value::Integer(3)]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            gte(&mut rng, &data, &[Value::Integer(4), Value::Integer(3)]).ok(),
            Some(Value::Boolean(true))
        );

        // Float comparisons
        assert_eq!(
            gte(&mut rng, &data, &[Value::Float(2.5), Value::Float(3.5)]).ok(),
            Some(Value::Boolean(false))
        );
        assert_eq!(
            gte(&mut rng, &data, &[Value::Float(3.5), Value::Float(3.5)]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            gte(&mut rng, &data, &[Value::Float(4.5), Value::Float(3.5)]).ok(),
            Some(Value::Boolean(true))
        );

        // Mixed integer/float comparisons
        assert_eq!(
            gte(&mut rng, &data, &[Value::Integer(3), Value::Float(3.0)]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            gte(&mut rng, &data, &[Value::Float(3.0), Value::Integer(3)]).ok(),
            Some(Value::Boolean(true))
        );

        // Char comparisons
        assert_eq!(
            gte(&mut rng, &data, &[Value::Char('b'), Value::Char('a')]).ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            gte(&mut rng, &data, &[Value::Char('a'), Value::Char('b')]).ok(),
            Some(Value::Boolean(false))
        );
        assert_eq!(
            gte(&mut rng, &data, &[Value::Char('a'), Value::Char('a')]).ok(),
            Some(Value::Boolean(true))
        );
    }

    #[test]
    fn test_and() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // All combinations of boolean values
        assert_eq!(
            and(
                &mut rng,
                &data,
                &[Value::Boolean(true), Value::Boolean(true)]
            )
            .ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            and(
                &mut rng,
                &data,
                &[Value::Boolean(true), Value::Boolean(false)]
            )
            .ok(),
            Some(Value::Boolean(false))
        );
        assert_eq!(
            and(
                &mut rng,
                &data,
                &[Value::Boolean(false), Value::Boolean(true)]
            )
            .ok(),
            Some(Value::Boolean(false))
        );
        assert_eq!(
            and(
                &mut rng,
                &data,
                &[Value::Boolean(false), Value::Boolean(false)]
            )
            .ok(),
            Some(Value::Boolean(false))
        );
    }

    #[test]
    fn test_or() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // All combinations of boolean values
        assert_eq!(
            or(
                &mut rng,
                &data,
                &[Value::Boolean(true), Value::Boolean(true)]
            )
            .ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            or(
                &mut rng,
                &data,
                &[Value::Boolean(true), Value::Boolean(false)]
            )
            .ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            or(
                &mut rng,
                &data,
                &[Value::Boolean(false), Value::Boolean(true)]
            )
            .ok(),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            or(
                &mut rng,
                &data,
                &[Value::Boolean(false), Value::Boolean(false)]
            )
            .ok(),
            Some(Value::Boolean(false))
        );
    }
}
