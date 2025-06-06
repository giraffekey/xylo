use crate::builtin_function;
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
         shape.borrow_mut().set_hsla(h, s, l, a);
         Value::Shape(shape.clone())
    }
});

builtin_function!(hue => {
    [Value::Integer(h), Value::Shape(shape)] => {
         shape.borrow_mut().set_hue(*h as f32);
         Value::Shape(shape.clone())
    },
    [Value::Float(h), Value::Shape(shape)] => {
         shape.borrow_mut().set_hue(*h);
         Value::Shape(shape.clone())
    }
});

builtin_function!(saturation => {
    [Value::Integer(s), Value::Shape(shape)] => {
         shape.borrow_mut().set_saturation(*s as f32);
         Value::Shape(shape.clone())
    },
    [Value::Float(s), Value::Shape(shape)] => {
         shape.borrow_mut().set_saturation(*s);
         Value::Shape(shape.clone())
    }
});

builtin_function!(lightness => {
    [Value::Integer(l), Value::Shape(shape)] => {
         shape.borrow_mut().set_lightness(*l as f32);
         Value::Shape(shape.clone())
    },
    [Value::Float(l), Value::Shape(shape)] => {
         shape.borrow_mut().set_lightness(*l);
         Value::Shape(shape.clone())
    }
});

builtin_function!(alpha => {
    [Value::Integer(a), Value::Shape(shape)] => {
         shape.borrow_mut().set_alpha(*a as f32);
         Value::Shape(shape.clone())
    },
    [Value::Float(a), Value::Shape(shape)] => {
         shape.borrow_mut().set_alpha(*a);
         Value::Shape(shape.clone())
    }
});

builtin_function!(hshift => {
    [Value::Integer(h), Value::Shape(shape)] => {
         shape.borrow_mut().shift_hue(*h as f32);
         Value::Shape(shape.clone())
    },
    [Value::Float(h), Value::Shape(shape)] => {
         shape.borrow_mut().shift_hue(*h);
         Value::Shape(shape.clone())
    }
});

builtin_function!(satshift => {
    [Value::Integer(s), Value::Shape(shape)] => {
         shape.borrow_mut().shift_saturation(*s as f32);
         Value::Shape(shape.clone())
    },
    [Value::Float(s), Value::Shape(shape)] => {
         shape.borrow_mut().shift_saturation(*s);
         Value::Shape(shape.clone())
    }
});

builtin_function!(lshift => {
    [Value::Integer(l), Value::Shape(shape)] => {
         shape.borrow_mut().shift_lightness(*l as f32);
         Value::Shape(shape.clone())
    },
    [Value::Float(l), Value::Shape(shape)] => {
         shape.borrow_mut().shift_lightness(*l);
         Value::Shape(shape.clone())
    }
});

builtin_function!(ashift => {
    [Value::Integer(a), Value::Shape(shape)] => {
         shape.borrow_mut().shift_alpha(*a as f32);
         Value::Shape(shape.clone())
    },
    [Value::Float(a), Value::Shape(shape)] => {
         shape.borrow_mut().shift_alpha(*a);
         Value::Shape(shape.clone())
    }
});

builtin_function!(hex => {
    [Value::Hex(hex), Value::Shape(shape)] => {
         shape.borrow_mut().set_hex(*hex);
         Value::Shape(shape.clone())
    }
});

builtin_function!(solid => {
    [Value::Shape(shape)] => {
        shape.borrow_mut().set_color(Color::Solid(WHITE));
        Value::Shape(shape.clone())
    }
});

builtin_function!(gradient => {
    [Value::Gradient(g), Value::Shape(shape)] => {
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
