#[cfg(feature = "std")]
use std::rc::Rc;

#[cfg(feature = "no-std")]
use alloc::{rc::Rc, vec::Vec};

use crate::builtin_function;
use crate::interpreter::Value;
use crate::shape::{HslaChange, Shape, IDENTITY, WHITE};
use core::cell::RefCell;

use anyhow::{anyhow, Result};
use rand_chacha::ChaCha8Rng;
use tiny_skia::BlendMode;

builtin_function!(compose => {
    [Value::Shape(a), Value::Shape(b)] => {
        let shape = match (&*a.borrow(), &*b.borrow()) {
            (
                Shape::Path {
                    segments: a,
                    transform: a_transform,
                    zindex,
                    color,
                    blend_mode,
                    anti_alias,
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
                    blend_mode: *blend_mode,
                    anti_alias: *anti_alias,
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
                blend_mode_overwrite: None,
                anti_alias_overwrite: None,
            },
        };
        Value::Shape(Rc::new(RefCell::new(shape)))
    }
});

builtin_function!(collect => {
    [Value::List(list)] => {
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
            let mut blend_mode = BlendMode::SourceOver;
            let mut anti_alias = true;

            for path in shapes {
                match &*path.borrow() {
                    Shape::Path {
                        segments: other_segments,
                        transform: other_transform,
                        zindex: other_zindex,
                        color: other_color,
                        blend_mode: other_blend_mode,
                        anti_alias: other_anti_alias,
                    } => {
                        segments.extend(other_segments);
                        transform = transform.post_concat(*other_transform);
                        zindex = *other_zindex;
                        color = *other_color;
                        blend_mode = *other_blend_mode;
                        anti_alias = *other_anti_alias;
                    }
                    _ => unreachable!(),
                }
            }

            Shape::Path {
                segments,
                transform,
                zindex,
                color,
                blend_mode,
                anti_alias,
            }
        } else {
            Shape::Collection {
                shapes,
                transform: IDENTITY,
                zindex_overwrite: None,
                zindex_shift: None,
                color_overwrite: HslaChange::default(),
                color_shift: HslaChange::default(),
                blend_mode_overwrite: None,
                anti_alias_overwrite: None,
            }
        };
        Value::Shape(Rc::new(RefCell::new(shape)))
    }
});
