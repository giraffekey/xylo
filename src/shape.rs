#[cfg(feature = "std")]
use std::sync::{Arc, Mutex, MutexGuard};

#[cfg(feature = "no-std")]
use {
    alloc::{sync::Arc, vec::Vec},
    spin::{Mutex, MutexGuard},
};

use anyhow::Result;
use palette::Hsla;
use tiny_skia::Transform;

#[derive(Debug, Clone)]
pub enum Shape {
    Square {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        transform: Transform,
        color: Hsla<f32>,
    },
    Circle {
        x: f32,
        y: f32,
        radius: f32,
        transform: Transform,
        color: Hsla<f32>,
    },
    Triangle {
        points: [f32; 6],
        transform: Transform,
        color: Hsla<f32>,
    },
    Fill {
        color: Hsla<f32>,
    },
    Composite {
        a: Arc<Mutex<Shape>>,
        b: Arc<Mutex<Shape>>,
        transform: Transform,
        color: Hsla<f32>,
    },
    Collection {
        shapes: Vec<Arc<Mutex<Shape>>>,
        transform: Transform,
        color: Hsla<f32>,
    },
}

impl Shape {
    pub fn translate(&mut self, tx: f32, ty: f32) {
        match self {
            Self::Square { transform, .. }
            | Self::Circle { transform, .. }
            | Self::Triangle { transform, .. }
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform.post_translate(tx, ty);
            }
            Self::Fill { .. } => (),
        }
    }

    pub fn rotate(&mut self, r: f32) {
        match self {
            Self::Square { transform, .. }
            | Self::Circle { transform, .. }
            | Self::Triangle { transform, .. }
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform.post_rotate(r);
            }
            Self::Fill { .. } => (),
        }
    }

    pub fn rotate_at(&mut self, r: f32, tx: f32, ty: f32) {
        match self {
            Self::Square { transform, .. }
            | Self::Circle { transform, .. }
            | Self::Triangle { transform, .. }
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform.post_rotate_at(r, tx, ty);
            }
            Self::Fill { .. } => (),
        }
    }

    pub fn scale(&mut self, sx: f32, sy: f32) {
        match self {
            Self::Square { transform, .. }
            | Self::Circle { transform, .. }
            | Self::Triangle { transform, .. }
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform.post_scale(sx, sy);
            }
            Self::Fill { .. } => (),
        }
    }

    pub fn skew(&mut self, kx: f32, ky: f32) {
        match self {
            Self::Square { transform, .. }
            | Self::Circle { transform, .. }
            | Self::Triangle { transform, .. }
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform.post_concat(Transform::from_skew(kx, ky));
            }
            Self::Fill { .. } => (),
        }
    }

    pub fn flip(&mut self, _f: f32) {
        todo!()
    }

    pub fn set_hue(&mut self, hue: f32) {
        match self {
            Self::Square { color, .. }
            | Self::Circle { color, .. }
            | Self::Triangle { color, .. }
            | Self::Composite { color, .. }
            | Self::Collection { color, .. }
            | Self::Fill { color } => {
                color.hue = hue.into();
            }
        }
    }

    pub fn set_saturation(&mut self, saturation: f32) {
        match self {
            Self::Square { color, .. }
            | Self::Circle { color, .. }
            | Self::Triangle { color, .. }
            | Self::Composite { color, .. }
            | Self::Collection { color, .. }
            | Self::Fill { color } => {
                color.saturation = saturation;
            }
        }
    }

    pub fn set_lightness(&mut self, lightness: f32) {
        match self {
            Self::Square { color, .. }
            | Self::Circle { color, .. }
            | Self::Triangle { color, .. }
            | Self::Composite { color, .. }
            | Self::Collection { color, .. }
            | Self::Fill { color } => {
                color.lightness = lightness;
            }
        }
    }

    pub fn set_alpha(&mut self, alpha: f32) {
        match self {
            Self::Square { color, .. }
            | Self::Circle { color, .. }
            | Self::Triangle { color, .. }
            | Self::Composite { color, .. }
            | Self::Collection { color, .. }
            | Self::Fill { color } => {
                color.alpha = alpha;
            }
        }
    }
}

pub fn unwrap_shape(shape: Arc<Mutex<Shape>>) -> Result<Shape> {
    #[cfg(feature = "std")]
    return Ok(Arc::try_unwrap(shape).unwrap().into_inner()?);
    #[cfg(feature = "no-std")]
    return Ok(Arc::try_unwrap(shape).unwrap().into_inner());
}

pub fn lock_shape<'a>(shape: &'a Arc<Mutex<Shape>>) -> MutexGuard<'a, Shape> {
    #[cfg(feature = "std")]
    return shape.lock().unwrap();
    #[cfg(feature = "no-std")]
    return shape.lock();
}
