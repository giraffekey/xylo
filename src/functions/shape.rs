#[cfg(feature = "std")]
use std::rc::Rc;

#[cfg(feature = "no-std")]
use alloc::{rc::Rc, vec, vec::Vec};

use crate::builtin_function;
use crate::error::{Error, Result};
use crate::functions::dedup_shape;
use crate::interpreter::{Data, Value};
use crate::shape::{Color, ColorChange, HslaChange, PathSegment, Shape, Style, IDENTITY, WHITE};
use core::cell::RefCell;

use rand_chacha::ChaCha8Rng;
use tiny_skia::{BlendMode, FillRule, StrokeDash};

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
                    style,
                    mask,
                    pattern,
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
                    color: color.clone(),
                    blend_mode: *blend_mode,
                    anti_alias: *anti_alias,
                    style: style.clone(),
                    mask: mask.clone(),
                    pattern: pattern.clone(),
                }
            }
            _ => {
                let b = if Rc::ptr_eq(a, b) {
                    Rc::new(RefCell::new(b.borrow().clone()))
                } else {
                    b.clone()
                };

                Shape::composite(a.clone(), b)
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
                 _ => return Err(Error::InvalidArgument("collect".into())),
            })
            .collect();
        let shapes = shapes?;

        if shapes.len() < 1 {
            return Ok(Value::Shape(Rc::new(RefCell::new(Shape::empty()))));
        }

        let is_path = shapes.iter().all(|shape| match &*shape.borrow() {
            Shape::Path { .. } => true,
            _ => false,
        });
        let shape = if is_path {
            let mut segments = Vec::with_capacity(shapes.len());
            let mut transform = IDENTITY;
            let mut zindex = None;
            let mut color = Color::Solid(WHITE);
            let mut blend_mode = BlendMode::SourceOver;
            let mut anti_alias = true;
            let mut style = Style::default();
            let mut mask = None;
            let mut pattern = None;

            for path in shapes {
                match &*path.borrow() {
                    Shape::Path {
                        segments: other_segments,
                        transform: other_transform,
                        zindex: other_zindex,
                        color: other_color,
                        blend_mode: other_blend_mode,
                        anti_alias: other_anti_alias,
                        style: other_style,
                        mask: other_mask,
                        pattern: other_pattern,
                    } => {
                        segments.extend(other_segments);
                        transform = transform.post_concat(*other_transform);
                        zindex = *other_zindex;
                        color = other_color.clone();
                        blend_mode = *other_blend_mode;
                        anti_alias = *other_anti_alias;
                        style = other_style.clone();
                        mask = other_mask.clone();
                        pattern = other_pattern.clone();
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
                style,
                mask,
                pattern,
            }
        } else {
            Shape::collection(shapes)
        };
        Value::Shape(Rc::new(RefCell::new(shape)))
    }
});

builtin_function!(blend => {
    [Value::BlendMode(blend_mode), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().set_blend_mode(*blend_mode);
        Value::Shape(shape.clone())
    }
});

builtin_function!(anti_alias => {
    [Value::Boolean(anti_alias), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().set_anti_alias(*anti_alias);
        Value::Shape(shape.clone())
    }
});

builtin_function!(fill => {
    [Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().set_fill_rule(FillRule::Winding);
        Value::Shape(shape.clone())
    }
});

builtin_function!(winding => {
    [Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().set_fill_rule(FillRule::Winding);
        Value::Shape(shape.clone())
    }
});

builtin_function!(even_odd => {
    [Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().set_fill_rule(FillRule::EvenOdd);
        Value::Shape(shape.clone())
    }
});

builtin_function!(stroke => {
    [width, Value::Shape(shape)] => {
        let width = match width {
            Value::Integer(width) => *width as f32,
            Value::Float(width)   => *width,
            _ => return Err(Error::InvalidArgument("stroke".into())),
        };

        let shape = dedup_shape(shape);
        shape.borrow_mut().set_stroke_width(width);
        Value::Shape(shape.clone())
    }
});

builtin_function!(miter_limit => {
    [n, Value::Shape(shape)] => {
        let n = match n {
            Value::Integer(n) => *n as f32,
            Value::Float(n)   => *n,
            _ => return Err(Error::InvalidArgument("miter_limit".into())),
        };

        let shape = dedup_shape(shape);
        shape.borrow_mut().set_miter_limit(n);
        Value::Shape(shape.clone())
    }
});

builtin_function!(line_cap => {
    [Value::LineCap(lc), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().set_line_cap(*lc);
        Value::Shape(shape.clone())
    }
});

builtin_function!(line_join => {
    [Value::LineJoin(lj), Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().set_line_join(*lj);
        Value::Shape(shape.clone())
    }
});

builtin_function!(dash => {
    [Value::List(array), offset, Value::Shape(shape)] => {
        let array = match array.get(0) {
            Some(Value::Integer(_)) => array.iter().map(|value| match value {
                Value::Integer(n) => *n as f32,
                _ => unreachable!(),
            }).collect(),
            Some(Value::Float(_)) => array.iter().map(|value| match value {
                Value::Float(n) => *n,
                _ => unreachable!(),
            }).collect(),
            _ => return Err(Error::InvalidArgument("dash".into())),
        };

        let offset = match offset {
            Value::Integer(offset) => *offset as f32,
            Value::Float(offset)   => *offset,
            _ => return Err(Error::InvalidArgument("dash".into())),
        };

        let shape = dedup_shape(shape);
        shape.borrow_mut().set_dash(StrokeDash::new(array, offset));
        Value::Shape(shape.clone())
    }
});

builtin_function!(no_dash => {
    [Value::Shape(shape)] => {
        let shape = dedup_shape(shape);
        shape.borrow_mut().set_dash(None);
        Value::Shape(shape.clone())
    }
});

builtin_function!(mask => {
    [Value::Shape(mask), Value::Shape(shape)] => {
        let mask = dedup_shape(mask);
        let shape = dedup_shape(shape);
        shape.borrow_mut().set_mask(mask.clone());
        Value::Shape(shape.clone())
    }
});

builtin_function!(pattern => {
    [Value::Shape(pattern), Value::SpreadMode(sm), Value::Shape(shape)] => {
        let pattern = dedup_shape(pattern);
        let shape = dedup_shape(shape);
        shape.borrow_mut().set_pattern(pattern.clone(), *sm);
        Value::Shape(shape.clone())
    }
});

builtin_function!(voronoi => {
    [Value::List(sites), boxsize] => {
        use voronoi::{voronoi, make_polygons, Point};

        let sites = match sites.get(0) {
            Some(Value::List(_)) => sites.iter().map(|value| match value {
                Value::List(point) => match point[..] {
                    [Value::Integer(x), Value::Integer(y)] => Ok(Point::new(x as f64, y as f64)),
                    [Value::Float(x), Value::Float(y)] => Ok(Point::new(x as f64, y as f64)),
                    _ => Err(Error::InvalidArgument("voronoi".into())),
                }
                    _ => Err(Error::InvalidArgument("voronoi".into())),
            }).collect::<Result<Vec<_>>>()?,
            _ => return Err(Error::InvalidArgument("voronoi".into())),
        };

        let boxsize = match boxsize {
            Value::Integer(boxsize) => *boxsize as f64,
            Value::Float(boxsize)   => *boxsize as f64,
            _ => return Err(Error::InvalidArgument("voronoi".into())),
        };

        let polygons = make_polygons(&voronoi(sites, boxsize));
        let shapes = polygons.iter().map(|points| {
            let mut segments = vec![PathSegment::MoveTo((*points[0].x) as f32, (*points[0].y) as f32)];
            for point in &points[1..] {
                segments.push(PathSegment::LineTo((*point.x) as f32, (*point.y) as f32));
            }
            segments.push(PathSegment::Close);

            Value::Shape(Rc::new(RefCell::new(Shape::Composite {
                a: Rc::new(RefCell::new(Shape::empty())),
                b: Rc::new(RefCell::new(Shape::path(segments))),
                transform: IDENTITY.post_translate(-boxsize as f32 / 2.0, -boxsize as f32 / 2.0),
                zindex_overwrite: None,
                zindex_shift: None,
                color_overwrite: ColorChange::default(),
                color_shift: HslaChange::default(),
                blend_mode_overwrite: None,
                anti_alias_overwrite: None,
                style_overwrite: None,
                mask_overwrite: None,
                pattern_overwrite: None,
            })))
        }).collect();

        Value::List(shapes)
    }
});
