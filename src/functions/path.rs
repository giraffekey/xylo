#[cfg(feature = "std")]
use std::rc::Rc;

#[cfg(feature = "no-std")]
use alloc::{rc::Rc, vec, vec::Vec};

use crate::builtin_function;
use crate::interpreter::Value;
use crate::shape::{PathSegment, Shape, IDENTITY, WHITE};
use core::cell::RefCell;

use anyhow::{anyhow, Result};
use rand_chacha::ChaCha8Rng;

builtin_function!(move_to => {
    [x, y] => {
        let x = match x {
            Value::Integer(x) => *x as f32,
            Value::Float(x) => *x,
            _ => return Err(anyhow!("Invalid type passed to `move_to` function.")),
        };

        let y = match y {
            Value::Integer(y) => *y as f32,
            Value::Float(y) => *y,
            _ => return Err(anyhow!("Invalid type passed to `move_to` function.")),
        };

        let segments = vec![PathSegment::MoveTo(x, y)];
        let shape = Shape::Path {
            segments,
            transform: IDENTITY,
            zindex: None,
            color: WHITE,
        };
        Value::Shape(Rc::new(RefCell::new(shape)))
    }
});

builtin_function!(line_to => {
    [x, y] => {
        let x = match x {
            Value::Integer(x) => *x as f32,
            Value::Float(x) => *x,
            _ => return Err(anyhow!("Invalid type passed to `line_to` function.")),
        };

        let y = match y {
            Value::Integer(y) => *y as f32,
            Value::Float(y) => *y,
            _ => return Err(anyhow!("Invalid type passed to `line_to` function.")),
        };

        let segments = vec![PathSegment::LineTo(x, y)];
        let shape = Shape::Path {
            segments,
            transform: IDENTITY,
            zindex: None,
            color: WHITE,
        };
        Value::Shape(Rc::new(RefCell::new(shape)))
    }
});

builtin_function!(quad_to => {
    [x1, y1, x, y] => {
        let x1 = match x1 {
            Value::Integer(x1) => *x1 as f32,
            Value::Float(x1) => *x1,
            _ => return Err(anyhow!("Invalid type passed to `quad_to` function.")),
        };

        let y1 = match y1 {
            Value::Integer(y1) => *y1 as f32,
            Value::Float(y1) => *y1,
            _ => return Err(anyhow!("Invalid type passed to `quad_to` function.")),
        };

        let x = match x {
            Value::Integer(x) => *x as f32,
            Value::Float(x) => *x,
            _ => return Err(anyhow!("Invalid type passed to `quad_to` function.")),
        };

        let y = match y {
            Value::Integer(y) => *y as f32,
            Value::Float(y) => *y,
            _ => return Err(anyhow!("Invalid type passed to `quad_to` function.")),
        };

        let segments = vec![PathSegment::QuadTo(x1, y1, x, y)];
        let shape = Shape::Path {
            segments,
            transform: IDENTITY,
            zindex: None,
            color: WHITE,
        };
        Value::Shape(Rc::new(RefCell::new(shape)))
    }
});

builtin_function!(cubic_to => {
    [x1, y1, x2, y2, x, y] => {
        let x1 = match x1 {
            Value::Integer(x1) => *x1 as f32,
            Value::Float(x1) => *x1,
            _ => return Err(anyhow!("Invalid type passed to `cubic_to` function.")),
        };

        let y1 = match y1 {
            Value::Integer(y1) => *y1 as f32,
            Value::Float(y1) => *y1,
            _ => return Err(anyhow!("Invalid type passed to `cubic_to` function.")),
        };

        let x2 = match x2 {
            Value::Integer(x2) => *x2 as f32,
            Value::Float(x2) => *x2,
            _ => return Err(anyhow!("Invalid type passed to `cubic_to` function.")),
        };

        let y2 = match y2 {
            Value::Integer(y2) => *y2 as f32,
            Value::Float(y2) => *y2,
            _ => return Err(anyhow!("Invalid type passed to `cubic_to` function.")),
        };

        let x = match x {
            Value::Integer(x) => *x as f32,
            Value::Float(x) => *x,
            _ => return Err(anyhow!("Invalid type passed to `cubic_to` function.")),
        };

        let y = match y {
            Value::Integer(y) => *y as f32,
            Value::Float(y) => *y,
            _ => return Err(anyhow!("Invalid type passed to `cubic_to` function.")),
        };

        let segments = vec![PathSegment::CubicTo(x1, y1, x2, y2, x, y)];
        let shape = Shape::Path {
            segments,
            transform: IDENTITY,
            zindex: None,
            color: WHITE,
        };
        Value::Shape(Rc::new(RefCell::new(shape)))
    }
});

builtin_function!(close => {
    [] => {
        let segments = vec![PathSegment::Close];
        let shape = Shape::Path {
            segments,
            transform: IDENTITY,
            zindex: None,
            color: WHITE,
        };
        Value::Shape(Rc::new(RefCell::new(shape)))
    }
});
