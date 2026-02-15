#[cfg(feature = "std")]
use std::rc::Rc;

#[cfg(feature = "alloc")]
use alloc::{rc::Rc, vec};

use crate::builtin_function;
use crate::error::{Error, Result};
use crate::interpreter::{Data, Value};
use crate::shape::{PathSegment, Shape};
use core::cell::RefCell;

use rand_chacha::ChaCha8Rng;

builtin_function!(move_to => {
    [x, y] => {
        let x = match x {
            Value::Integer(x) => *x as f32,
            Value::Float(x) => *x,
            _ => return Err(Error::InvalidArgument("move_to".into())),
        };

        let y = match y {
            Value::Integer(y) => *y as f32,
            Value::Float(y) => *y,
            _ => return Err(Error::InvalidArgument("move_to".into())),
        };

        let segments = vec![PathSegment::MoveTo(x, y)];
        let shape = Shape::path(segments);
        Value::Shape(Rc::new(RefCell::new(shape)))
    }
});

builtin_function!(line_to => {
    [x, y] => {
        let x = match x {
            Value::Integer(x) => *x as f32,
            Value::Float(x) => *x,
            _ => return Err(Error::InvalidArgument("line_to".into())),
        };

        let y = match y {
            Value::Integer(y) => *y as f32,
            Value::Float(y) => *y,
            _ => return Err(Error::InvalidArgument("line_to".into())),
        };

        let segments = vec![PathSegment::LineTo(x, y)];
        let shape = Shape::path(segments);
        Value::Shape(Rc::new(RefCell::new(shape)))
    }
});

builtin_function!(quad_to => {
    [x1, y1, x, y] => {
        let x1 = match x1 {
            Value::Integer(x1) => *x1 as f32,
            Value::Float(x1) => *x1,
            _ => return Err(Error::InvalidArgument("quad_to".into())),
        };

        let y1 = match y1 {
            Value::Integer(y1) => *y1 as f32,
            Value::Float(y1) => *y1,
            _ => return Err(Error::InvalidArgument("quad_to".into())),
        };

        let x = match x {
            Value::Integer(x) => *x as f32,
            Value::Float(x) => *x,
            _ => return Err(Error::InvalidArgument("quad_to".into())),
        };

        let y = match y {
            Value::Integer(y) => *y as f32,
            Value::Float(y) => *y,
            _ => return Err(Error::InvalidArgument("quad_to".into())),
        };

        let segments = vec![PathSegment::QuadTo(x1, y1, x, y)];
        let shape = Shape::path(segments);
        Value::Shape(Rc::new(RefCell::new(shape)))
    }
});

builtin_function!(cubic_to => {
    [x1, y1, x2, y2, x, y] => {
        let x1 = match x1 {
            Value::Integer(x1) => *x1 as f32,
            Value::Float(x1) => *x1,
            _ => return Err(Error::InvalidArgument("cubic_to".into())),
        };

        let y1 = match y1 {
            Value::Integer(y1) => *y1 as f32,
            Value::Float(y1) => *y1,
            _ => return Err(Error::InvalidArgument("cubic_to".into())),
        };

        let x2 = match x2 {
            Value::Integer(x2) => *x2 as f32,
            Value::Float(x2) => *x2,
            _ => return Err(Error::InvalidArgument("cubic_to".into())),
        };

        let y2 = match y2 {
            Value::Integer(y2) => *y2 as f32,
            Value::Float(y2) => *y2,
            _ => return Err(Error::InvalidArgument("cubic_to".into())),
        };

        let x = match x {
            Value::Integer(x) => *x as f32,
            Value::Float(x) => *x,
            _ => return Err(Error::InvalidArgument("cubic_to".into())),
        };

        let y = match y {
            Value::Integer(y) => *y as f32,
            Value::Float(y) => *y,
            _ => return Err(Error::InvalidArgument("cubic_to".into())),
        };

        let segments = vec![PathSegment::CubicTo(x1, y1, x2, y2, x, y)];
        let shape = Shape::path(segments);
        Value::Shape(Rc::new(RefCell::new(shape)))
    }
});

builtin_function!(close => {
    [] => {
        let segments = vec![PathSegment::Close];
        let shape = Shape::path(segments);
        Value::Shape(Rc::new(RefCell::new(shape)))
    }
});

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn test_move_to() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // Test with integer coordinates
        let result_int =
            move_to(&mut rng, &data, &[Value::Integer(10), Value::Integer(20)]).unwrap();

        // Test with float coordinates
        let result_float =
            move_to(&mut rng, &data, &[Value::Float(10.5), Value::Float(20.5)]).unwrap();

        // Test with mixed coordinates
        let result_mixed =
            move_to(&mut rng, &data, &[Value::Integer(10), Value::Float(20.5)]).unwrap();

        // Verify all results are shapes with MoveTo segments
        for result in [result_int, result_float, result_mixed] {
            if let Value::Shape(shape) = result {
                let shape = shape.borrow();
                if let Shape::Path { segments, .. } = &*shape {
                    assert_eq!(segments.len(), 1);
                    assert!(matches!(segments[0], PathSegment::MoveTo(_, _)));
                } else {
                    panic!("Expected Path shape");
                }
            } else {
                panic!("Expected Shape value");
            }
        }

        // Test invalid arguments
        assert!(move_to(
            &mut rng,
            &data,
            &[Value::String("10".into()), Value::Integer(20)]
        )
        .is_err());

        assert!(move_to(&mut rng, &data, &[Value::Integer(10)]).is_err());
    }

    #[test]
    fn test_line_to() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // Test with integer coordinates
        let result_int =
            line_to(&mut rng, &data, &[Value::Integer(30), Value::Integer(40)]).unwrap();

        // Test with float coordinates
        let result_float =
            line_to(&mut rng, &data, &[Value::Float(30.5), Value::Float(40.5)]).unwrap();

        // Verify all results are shapes with LineTo segments
        for result in [result_int, result_float] {
            if let Value::Shape(shape) = result {
                let shape = shape.borrow();
                if let Shape::Path { segments, .. } = &*shape {
                    assert_eq!(segments.len(), 1);
                    assert!(matches!(segments[0], PathSegment::LineTo(_, _)));
                } else {
                    panic!("Expected Path shape");
                }
            } else {
                panic!("Expected Shape value");
            }
        }
    }

    #[test]
    fn test_quad_to() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // Test with integer coordinates
        let result_int = quad_to(
            &mut rng,
            &data,
            &[
                Value::Integer(10),
                Value::Integer(15),
                Value::Integer(20),
                Value::Integer(25),
            ],
        )
        .unwrap();

        // Test with float coordinates
        let result_float = quad_to(
            &mut rng,
            &data,
            &[
                Value::Float(10.5),
                Value::Float(15.5),
                Value::Float(20.5),
                Value::Float(25.5),
            ],
        )
        .unwrap();

        // Verify all results are shapes with QuadTo segments
        for result in [result_int, result_float] {
            if let Value::Shape(shape) = result {
                let shape = shape.borrow();
                if let Shape::Path { segments, .. } = &*shape {
                    assert_eq!(segments.len(), 1);
                    assert!(matches!(segments[0], PathSegment::QuadTo(_, _, _, _)));
                } else {
                    panic!("Expected Path shape");
                }
            } else {
                panic!("Expected Shape value");
            }
        }

        // Test with wrong number of arguments
        assert!(quad_to(
            &mut rng,
            &data,
            &[Value::Integer(10), Value::Integer(15), Value::Integer(20)]
        )
        .is_err());
    }

    #[test]
    fn test_cubic_to() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // Test with integer coordinates
        let result_int = cubic_to(
            &mut rng,
            &data,
            &[
                Value::Integer(10),
                Value::Integer(15),
                Value::Integer(20),
                Value::Integer(25),
                Value::Integer(30),
                Value::Integer(35),
            ],
        )
        .unwrap();

        // Test with float coordinates
        let result_float = cubic_to(
            &mut rng,
            &data,
            &[
                Value::Float(10.5),
                Value::Float(15.5),
                Value::Float(20.5),
                Value::Float(25.5),
                Value::Float(30.5),
                Value::Float(35.5),
            ],
        )
        .unwrap();

        // Verify all results are shapes with CubicTo segments
        for result in [result_int, result_float] {
            if let Value::Shape(shape) = result {
                let shape = shape.borrow();
                if let Shape::Path { segments, .. } = &*shape {
                    assert_eq!(segments.len(), 1);
                    assert!(matches!(
                        segments[0],
                        PathSegment::CubicTo(_, _, _, _, _, _)
                    ));
                } else {
                    panic!("Expected Path shape");
                }
            } else {
                panic!("Expected Shape value");
            }
        }

        // Test with wrong number of arguments
        assert!(cubic_to(
            &mut rng,
            &data,
            &[
                Value::Integer(10),
                Value::Integer(15),
                Value::Integer(20),
                Value::Integer(25),
                Value::Integer(30)
            ]
        )
        .is_err());
    }

    #[test]
    fn test_close() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // Test close path
        let result = close(&mut rng, &data, &[]).unwrap();

        if let Value::Shape(shape) = result {
            let shape = shape.borrow();
            if let Shape::Path { segments, .. } = &*shape {
                assert_eq!(segments.len(), 1);
                assert!(matches!(segments[0], PathSegment::Close));
            } else {
                panic!("Expected Path shape");
            }
        } else {
            panic!("Expected Shape value");
        }

        // Test with arguments (should error)
        assert!(close(&mut rng, &data, &[Value::Integer(10)]).is_err());
    }
}
