use crate::builtin_function;
use crate::error::{Error, Result};
use crate::interpreter::Value;

use rand_chacha::ChaCha8Rng;

builtin_function!(translate => {
    [Value::Integer(tx), Value::Integer(ty), Value::Shape(shape)] => {
         shape.borrow_mut().translate(*tx as f32, *ty as f32);
         Value::Shape(shape.clone())
    },
    [Value::Float(tx), Value::Float(ty), Value::Shape(shape)] => {
         shape.borrow_mut().translate(*tx, *ty);
         Value::Shape(shape.clone())
    },
    [Value::Integer(tx), Value::Float(ty), Value::Shape(shape)] => {
         shape.borrow_mut().translate(*tx as f32, *ty);
         Value::Shape(shape.clone())
    },
    [Value::Float(tx), Value::Integer(ty), Value::Shape(shape)] => {
         shape.borrow_mut().translate(*tx, *ty as f32);
         Value::Shape(shape.clone())
    },
});

builtin_function!(translatex => {
    [Value::Integer(tx), Value::Shape(shape)] => {
         shape.borrow_mut().translate(*tx as f32, 0.0);
         Value::Shape(shape.clone())
    },
    [Value::Float(tx), Value::Shape(shape)] => {
         shape.borrow_mut().translate(*tx, 0.0);
         Value::Shape(shape.clone())
    },
});

builtin_function!(translatey => {
    [Value::Integer(ty), Value::Shape(shape)] => {
         shape.borrow_mut().translate(0.0, *ty as f32);
         Value::Shape(shape.clone())
    },
    [Value::Float(ty), Value::Shape(shape)] => {
         shape.borrow_mut().translate(0.0, *ty);
         Value::Shape(shape.clone())
    },
});

builtin_function!(translateb => {
    [Value::Integer(t), Value::Shape(shape)] => {
         shape.borrow_mut().translate(*t as f32, *t as f32);
         Value::Shape(shape.clone())
    },
    [Value::Float(t), Value::Shape(shape)] => {
         shape.borrow_mut().translate(*t, *t);
         Value::Shape(shape.clone())
    },
});

builtin_function!(rotate => {
    [Value::Integer(r), Value::Shape(shape)] => {
         shape.borrow_mut().rotate(*r as f32);
         Value::Shape(shape.clone())
    },
    [Value::Float(r), Value::Shape(shape)] => {
         shape.borrow_mut().rotate(*r);
         Value::Shape(shape.clone())
    },
});

builtin_function!(rotate_at => {
    [r, tx, ty, Value::Shape(shape)] => {
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
         shape.borrow_mut().scale(*sx as f32, *sy as f32);
         Value::Shape(shape.clone())
    },
    [Value::Float(sx), Value::Float(sy), Value::Shape(shape)] => {
         shape.borrow_mut().scale(*sx, *sy);
         Value::Shape(shape.clone())
    },
    [Value::Integer(sx), Value::Float(sy), Value::Shape(shape)] => {
         shape.borrow_mut().scale(*sx as f32, *sy);
         Value::Shape(shape.clone())
    },
    [Value::Float(sx), Value::Integer(sy), Value::Shape(shape)] => {
         shape.borrow_mut().scale(*sx, *sy as f32);
         Value::Shape(shape.clone())
    },
});

builtin_function!(scalex => {
    [Value::Integer(sx), Value::Shape(shape)] => {
         shape.borrow_mut().scale(*sx as f32, 0.0);
         Value::Shape(shape.clone())
    },
    [Value::Float(sx), Value::Shape(shape)] => {
         shape.borrow_mut().scale(*sx, 0.0);
         Value::Shape(shape.clone())
    },
});

builtin_function!(scaley => {
    [Value::Integer(sy), Value::Shape(shape)] => {
         shape.borrow_mut().scale(0.0, *sy as f32);
         Value::Shape(shape.clone())
    },
    [Value::Float(sy), Value::Shape(shape)] => {
         shape.borrow_mut().scale(0.0, *sy);
         Value::Shape(shape.clone())
    },
});

builtin_function!(scaleb => {
    [Value::Integer(s), Value::Shape(shape)] => {
         shape.borrow_mut().scale(*s as f32, *s as f32);
         Value::Shape(shape.clone())
    },
    [Value::Float(s), Value::Shape(shape)] => {
         shape.borrow_mut().scale(*s, *s);
         Value::Shape(shape.clone())
    },
});

builtin_function!(skew => {
    [Value::Integer(kx), Value::Integer(ky), Value::Shape(shape)] => {
         shape.borrow_mut().skew(*kx as f32, *ky as f32);
         Value::Shape(shape.clone())
    },
    [Value::Float(kx), Value::Float(ky), Value::Shape(shape)] => {
         shape.borrow_mut().skew(*kx, *ky);
         Value::Shape(shape.clone())
    },
    [Value::Integer(kx), Value::Float(ky), Value::Shape(shape)] => {
         shape.borrow_mut().skew(*kx as f32, *ky);
         Value::Shape(shape.clone())
    },
    [Value::Float(kx), Value::Integer(ky), Value::Shape(shape)] => {
         shape.borrow_mut().skew(*kx, *ky as f32);
         Value::Shape(shape.clone())
    },
});

builtin_function!(skewx => {
    [Value::Integer(kx), Value::Shape(shape)] => {
         shape.borrow_mut().skew(*kx as f32, 0.0);
         Value::Shape(shape.clone())
    },
    [Value::Float(kx), Value::Shape(shape)] => {
         shape.borrow_mut().skew(*kx, 0.0);
         Value::Shape(shape.clone())
    },
});

builtin_function!(skewy => {
    [Value::Integer(ky), Value::Shape(shape)] => {
         shape.borrow_mut().skew(0.0, *ky as f32);
         Value::Shape(shape.clone())
    },
    [Value::Float(ky), Value::Shape(shape)] => {
         shape.borrow_mut().skew(0.0, *ky);
         Value::Shape(shape.clone())
    },
});

builtin_function!(skewb => {
    [Value::Integer(k), Value::Shape(shape)] => {
         shape.borrow_mut().skew(*k as f32, *k as f32);
         Value::Shape(shape.clone())
    },
    [Value::Float(k), Value::Shape(shape)] => {
         shape.borrow_mut().skew(*k, *k);
         Value::Shape(shape.clone())
    },
});

builtin_function!(flip => {
    [Value::Integer(f), Value::Shape(shape)] => {
        shape.borrow_mut().flip(*f as f32);
        Value::Shape(shape.clone())
    },
    [Value::Float(f), Value::Shape(shape)] => {
        shape.borrow_mut().flip(*f);
        Value::Shape(shape.clone())
    },
});

builtin_function!(fliph => {
    [Value::Shape(shape)] => {
        shape.borrow_mut().fliph();
        Value::Shape(shape.clone())
    },
});

builtin_function!(flipv => {
    [Value::Shape(shape)] => {
        shape.borrow_mut().flipv();
        Value::Shape(shape.clone())
    },
});

builtin_function!(flipd => {
    [Value::Shape(shape)] => {
        shape.borrow_mut().flipd();
        Value::Shape(shape.clone())
    },
});

builtin_function!(zindex => {
    [Value::Integer(z), Value::Shape(shape)] => {
        shape.borrow_mut().set_zindex(*z as f32);
        Value::Shape(shape.clone())
    },
    [Value::Float(z), Value::Shape(shape)] => {
        shape.borrow_mut().set_zindex(*z);
        Value::Shape(shape.clone())
    },
});

builtin_function!(zshift => {
    [Value::Integer(z), Value::Shape(shape)] => {
        shape.borrow_mut().shift_zindex(*z as f32);
        Value::Shape(shape.clone())
    },
    [Value::Float(z), Value::Shape(shape)] => {
        shape.borrow_mut().shift_zindex(*z);
        Value::Shape(shape.clone())
    },
});
