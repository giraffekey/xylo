use crate::builtin_function;
use crate::error::{Error, Result};
use crate::functions::dedup_shape;
use crate::interpreter::{Data, Value};

use rand_chacha::ChaCha8Rng;

builtin_function!(translate => {
    [Value::Integer(tx), Value::Integer(ty), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().translate(*tx as f32, *ty as f32);
        Value::Shape(shape.clone())
    },
    [Value::Float(tx), Value::Float(ty), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().translate(*tx, *ty);
        Value::Shape(shape.clone())
    },
    [Value::Integer(tx), Value::Float(ty), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().translate(*tx as f32, *ty);
        Value::Shape(shape.clone())
    },
    [Value::Float(tx), Value::Integer(ty), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().translate(*tx, *ty as f32);
        Value::Shape(shape.clone())
    },
});

builtin_function!(translatex => {
    [Value::Integer(tx), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().translate(*tx as f32, 0.0);
        Value::Shape(shape.clone())
    },
    [Value::Float(tx), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().translate(*tx, 0.0);
        Value::Shape(shape.clone())
    },
});

builtin_function!(translatey => {
    [Value::Integer(ty), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().translate(0.0, *ty as f32);
        Value::Shape(shape.clone())
    },
    [Value::Float(ty), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().translate(0.0, *ty);
        Value::Shape(shape.clone())
    },
});

builtin_function!(translateb => {
    [Value::Integer(t), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().translate(*t as f32, *t as f32);
        Value::Shape(shape.clone())
    },
    [Value::Float(t), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().translate(*t, *t);
        Value::Shape(shape.clone())
    },
});

builtin_function!(rotate => {
    [Value::Integer(r), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().rotate(*r as f32);
        Value::Shape(shape.clone())
    },
    [Value::Float(r), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().rotate(*r);
        Value::Shape(shape.clone())
    },
});

builtin_function!(rotate_at => {
    [r, tx, ty, Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        let r = match r {
            Value::Integer(n) => *n as f32,
            Value::Float(n)   => *n,
           _ => return Err(Error::InvalidArgument("rotate_at".into())),
        };
        let tx = match tx {
            Value::Integer(n) => *n as f32,
            Value::Float(n)   => *n,
           _ => return Err(Error::InvalidArgument("rotate_at".into())),
        };
        let ty = match ty {
            Value::Integer(n) => *n as f32,
            Value::Float(n)   => *n,
           _ => return Err(Error::InvalidArgument("rotate_at".into())),
        };
        shape.borrow_mut().rotate_at(r, tx, ty);
        Value::Shape(shape.clone())
    }
});

builtin_function!(scale => {
    [Value::Integer(sx), Value::Integer(sy), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().scale(*sx as f32, *sy as f32);
        Value::Shape(shape.clone())
    },
    [Value::Float(sx), Value::Float(sy), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().scale(*sx, *sy);
        Value::Shape(shape.clone())
    },
    [Value::Integer(sx), Value::Float(sy), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().scale(*sx as f32, *sy);
        Value::Shape(shape.clone())
    },
    [Value::Float(sx), Value::Integer(sy), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().scale(*sx, *sy as f32);
        Value::Shape(shape.clone())
    },
});

builtin_function!(scalex => {
    [Value::Integer(sx), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().scale(*sx as f32, 0.0);
        Value::Shape(shape.clone())
    },
    [Value::Float(sx), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().scale(*sx, 0.0);
        Value::Shape(shape.clone())
    },
});

builtin_function!(scaley => {
    [Value::Integer(sy), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().scale(0.0, *sy as f32);
        Value::Shape(shape.clone())
    },
    [Value::Float(sy), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().scale(0.0, *sy);
        Value::Shape(shape.clone())
    },
});

builtin_function!(scaleb => {
    [Value::Integer(s), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);

        shape.borrow_mut().scale(*s as f32, *s as f32);
        Value::Shape(shape)
    },
    [Value::Float(s), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().scale(*s, *s);
        Value::Shape(shape.clone())
    },
});

builtin_function!(skew => {
    [Value::Integer(kx), Value::Integer(ky), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().skew(*kx as f32, *ky as f32);
        Value::Shape(shape.clone())
    },
    [Value::Float(kx), Value::Float(ky), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().skew(*kx, *ky);
        Value::Shape(shape.clone())
    },
    [Value::Integer(kx), Value::Float(ky), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().skew(*kx as f32, *ky);
        Value::Shape(shape.clone())
    },
    [Value::Float(kx), Value::Integer(ky), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().skew(*kx, *ky as f32);
        Value::Shape(shape.clone())
    },
});

builtin_function!(skewx => {
    [Value::Integer(kx), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().skew(*kx as f32, 0.0);
        Value::Shape(shape.clone())
    },
    [Value::Float(kx), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().skew(*kx, 0.0);
        Value::Shape(shape.clone())
    },
});

builtin_function!(skewy => {
    [Value::Integer(ky), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().skew(0.0, *ky as f32);
        Value::Shape(shape.clone())
    },
    [Value::Float(ky), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().skew(0.0, *ky);
        Value::Shape(shape.clone())
    },
});

builtin_function!(skewb => {
    [Value::Integer(k), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().skew(*k as f32, *k as f32);
        Value::Shape(shape.clone())
    },
    [Value::Float(k), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().skew(*k, *k);
        Value::Shape(shape.clone())
    },
});

builtin_function!(flip => {
    [Value::Integer(f), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().flip(*f as f32);
        Value::Shape(shape.clone())
    },
    [Value::Float(f), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().flip(*f);
        Value::Shape(shape.clone())
    },
});

builtin_function!(fliph => {
    [Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().fliph();
        Value::Shape(shape.clone())
    },
});

builtin_function!(flipv => {
    [Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().flipv();
        Value::Shape(shape.clone())
    },
});

builtin_function!(flipd => {
    [Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().flipd();
        Value::Shape(shape.clone())
    },
});

builtin_function!(zindex => {
    [Value::Integer(z), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().set_zindex(*z as f32);
        Value::Shape(shape.clone())
    },
    [Value::Float(z), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().set_zindex(*z);
        Value::Shape(shape.clone())
    },
});

builtin_function!(zshift => {
    [Value::Integer(z), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().shift_zindex(*z as f32);
        Value::Shape(shape.clone())
    },
    [Value::Float(z), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().shift_zindex(*z);
        Value::Shape(shape.clone())
    },
});

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "std")]
    use std::rc::Rc;

    #[cfg(feature = "no-std")]
    use alloc::{rc::Rc, vec};

    use crate::shape::{PathSegment, Shape};
    use core::cell::RefCell;
    use rand::SeedableRng;

    #[test]
    fn test_translation_functions() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // Create a test shape
        let shape = Rc::new(RefCell::new(Shape::path(vec![
            PathSegment::MoveTo(0.0, 0.0),
            PathSegment::LineTo(10.0, 10.0),
        ])));

        // Test translate with various input types
        let translate_tests = vec![
            (Value::Integer(5), Value::Integer(10)), // int, int
            (Value::Float(5.0), Value::Float(10.0)), // float, float
            (Value::Integer(5), Value::Float(10.0)), // int, float
            (Value::Float(5.0), Value::Integer(10)), // float, int
        ];

        for (tx, ty) in translate_tests {
            let result =
                translate(&mut rng, &data, &[tx, ty, Value::Shape(shape.clone())]).unwrap();

            assert!(matches!(result, Value::Shape(_)));
        }

        // Test translatex
        let translated_x = translatex(
            &mut rng,
            &data,
            &[Value::Integer(5), Value::Shape(shape.clone())],
        )
        .unwrap();
        assert!(matches!(translated_x, Value::Shape(_)));

        // Test translatey
        let translated_y = translatey(
            &mut rng,
            &data,
            &[Value::Integer(5), Value::Shape(shape.clone())],
        )
        .unwrap();
        assert!(matches!(translated_y, Value::Shape(_)));

        // Test translateb
        let translated_both = translateb(
            &mut rng,
            &data,
            &[Value::Integer(5), Value::Shape(shape.clone())],
        )
        .unwrap();
        assert!(matches!(translated_both, Value::Shape(_)));
    }

    #[test]
    fn test_rotation_functions() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // Create a test shape
        let shape = Rc::new(RefCell::new(Shape::path(vec![
            PathSegment::MoveTo(0.0, 0.0),
            PathSegment::LineTo(10.0, 10.0),
        ])));

        // Test basic rotation
        let rotated = rotate(
            &mut rng,
            &data,
            &[Value::Integer(45), Value::Shape(shape.clone())],
        )
        .unwrap();
        assert!(matches!(rotated, Value::Shape(_)));

        // Test rotation at point
        let rotated_at = rotate_at(
            &mut rng,
            &data,
            &[
                Value::Integer(45),
                Value::Integer(5),
                Value::Integer(5),
                Value::Shape(shape.clone()),
            ],
        )
        .unwrap();
        assert!(matches!(rotated_at, Value::Shape(_)));
    }

    #[test]
    fn test_scale_functions() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // Create a test shape
        let shape = Rc::new(RefCell::new(Shape::path(vec![
            PathSegment::MoveTo(0.0, 0.0),
            PathSegment::LineTo(10.0, 10.0),
        ])));

        // Test scale with various input types
        let scale_tests = vec![
            (Value::Integer(2), Value::Integer(3)), // int, int
            (Value::Float(2.0), Value::Float(3.0)), // float, float
            (Value::Integer(2), Value::Float(3.0)), // int, float
            (Value::Float(2.0), Value::Integer(3)), // float, int
        ];

        for (sx, sy) in scale_tests {
            let result = scale(&mut rng, &data, &[sx, sy, Value::Shape(shape.clone())]).unwrap();

            assert!(matches!(result, Value::Shape(_)));
        }

        // Test scalex
        let scaled_x = scalex(
            &mut rng,
            &data,
            &[Value::Integer(2), Value::Shape(shape.clone())],
        )
        .unwrap();
        assert!(matches!(scaled_x, Value::Shape(_)));

        // Test scaley
        let scaled_y = scaley(
            &mut rng,
            &data,
            &[Value::Integer(2), Value::Shape(shape.clone())],
        )
        .unwrap();
        assert!(matches!(scaled_y, Value::Shape(_)));

        // Test scaleb
        let scaled_both = scaleb(
            &mut rng,
            &data,
            &[Value::Integer(2), Value::Shape(shape.clone())],
        )
        .unwrap();
        assert!(matches!(scaled_both, Value::Shape(_)));
    }

    #[test]
    fn test_skew_functions() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // Create a test shape
        let shape = Rc::new(RefCell::new(Shape::path(vec![
            PathSegment::MoveTo(0.0, 0.0),
            PathSegment::LineTo(10.0, 10.0),
        ])));

        // Test skew with various input types
        let skew_tests = vec![
            (Value::Integer(10), Value::Integer(20)), // int, int
            (Value::Float(10.0), Value::Float(20.0)), // float, float
            (Value::Integer(10), Value::Float(20.0)), // int, float
            (Value::Float(10.0), Value::Integer(20)), // float, int
        ];

        for (kx, ky) in skew_tests {
            let result = skew(&mut rng, &data, &[kx, ky, Value::Shape(shape.clone())]).unwrap();

            assert!(matches!(result, Value::Shape(_)));
        }

        // Test skewx
        let skewed_x = skewx(
            &mut rng,
            &data,
            &[Value::Integer(10), Value::Shape(shape.clone())],
        )
        .unwrap();
        assert!(matches!(skewed_x, Value::Shape(_)));

        // Test skewy
        let skewed_y = skewy(
            &mut rng,
            &data,
            &[Value::Integer(10), Value::Shape(shape.clone())],
        )
        .unwrap();
        assert!(matches!(skewed_y, Value::Shape(_)));

        // Test skewb
        let skewed_both = skewb(
            &mut rng,
            &data,
            &[Value::Integer(10), Value::Shape(shape.clone())],
        )
        .unwrap();
        assert!(matches!(skewed_both, Value::Shape(_)));
    }

    #[test]
    fn test_flip_functions() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // Create a test shape
        let shape = Rc::new(RefCell::new(Shape::path(vec![
            PathSegment::MoveTo(0.0, 0.0),
            PathSegment::LineTo(10.0, 10.0),
        ])));

        // Test basic flip
        let flipped = flip(
            &mut rng,
            &data,
            &[Value::Integer(45), Value::Shape(shape.clone())],
        )
        .unwrap();
        assert!(matches!(flipped, Value::Shape(_)));

        // Test fliph
        let flipped_h = fliph(&mut rng, &data, &[Value::Shape(shape.clone())]).unwrap();
        assert!(matches!(flipped_h, Value::Shape(_)));

        // Test flipv
        let flipped_v = flipv(&mut rng, &data, &[Value::Shape(shape.clone())]).unwrap();
        assert!(matches!(flipped_v, Value::Shape(_)));

        // Test flipd
        let flipped_d = flipd(&mut rng, &data, &[Value::Shape(shape.clone())]).unwrap();
        assert!(matches!(flipped_d, Value::Shape(_)));
    }

    #[test]
    fn test_zindex_functions() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // Create a test shape
        let shape = Rc::new(RefCell::new(Shape::path(vec![
            PathSegment::MoveTo(0.0, 0.0),
            PathSegment::LineTo(10.0, 10.0),
        ])));

        // Test zindex
        let zindexed = zindex(
            &mut rng,
            &data,
            &[Value::Integer(5), Value::Shape(shape.clone())],
        )
        .unwrap();
        assert!(matches!(zindexed, Value::Shape(_)));

        // Test zshift
        let zshifted = zshift(
            &mut rng,
            &data,
            &[Value::Integer(5), Value::Shape(shape.clone())],
        )
        .unwrap();
        assert!(matches!(zshifted, Value::Shape(_)));
    }

    #[test]
    fn test_invalid_inputs() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // Create a test shape
        let shape = Rc::new(RefCell::new(Shape::path(vec![
            PathSegment::MoveTo(0.0, 0.0),
            PathSegment::LineTo(10.0, 10.0),
        ])));

        // Test with invalid shape argument
        assert!(translate(
            &mut rng,
            &data,
            &[Value::Integer(5), Value::Integer(10), Value::Integer(0)]
        )
        .is_err());

        // Test with wrong number of arguments
        assert!(translate(&mut rng, &data, &[Value::Integer(5)]).is_err());

        // Test with invalid rotation angle type
        assert!(rotate(
            &mut rng,
            &data,
            &[Value::String("45".into()), Value::Shape(shape.clone())]
        )
        .is_err());
    }
}
