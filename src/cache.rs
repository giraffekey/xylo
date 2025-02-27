#[cfg(feature = "std")]
use std::sync::{Arc, Mutex};

#[cfg(feature = "no-std")]
use {
    alloc::{boxed::Box, sync::Arc, vec::Vec},
    spin::Mutex,
};

use crate::compiler::Value;
use crate::shape::{lock_shape, BasicShape, Shape};

use ahash::AHasher;
use core::hash::Hasher;
use dashmap::DashMap;
use serde::Serialize;
use tiny_skia::Transform;

#[derive(Debug, Clone, Copy, Serialize)]
pub enum CacheBasicShape {
    Square {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        transform: [f32; 6],
        color: [f32; 4],
    },
    Circle {
        x: f32,
        y: f32,
        radius: f32,
        transform: [f32; 6],
        color: [f32; 4],
    },
    Triangle {
        points: [f32; 6],
        transform: [f32; 6],
        color: [f32; 4],
    },
    Fill {
        color: [f32; 4],
    },
}

impl From<&BasicShape> for CacheBasicShape {
    fn from(shape: &BasicShape) -> CacheBasicShape {
        match shape {
            BasicShape::Square {
                x,
                y,
                width,
                height,
                transform,
                color,
            } => CacheBasicShape::Square {
                x: *x,
                y: *y,
                width: *width,
                height: *height,
                transform: transform_to_data(*transform),
                color: (*color).into(),
            },
            BasicShape::Circle {
                x,
                y,
                radius,
                transform,
                color,
            } => CacheBasicShape::Circle {
                x: *x,
                y: *y,
                radius: *radius,
                transform: transform_to_data(*transform),
                color: (*color).into(),
            },
            BasicShape::Triangle {
                points,
                transform,
                color,
            } => CacheBasicShape::Triangle {
                points: *points,
                transform: transform_to_data(*transform),
                color: (*color).into(),
            },
            BasicShape::Fill { color } => CacheBasicShape::Fill {
                color: (*color).into(),
            },
        }
    }
}

impl Into<BasicShape> for CacheBasicShape {
    fn into(self) -> BasicShape {
        match self {
            CacheBasicShape::Square {
                x,
                y,
                width,
                height,
                transform,
                color,
            } => BasicShape::Square {
                x,
                y,
                width,
                height,
                transform: transform_from_data(transform),
                color: color.into(),
            },
            CacheBasicShape::Circle {
                x,
                y,
                radius,
                transform,
                color,
            } => BasicShape::Circle {
                x,
                y,
                radius,
                transform: transform_from_data(transform),
                color: color.into(),
            },
            CacheBasicShape::Triangle {
                points,
                transform,
                color,
            } => BasicShape::Triangle {
                points,
                transform: transform_from_data(transform),
                color: color.into(),
            },
            CacheBasicShape::Fill { color } => BasicShape::Fill {
                color: color.into(),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum CacheShape {
    Basic(CacheBasicShape),
    Composite {
        a: Box<CacheShape>,
        b: Box<CacheShape>,
        transform: [f32; 6],
        color: [f32; 4],
    },
    Collection {
        shapes: Vec<CacheShape>,
        transform: [f32; 6],
        color: [f32; 4],
    },
}

impl From<&Shape> for CacheShape {
    fn from(shape: &Shape) -> CacheShape {
        match shape {
            Shape::Basic(s) => CacheShape::Basic(s.into()),
            Shape::Composite {
                a,
                b,
                transform,
                color,
            } => CacheShape::Composite {
                a: Box::new((&*lock_shape(a)).into()),
                b: Box::new((&*lock_shape(b)).into()),
                transform: transform_to_data(*transform),
                color: (*color).into(),
            },
            Shape::Collection {
                shapes,
                transform,
                color,
            } => CacheShape::Collection {
                shapes: shapes
                    .iter()
                    .map(|shape| (&*lock_shape(shape)).into())
                    .collect(),
                transform: transform_to_data(*transform),
                color: (*color).into(),
            },
        }
    }
}

impl Into<Shape> for CacheShape {
    fn into(self) -> Shape {
        match self {
            CacheShape::Basic(s) => Shape::Basic(s.into()),
            CacheShape::Composite {
                a,
                b,
                transform,
                color,
            } => {
                let a = Arc::new(Mutex::new((*a).into()));
                let b = Arc::new(Mutex::new((*b).into()));
                Shape::Composite {
                    a,
                    b,
                    transform: transform_from_data(transform),
                    color: color.into(),
                }
            }
            CacheShape::Collection {
                shapes,
                transform,
                color,
            } => Shape::Collection {
                shapes: shapes
                    .iter()
                    .cloned()
                    .map(|s| Arc::new(Mutex::new(s.into())))
                    .collect(),
                transform: transform_from_data(transform),
                color: color.into(),
            },
        }
    }
}

fn transform_to_data(transform: Transform) -> [f32; 6] {
    [
        transform.sx,
        transform.kx,
        transform.ky,
        transform.sy,
        transform.tx,
        transform.ty,
    ]
}

fn transform_from_data(data: [f32; 6]) -> Transform {
    Transform {
        sx: data[0],
        kx: data[1],
        ky: data[2],
        sy: data[3],
        tx: data[4],
        ty: data[5],
    }
}

#[derive(Debug, Clone, Serialize)]
enum CacheValue {
    Integer(i32),
    Float(f32),
    Boolean(bool),
    Shape(CacheShape),
    List(Vec<CacheValue>),
}

impl From<&Value> for CacheValue {
    fn from(value: &Value) -> CacheValue {
        match value {
            Value::Integer(n) => CacheValue::Integer(*n),
            Value::Float(n) => CacheValue::Float(*n),
            Value::Boolean(b) => CacheValue::Boolean(*b),
            Value::Shape(s) => CacheValue::Shape((&*lock_shape(s)).into()),
            Value::List(list) => CacheValue::List(list.iter().map(CacheValue::from).collect()),
        }
    }
}

impl Into<Value> for CacheValue {
    fn into(self) -> Value {
        match self {
            CacheValue::Integer(n) => Value::Integer(n),
            CacheValue::Float(n) => Value::Float(n),
            CacheValue::Boolean(b) => Value::Boolean(b),
            CacheValue::Shape(s) => Value::Shape(Arc::new(Mutex::new(s.into()))),
            CacheValue::List(list) => {
                Value::List(list.iter().cloned().map(CacheValue::into).collect())
            }
        }
    }
}

#[derive(Debug)]
pub struct Cache {
    call_results: DashMap<u64, CacheValue>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            call_results: DashMap::new(),
        }
    }

    pub fn hash_call(name: &str, args: &[Value]) -> u64 {
        let args: Vec<CacheValue> = args.iter().map(CacheValue::from).collect();

        let mut buf = Vec::new();
        buf.extend(name.as_bytes());
        buf.extend(bincode::serialize(&args).unwrap());

        let mut hasher = AHasher::default();
        hasher.write(&buf);
        hasher.finish()
    }

    pub fn get(&self, key: u64) -> Option<Value> {
        self.call_results
            .get(&key)
            .map(|r| r.value().clone().into())
    }

    pub fn insert(&self, key: u64, value: &Value) {
        self.call_results.insert(key, value.into());
    }
}
