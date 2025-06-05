#[cfg(feature = "std")]
use std::rc::Rc;

#[cfg(feature = "no-std")]
use alloc::{rc::Rc, vec};

use crate::builtin_function;
use crate::error::{Error, Result};
use crate::interpreter::{Data, Value};
use crate::shape::{PathSegment, Shape, IDENTITY, WHITE};
use core::cell::RefCell;

use rand_chacha::ChaCha8Rng;
use tiny_skia::BlendMode;

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
        let shape = Shape::Path {
            segments,
            transform: IDENTITY,
            zindex: None,
            color: WHITE,
            blend_mode: BlendMode::SourceOver,
            anti_alias: true,
        };
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
        let shape = Shape::Path {
            segments,
            transform: IDENTITY,
            zindex: None,
            color: WHITE,
            blend_mode: BlendMode::SourceOver,
            anti_alias: true,
        };
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
        let shape = Shape::Path {
            segments,
            transform: IDENTITY,
            zindex: None,
            color: WHITE,
            blend_mode: BlendMode::SourceOver,
            anti_alias: true,
        };
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
        let shape = Shape::Path {
            segments,
            transform: IDENTITY,
            zindex: None,
            color: WHITE,
            blend_mode: BlendMode::SourceOver,
            anti_alias: true,
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
            blend_mode: BlendMode::SourceOver,
            anti_alias: true,
        };
        Value::Shape(Rc::new(RefCell::new(shape)))
    }
});
