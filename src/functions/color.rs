use crate::builtin_function;
use crate::functions::dedup_shape;
use crate::interpreter::{Data, Value};
use crate::shape::{Color, Gradient, WHITE};

use crate::error::{Error, Result};
use rand_chacha::ChaCha8Rng;

builtin_function!(hsl => {
    [h, s, l, Value::Shape(shape)] => {
         let h = match h {
             Value::Integer(h) => *h as f32,
             Value::Float(h)   => *h,
             _ => return Err(Error::InvalidArgument("hsl".into())),
         };
         let s = match s {
             Value::Integer(s) => *s as f32,
             Value::Float(s)   => *s,
             _ => return Err(Error::InvalidArgument("hsl".into())),
         };
         let l = match l {
             Value::Integer(l) => *l as f32,
             Value::Float(l)   => *l,
             _ => return Err(Error::InvalidArgument("hsl".into())),
         };

        let shape = dedup_shape(shape);
        shape.borrow_mut().set_hsl(h, s, l);
        Value::Shape(shape.clone())
    }
});

builtin_function!(hsla => {
    [h, s, l, a, Value::Shape(shape)] => {
         let h = match h {
             Value::Integer(h) => *h as f32,
             Value::Float(h)   => *h,
             _ => return Err(Error::InvalidArgument("hsla".into())),
         };
         let s = match s {
             Value::Integer(s) => *s as f32,
             Value::Float(s)   => *s,
             _ => return Err(Error::InvalidArgument("hsla".into())),
         };
         let l = match l {
             Value::Integer(l) => *l as f32,
             Value::Float(l)   => *l,
             _ => return Err(Error::InvalidArgument("hsla".into())),
         };
         let a = match a {
             Value::Integer(a) => *a as f32,
             Value::Float(a)   => *a,
             _ => return Err(Error::InvalidArgument("hsla".into())),
         };

        let shape = dedup_shape(shape);
        shape.borrow_mut().set_hsla(h, s, l, a);
        Value::Shape(shape.clone())
    }
});

builtin_function!(hue => {
    [Value::Integer(h), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().set_hue(*h as f32);
        Value::Shape(shape.clone())
    },
    [Value::Float(h), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().set_hue(*h);
        Value::Shape(shape.clone())
    }
});

builtin_function!(saturation => {
    [Value::Integer(s), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().set_saturation(*s as f32);
        Value::Shape(shape.clone())
    },
    [Value::Float(s), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().set_saturation(*s);
        Value::Shape(shape.clone())
    }
});

builtin_function!(lightness => {
    [Value::Integer(l), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().set_lightness(*l as f32);
        Value::Shape(shape.clone())
    },
    [Value::Float(l), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().set_lightness(*l);
        Value::Shape(shape.clone())
    }
});

builtin_function!(alpha => {
    [Value::Integer(a), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().set_alpha(*a as f32);
        Value::Shape(shape.clone())
    },
    [Value::Float(a), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().set_alpha(*a);
        Value::Shape(shape.clone())
    }
});

builtin_function!(hshift => {
    [Value::Integer(h), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().shift_hue(*h as f32);
        Value::Shape(shape.clone())
    },
    [Value::Float(h), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().shift_hue(*h);
        Value::Shape(shape.clone())
    }
});

builtin_function!(satshift => {
    [Value::Integer(s), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().shift_saturation(*s as f32);
        Value::Shape(shape.clone())
    },
    [Value::Float(s), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().shift_saturation(*s);
        Value::Shape(shape.clone())
    }
});

builtin_function!(lshift => {
    [Value::Integer(l), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().shift_lightness(*l as f32);
        Value::Shape(shape.clone())
    },
    [Value::Float(l), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().shift_lightness(*l);
        Value::Shape(shape.clone())
    }
});

builtin_function!(ashift => {
    [Value::Integer(a), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().shift_alpha(*a as f32);
        Value::Shape(shape.clone())
    },
    [Value::Float(a), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().shift_alpha(*a);
        Value::Shape(shape.clone())
    }
});

builtin_function!(hex => {
    [Value::Hex(hex), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().set_hex(*hex);
        Value::Shape(shape.clone())
    }
});

builtin_function!(solid => {
    [Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().set_color(Color::Solid(WHITE));
        Value::Shape(shape.clone())
    }
});

builtin_function!(gradient => {
    [Value::Gradient(g), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().set_color(Color::Gradient(g.clone()));
        Value::Shape(shape.clone())
    }
});

builtin_function!(linear_grad => {
    [start_x, start_y, end_x, end_y] => {
        let start_x = match start_x {
            Value::Integer(start_x) => *start_x as f32,
            Value::Float(start_x)   => *start_x,
        _ => return Err(Error::InvalidArgument("linear_grad".into())),
        };
        let start_y = match start_y {
            Value::Integer(start_y) => *start_y as f32,
            Value::Float(start_y)   => *start_y,
        _ => return Err(Error::InvalidArgument("linear_grad".into())),
        };
        let end_x = match end_x {
            Value::Integer(end_x) => *end_x as f32,
            Value::Float(end_x)   => *end_x,
        _ => return Err(Error::InvalidArgument("linear_grad".into())),
        };
        let end_y = match end_y {
            Value::Integer(end_y) => *end_y as f32,
            Value::Float(end_y)   => *end_y,
        _ => return Err(Error::InvalidArgument("linear_grad".into())),
        };
        Value::Gradient(Gradient::linear(start_x, start_y, end_x, end_y))
    }
});

builtin_function!(radial_grad => {
    [start_x, start_y, end_x, end_y, radius] => {
        let start_x = match start_x {
            Value::Integer(start_x) => *start_x as f32,
            Value::Float(start_x)   => *start_x,
        _ => return Err(Error::InvalidArgument("radial_grad".into())),
        };
        let start_y = match start_y {
            Value::Integer(start_y) => *start_y as f32,
            Value::Float(start_y)   => *start_y,
        _ => return Err(Error::InvalidArgument("radial_grad".into())),
        };
        let end_x = match end_x {
            Value::Integer(end_x) => *end_x as f32,
            Value::Float(end_x)   => *end_x,
        _ => return Err(Error::InvalidArgument("radial_grad".into())),
        };
        let end_y = match end_y {
            Value::Integer(end_y) => *end_y as f32,
            Value::Float(end_y)   => *end_y,
        _ => return Err(Error::InvalidArgument("radial_grad".into())),
        };
        let radius = match radius {
            Value::Integer(radius) => *radius as f32,
            Value::Float(radius)   => *radius,
            _ => return Err(Error::InvalidArgument("radial_grad".into())),
        };
        Value::Gradient(Gradient::radial(start_x, start_y, end_x, end_y, radius))
    }
});

builtin_function!(grad_start => {
    [start_x, start_y, Value::Gradient(g)] => {
        let start_x = match start_x {
            Value::Integer(start_x) => *start_x as f32,
            Value::Float(start_x)   => *start_x,
            _ => return Err(Error::InvalidArgument("grad_start".into())),
        };
        let start_y = match start_y {
            Value::Integer(start_y) => *start_y as f32,
            Value::Float(start_y)   => *start_y,
            _ => return Err(Error::InvalidArgument("grad_start".into())),
        };

        let mut g = g.clone();
        g.set_start(start_x, start_y);
        Value::Gradient(g)
    }
});

builtin_function!(grad_end => {
    [end_x, end_y, Value::Gradient(g)] => {
        let end_x = match end_x {
            Value::Integer(end_x) => *end_x as f32,
            Value::Float(end_x)   => *end_x,
            _ => return Err(Error::InvalidArgument("grad_end".into())),
        };
        let end_y = match end_y {
            Value::Integer(end_y) => *end_y as f32,
            Value::Float(end_y)   => *end_y,
            _ => return Err(Error::InvalidArgument("grad_end".into())),
        };

        let mut g = g.clone();
        g.set_end(end_x, end_y);
        Value::Gradient(g)
    }
});

builtin_function!(to_linear_grad => {
    [Value::Gradient(g)] => {
        let mut g = g.clone();
        g.set_radius(None);
        Value::Gradient(g)
    }
});

builtin_function!(grad_radius => {
    [radius, Value::Gradient(g)] => {
        let radius = match radius {
            Value::Integer(radius) => *radius as f32,
            Value::Float(radius)   => *radius,
            _ => return Err(Error::InvalidArgument("grad_radius".into())),
        };

        let mut g = g.clone();
        g.set_radius(Some(radius));
        Value::Gradient(g)
    }
});

builtin_function!(grad_stop_hsl => {
    [pos, h, s, l, Value::Gradient(g)] => {
         let pos = match pos {
             Value::Integer(pos) => *pos as f32,
             Value::Float(pos)   => *pos,
             _ => return Err(Error::InvalidArgument("grad_stop_hsl".into())),
         };
         let h = match h {
             Value::Integer(h) => *h as f32,
             Value::Float(h)   => *h,
             _ => return Err(Error::InvalidArgument("grad_stop_hsl".into())),
         };
         let s = match s {
             Value::Integer(s) => *s as f32,
             Value::Float(s)   => *s,
             _ => return Err(Error::InvalidArgument("grad_stop_hsl".into())),
         };
         let l = match l {
             Value::Integer(l) => *l as f32,
             Value::Float(l)   => *l,
             _ => return Err(Error::InvalidArgument("grad_stop_hsl".into())),
         };

        let mut g = g.clone();
        g.set_stop_hsl(pos, h, s, l);
        Value::Gradient(g)
    }
});

builtin_function!(grad_stop_hsla => {
    [pos, h, s, l, a, Value::Gradient(g)] => {
         let pos = match pos {
             Value::Integer(pos) => *pos as f32,
             Value::Float(pos)   => *pos,
             _ => return Err(Error::InvalidArgument("grad_stop_hsla".into())),
         };
         let h = match h {
             Value::Integer(h) => *h as f32,
             Value::Float(h)   => *h,
             _ => return Err(Error::InvalidArgument("grad_stop_hsla".into())),
         };
         let s = match s {
             Value::Integer(s) => *s as f32,
             Value::Float(s)   => *s,
             _ => return Err(Error::InvalidArgument("grad_stop_hsla".into())),
         };
         let l = match l {
             Value::Integer(l) => *l as f32,
             Value::Float(l)   => *l,
             _ => return Err(Error::InvalidArgument("grad_stop_hsla".into())),
         };
         let a = match a {
             Value::Integer(a) => *a as f32,
             Value::Float(a)   => *a,
             _ => return Err(Error::InvalidArgument("grad_stop_hsla".into())),
         };

        let mut g = g.clone();
        g.set_stop_hsla(pos, h, s, l, a);
        Value::Gradient(g)
    }
});

builtin_function!(grad_stop_hex => {
    [pos, Value::Hex(hex), Value::Gradient(g)] => {
         let pos = match pos {
             Value::Integer(pos) => *pos as f32,
             Value::Float(pos)   => *pos,
             _ => return Err(Error::InvalidArgument("grad_stop_hex".into())),
         };

        let mut g = g.clone();
        g.set_stop_hex(pos, *hex);
        Value::Gradient(g)
    }
});

builtin_function!(grad_spread_mode => {
    [Value::SpreadMode(spread_mode), Value::Gradient(g)] => {
        let mut g = g.clone();
        g.set_spread_mode(*spread_mode);
        Value::Gradient(g)
    }
});

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "std")]
    use std::rc::Rc;

    #[cfg(feature = "no-std")]
    use alloc::{rc::Rc, vec};

    use crate::shape::Shape;
    use core::cell::RefCell;
    use rand::SeedableRng;
    use tiny_skia::SpreadMode;

    #[test]
    fn test_hsl_functions() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        let shape = Rc::new(RefCell::new(Shape::path(vec![])));

        // Test hsl with various input types
        let hsl_tests = vec![
            (Value::Integer(180), Value::Integer(50), Value::Integer(50)), // int, int, int
            (Value::Float(180.0), Value::Float(0.5), Value::Float(0.5)),   // float, float, float
            (Value::Integer(180), Value::Float(0.5), Value::Integer(50)),  // mixed types
        ];

        for (h, s, l) in hsl_tests {
            let result = hsl(&mut rng, &data, &[h, s, l, Value::Shape(shape.clone())]).unwrap();

            assert!(matches!(result, Value::Shape(_)));
        }

        // Test hsla
        let hsla_result = hsla(
            &mut rng,
            &data,
            &[
                Value::Integer(180),
                Value::Integer(50),
                Value::Integer(50),
                Value::Float(0.8),
                Value::Shape(shape.clone()),
            ],
        )
        .unwrap();
        assert!(matches!(hsla_result, Value::Shape(_)));

        // Test individual components
        let hue_result = hue(
            &mut rng,
            &data,
            &[Value::Integer(90), Value::Shape(shape.clone())],
        )
        .unwrap();
        assert!(matches!(hue_result, Value::Shape(_)));

        let sat_result = saturation(
            &mut rng,
            &data,
            &[Value::Float(0.75), Value::Shape(shape.clone())],
        )
        .unwrap();
        assert!(matches!(sat_result, Value::Shape(_)));

        // Test shifts
        let hshift_result = hshift(
            &mut rng,
            &data,
            &[Value::Integer(30), Value::Shape(shape.clone())],
        )
        .unwrap();
        assert!(matches!(hshift_result, Value::Shape(_)));
    }

    #[test]
    fn test_solid_and_hex() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        let shape = Rc::new(RefCell::new(Shape::path(vec![])));

        // Test solid
        let solid_result = solid(&mut rng, &data, &[Value::Shape(shape.clone())]).unwrap();
        assert!(matches!(solid_result, Value::Shape(_)));

        // Test hex
        let hex_result = hex(
            &mut rng,
            &data,
            &[Value::Hex([255, 0, 0]), Value::Shape(shape.clone())],
        )
        .unwrap();
        assert!(matches!(hex_result, Value::Shape(_)));
    }

    #[test]
    fn test_gradient_creation() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // Test linear gradient
        let linear_grad = linear_grad(
            &mut rng,
            &data,
            &[
                Value::Integer(0),
                Value::Integer(0),
                Value::Integer(100),
                Value::Integer(100),
            ],
        )
        .unwrap();
        assert!(matches!(linear_grad, Value::Gradient(_)));

        // Test radial gradient
        let radial_grad = radial_grad(
            &mut rng,
            &data,
            &[
                Value::Integer(50),
                Value::Integer(50),
                Value::Integer(100),
                Value::Integer(100),
                Value::Integer(50),
            ],
        )
        .unwrap();
        assert!(matches!(radial_grad, Value::Gradient(_)));

        // Test gradient application to shape
        let shape = Rc::new(RefCell::new(Shape::path(vec![])));
        let grad_result =
            gradient(&mut rng, &data, &[linear_grad, Value::Shape(shape.clone())]).unwrap();
        assert!(matches!(grad_result, Value::Shape(_)));
    }

    #[test]
    fn test_gradient_manipulation() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // Create base gradient
        let grad = linear_grad(
            &mut rng,
            &data,
            &[
                Value::Integer(0),
                Value::Integer(0),
                Value::Integer(100),
                Value::Integer(100),
            ],
        )
        .unwrap();

        // Test gradient start/end modification
        let new_start = grad_start(
            &mut rng,
            &data,
            &[Value::Integer(10), Value::Integer(10), grad.clone()],
        )
        .unwrap();
        assert!(matches!(new_start, Value::Gradient(_)));

        let new_end = grad_end(
            &mut rng,
            &data,
            &[Value::Integer(90), Value::Integer(90), new_start.clone()],
        )
        .unwrap();
        assert!(matches!(new_end, Value::Gradient(_)));

        // Test gradient stops
        let with_stop = grad_stop_hsl(
            &mut rng,
            &data,
            &[
                Value::Float(0.5),
                Value::Integer(180),
                Value::Integer(50),
                Value::Integer(50),
                new_end.clone(),
            ],
        )
        .unwrap();
        assert!(matches!(with_stop, Value::Gradient(_)));

        // Test spread mode
        let with_spread = grad_spread_mode(
            &mut rng,
            &data,
            &[Value::SpreadMode(SpreadMode::Repeat), with_stop.clone()],
        )
        .unwrap();
        assert!(matches!(with_spread, Value::Gradient(_)));
    }

    #[test]
    fn test_gradient_conversion() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // Create radial gradient
        let radial = radial_grad(
            &mut rng,
            &data,
            &[
                Value::Integer(50),
                Value::Integer(50),
                Value::Integer(100),
                Value::Integer(100),
                Value::Integer(50),
            ],
        )
        .unwrap();

        // Convert to linear
        let linear = to_linear_grad(&mut rng, &data, &[radial.clone()]).unwrap();
        assert!(matches!(linear, Value::Gradient(_)));

        // Convert back to radial
        let radial_again =
            grad_radius(&mut rng, &data, &[Value::Integer(75), linear.clone()]).unwrap();
        assert!(matches!(radial_again, Value::Gradient(_)));
    }

    #[test]
    fn test_invalid_inputs() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        let shape = Rc::new(RefCell::new(Shape::path(vec![])));

        // Test hsl with wrong number of arguments
        assert!(hsl(
            &mut rng,
            &data,
            &[
                Value::Integer(180),
                Value::Integer(50),
                Value::Shape(shape.clone())
            ]
        )
        .is_err());

        // Test gradient with invalid position
        let grad = linear_grad(
            &mut rng,
            &data,
            &[
                Value::Integer(0),
                Value::Integer(0),
                Value::Integer(100),
                Value::Integer(100),
            ],
        )
        .unwrap();

        assert!(grad_stop_hsl(
            &mut rng,
            &data,
            &[
                Value::String("0.5".into()), // invalid position
                Value::Integer(180),
                Value::Integer(50),
                Value::Integer(50),
                grad.clone()
            ]
        )
        .is_err());
    }
}
