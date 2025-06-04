use crate::builtin_function;
use crate::interpreter::Value;

use crate::error::{Error, Result};
use noise::Perlin;
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

builtin_function!(blend => {
    [Value::BlendMode(blend_mode), Value::Shape(shape)] => {
         shape.borrow_mut().set_blend_mode(*blend_mode);
         Value::Shape(shape.clone())
    }
});

builtin_function!(anti_alias => {
    [Value::Boolean(anti_alias), Value::Shape(shape)] => {
         shape.borrow_mut().set_anti_alias(*anti_alias);
         Value::Shape(shape.clone())
    }
});
