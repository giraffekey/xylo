#[cfg(feature = "std")]
use std::rc::Rc;

#[cfg(feature = "no-std")]
use alloc::{rc::Rc, vec, vec::Vec};

use crate::interpreter::Value;
use crate::shape::{HslaChange, PathSegment, Shape, IDENTITY, WHITE};
use core::cell::RefCell;

use anyhow::{anyhow, Result};
use rand_chacha::ChaCha8Rng;

pub fn compose(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match (&args[0], &args[1]) {
        (Value::Shape(a), Value::Shape(b)) => {
            let shape = match (&*a.borrow(), &*b.borrow()) {
                (
                    Shape::Path {
                        segments: a,
                        transform: a_transform,
                        zindex,
                        color,
                    },
                    Shape::Path {
                        segments: b,
                        transform: b_transform,
                        ..
                    },
                ) => {
                    let mut segments = Vec::with_capacity(a.len() + b.len());
                    segments.extend(a);
                    segments.extend(b);
                    Shape::Path {
                        segments,
                        transform: a_transform.post_concat(*b_transform),
                        zindex: *zindex,
                        color: *color,
                    }
                }
                _ => Shape::Composite {
                    a: a.clone(),
                    b: b.clone(),
                    transform: IDENTITY,
                    zindex_overwrite: None,
                    zindex_shift: None,
                    color_overwrite: HslaChange::default(),
                    color_shift: HslaChange::default(),
                },
            };
            Ok(Value::Shape(Rc::new(RefCell::new(shape))))
        }
        _ => Err(anyhow!("Invalid type passed to `compose` function.")),
    }
}

pub fn collect(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    match &args[0] {
        Value::List(list) => {
            let shapes: Result<Vec<Rc<RefCell<Shape>>>> = list
                .iter()
                .map(|item| match item {
                    Value::Shape(shape) => Ok(shape.clone()),
                    _ => Err(anyhow!("Invalid type passed to `collect` function.")),
                })
                .collect();
            let shapes = shapes?;

            if shapes.len() < 1 {
                return Err(anyhow!("Cannot collect zero shapes."));
            }

            let is_path = shapes.iter().all(|shape| match &*shape.borrow() {
                Shape::Path { .. } => true,
                _ => false,
            });
            let shape = if is_path {
                let mut segments = Vec::with_capacity(shapes.len());
                let mut transform = IDENTITY;
                let mut zindex = None;
                let mut color = WHITE;

                for path in shapes {
                    match &*path.borrow() {
                        Shape::Path {
                            segments: other_segments,
                            transform: other_transform,
                            zindex: other_zindex,
                            color: other_color,
                        } => {
                            segments.extend(other_segments);
                            transform = transform.post_concat(*other_transform);
                            zindex = *other_zindex;
                            color = *other_color;
                        }
                        _ => unreachable!(),
                    }
                }

                Shape::Path {
                    segments,
                    transform,
                    zindex,
                    color,
                }
            } else {
                Shape::Collection {
                    shapes,
                    transform: IDENTITY,
                    zindex_overwrite: None,
                    zindex_shift: None,
                    color_overwrite: HslaChange::default(),
                    color_shift: HslaChange::default(),
                }
            };
            Ok(Value::Shape(Rc::new(RefCell::new(shape))))
        }
        _ => Err(anyhow!("Invalid type passed to `collect` function.")),
    }
}

pub fn translate(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let tx = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `translate` function.")),
    };

    let ty = match args[1] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `translate` function.")),
    };

    let shape = match &args[2] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `translate` function.")),
    };

    shape.borrow_mut().translate(tx, ty);

    Ok(Value::Shape(shape))
}

pub fn translatex(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let tx = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `translatex` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `translatex` function.")),
    };

    shape.borrow_mut().translate(tx, 0.0);
    Ok(Value::Shape(shape))
}

pub fn translatey(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let ty = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `translatey` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `translatey` function.")),
    };

    shape.borrow_mut().translate(0.0, ty);
    Ok(Value::Shape(shape))
}

pub fn translateb(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let t = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `translateb` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `translateb` function.")),
    };

    shape.borrow_mut().translate(t, t);
    Ok(Value::Shape(shape))
}

pub fn rotate(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let r = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `rotate` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `rotate` function.")),
    };

    shape.borrow_mut().rotate(r);
    Ok(Value::Shape(shape))
}

pub fn rotate_at(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let r = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `rotate_at` function.")),
    };

    let tx = match args[1] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `rotate_at` function.")),
    };

    let ty = match args[2] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `rotate_at` function.")),
    };

    let shape = match &args[3] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `rotate_at` function.")),
    };

    shape.borrow_mut().rotate_at(r, tx, ty);
    Ok(Value::Shape(shape))
}

pub fn scale(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let sx = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `scale` function.")),
    };

    let sy = match args[1] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `scale` function.")),
    };

    let shape = match &args[2] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `scale` function.")),
    };

    shape.borrow_mut().scale(sx, sy);
    Ok(Value::Shape(shape))
}

pub fn scalex(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let sx = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `scalex` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `scalex` function.")),
    };

    shape.borrow_mut().scale(sx, 1.0);
    Ok(Value::Shape(shape))
}

pub fn scaley(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let sy = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `scaley` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `scaley` function.")),
    };

    shape.borrow_mut().scale(1.0, sy);
    Ok(Value::Shape(shape))
}

pub fn scaleb(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let s = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `scaleb` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `scaleb` function.")),
    };

    shape.borrow_mut().scale(s, s);
    Ok(Value::Shape(shape))
}

pub fn skew(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let kx = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `skew` function.")),
    };

    let ky = match args[1] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `skew` function.")),
    };

    let shape = match &args[2] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `skew` function.")),
    };

    shape.borrow_mut().skew(kx, ky);
    Ok(Value::Shape(shape))
}

pub fn skewx(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let kx = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `skewx` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `skewx` function.")),
    };

    shape.borrow_mut().skew(kx, 0.0);
    Ok(Value::Shape(shape))
}

pub fn skewy(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let ky = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `skewy` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `skewy` function.")),
    };

    shape.borrow_mut().skew(0.0, ky);
    Ok(Value::Shape(shape))
}

pub fn skewb(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let k = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `skewb` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `skewb` function.")),
    };

    shape.borrow_mut().skew(k, k);
    Ok(Value::Shape(shape))
}

pub fn flip(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let f = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `flip` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `flip` function.")),
    };

    shape.borrow_mut().flip(f);
    Ok(Value::Shape(shape))
}

pub fn fliph(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let shape = match &args[0] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `fliph` function.")),
    };

    shape.borrow_mut().fliph();
    Ok(Value::Shape(shape))
}

pub fn flipv(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let shape = match &args[0] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `flipv` function.")),
    };

    shape.borrow_mut().flipv();
    Ok(Value::Shape(shape))
}

pub fn flipd(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let shape = match &args[0] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `flipd` function.")),
    };

    shape.borrow_mut().flipd();
    Ok(Value::Shape(shape))
}

pub fn zindex(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let z = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `zindex` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `zindex` function.")),
    };

    shape.borrow_mut().set_zindex(z);
    Ok(Value::Shape(shape))
}

pub fn hsl(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let h = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `hsl` function.")),
    };

    let s = match args[1] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `hsl` function.")),
    };

    let l = match args[2] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `hsl` function.")),
    };

    let shape = match &args[3] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `hsl` function.")),
    };

    shape.borrow_mut().set_hsl(h, s, l);
    Ok(Value::Shape(shape))
}

pub fn hsla(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let h = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `hsla` function.")),
    };

    let s = match args[1] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `hsla` function.")),
    };

    let l = match args[2] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `hsla` function.")),
    };

    let a = match args[3] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `hsla` function.")),
    };

    let shape = match &args[4] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `hsla` function.")),
    };

    shape.borrow_mut().set_hsla(h, s, l, a);
    Ok(Value::Shape(shape))
}

pub fn hue(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let h = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `hue` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `hue` function.")),
    };

    shape.borrow_mut().set_hue(h);
    Ok(Value::Shape(shape))
}

pub fn saturation(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let s = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `saturation` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `saturation` function.")),
    };

    shape.borrow_mut().set_saturation(s);
    Ok(Value::Shape(shape))
}

pub fn lightness(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let l = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `lightness` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `lightness` function.")),
    };

    shape.borrow_mut().set_lightness(l);
    Ok(Value::Shape(shape))
}

pub fn alpha(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let a = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `alpha` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `alpha` function.")),
    };

    shape.borrow_mut().set_alpha(a);
    Ok(Value::Shape(shape))
}

pub fn hshift(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let h = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `hshift` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `hshift` function.")),
    };

    shape.borrow_mut().shift_hue(h);
    Ok(Value::Shape(shape))
}

pub fn sshift(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let s = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `sshift` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `sshift` function.")),
    };

    shape.borrow_mut().shift_saturation(s);
    Ok(Value::Shape(shape))
}

pub fn lshift(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let l = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `lshift` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `lshift` function.")),
    };

    shape.borrow_mut().shift_lightness(l);
    Ok(Value::Shape(shape))
}

pub fn ashift(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let a = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `ashift` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `ashift` function.")),
    };

    shape.borrow_mut().shift_alpha(a);
    Ok(Value::Shape(shape))
}

pub fn hex(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let hex = match args[0] {
        Value::Hex(hex) => hex,
        _ => return Err(anyhow!("Invalid type passed to `hex` function.")),
    };

    let shape = match &args[1] {
        Value::Shape(shape) => shape.clone(),
        _ => return Err(anyhow!("Invalid type passed to `hex` function.")),
    };

    shape.borrow_mut().set_hex(hex);
    Ok(Value::Shape(shape))
}

pub fn move_to(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let x = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `move_to` function.")),
    };

    let y = match args[1] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `move_to` function.")),
    };

    let segments = vec![PathSegment::MoveTo(x, y)];
    let shape = Shape::Path {
        segments,
        transform: IDENTITY,
        zindex: None,
        color: WHITE,
    };
    Ok(Value::Shape(Rc::new(RefCell::new(shape))))
}

pub fn line_to(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let x = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `line_to` function.")),
    };

    let y = match args[1] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `line_to` function.")),
    };

    let segments = vec![PathSegment::LineTo(x, y)];
    let shape = Shape::Path {
        segments,
        transform: IDENTITY,
        zindex: None,
        color: WHITE,
    };
    Ok(Value::Shape(Rc::new(RefCell::new(shape))))
}

pub fn quad_to(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let x1 = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `quad_to` function.")),
    };

    let y1 = match args[1] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `quad_to` function.")),
    };

    let x = match args[2] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `quad_to` function.")),
    };

    let y = match args[3] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `quad_to` function.")),
    };

    let segments = vec![PathSegment::QuadTo(x1, y1, x, y)];
    let shape = Shape::Path {
        segments,
        transform: IDENTITY,
        zindex: None,
        color: WHITE,
    };
    Ok(Value::Shape(Rc::new(RefCell::new(shape))))
}

pub fn cubic_to(_rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
    let x1 = match args[0] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `cubic_to` function.")),
    };

    let y1 = match args[1] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `cubic_to` function.")),
    };

    let x2 = match args[2] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `cubic_to` function.")),
    };

    let y2 = match args[3] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `cubic_to` function.")),
    };

    let x = match args[4] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `cubic_to` function.")),
    };

    let y = match args[5] {
        Value::Integer(n) => n as f32,
        Value::Float(n) => n,
        _ => return Err(anyhow!("Invalid type passed to `cubic_to` function.")),
    };

    let segments = vec![PathSegment::CubicTo(x1, y1, x2, y2, x, y)];
    let shape = Shape::Path {
        segments,
        transform: IDENTITY,
        zindex: None,
        color: WHITE,
    };
    Ok(Value::Shape(Rc::new(RefCell::new(shape))))
}

pub fn close(_rng: &mut ChaCha8Rng, _args: &[Value]) -> Result<Value> {
    let shape = Shape::Path {
        segments: vec![PathSegment::Close],
        transform: IDENTITY,
        zindex: None,
        color: WHITE,
    };
    Ok(Value::Shape(Rc::new(RefCell::new(shape))))
}
