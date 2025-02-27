#[cfg(feature = "std")]
use std::sync::{Arc, Mutex, MutexGuard};

#[cfg(feature = "no-std")]
use {
    alloc::{sync::Arc, vec::Vec},
    spin::{Mutex, MutexGuard},
};

use palette::{Hsla, RgbHue};
use tiny_skia::Transform;

pub static IDENTITY: Transform = Transform {
    sx: 1.0,
    kx: 0.0,
    ky: 0.0,
    sy: 1.0,
    tx: 0.0,
    ty: 0.0,
};

pub static WHITE: Hsla<f32> = Hsla::new_const(RgbHue::new(360.0), 0.0, 1.0, 1.0);

pub static TRANSPARENT: Hsla<f32> = Hsla::new_const(RgbHue::new(360.0), 0.0, 1.0, 0.0);

pub static SQUARE: BasicShape = BasicShape::Square {
    x: -1.0,
    y: -1.0,
    width: 2.0,
    height: 2.0,
    transform: IDENTITY,
    color: WHITE,
};

pub static CIRCLE: BasicShape = BasicShape::Circle {
    x: 0.0,
    y: 0.0,
    radius: 1.0,
    transform: IDENTITY,
    color: WHITE,
};

pub static TRIANGLE: BasicShape = BasicShape::Triangle {
    points: [-1.0, 0.577350269, 1.0, 0.577350269, 0.0, -1.154700538],
    transform: IDENTITY,
    color: WHITE,
};

pub static FILL: BasicShape = BasicShape::Fill { color: WHITE };

#[derive(Debug, Clone, Copy)]
pub enum BasicShape {
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
}

#[derive(Debug, Clone)]
pub enum Shape {
    Basic(BasicShape),
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
            Self::Basic(BasicShape::Square { transform, .. })
            | Self::Basic(BasicShape::Circle { transform, .. })
            | Self::Basic(BasicShape::Triangle { transform, .. })
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform.post_translate(tx, ty);
            }
            Self::Basic(BasicShape::Fill { .. }) => (),
        }
    }

    pub fn rotate(&mut self, r: f32) {
        match self {
            Self::Basic(BasicShape::Square { transform, .. })
            | Self::Basic(BasicShape::Circle { transform, .. })
            | Self::Basic(BasicShape::Triangle { transform, .. })
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform.post_rotate(r);
            }
            Self::Basic(BasicShape::Fill { .. }) => (),
        }
    }

    pub fn rotate_at(&mut self, r: f32, tx: f32, ty: f32) {
        match self {
            Self::Basic(BasicShape::Square { transform, .. })
            | Self::Basic(BasicShape::Circle { transform, .. })
            | Self::Basic(BasicShape::Triangle { transform, .. })
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform.post_rotate_at(r, tx, ty);
            }
            Self::Basic(BasicShape::Fill { .. }) => (),
        }
    }

    pub fn scale(&mut self, sx: f32, sy: f32) {
        match self {
            Self::Basic(BasicShape::Square { transform, .. })
            | Self::Basic(BasicShape::Circle { transform, .. })
            | Self::Basic(BasicShape::Triangle { transform, .. })
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform.post_scale(sx, sy);
            }
            Self::Basic(BasicShape::Fill { .. }) => (),
        }
    }

    pub fn skew(&mut self, kx: f32, ky: f32) {
        match self {
            Self::Basic(BasicShape::Square { transform, .. })
            | Self::Basic(BasicShape::Circle { transform, .. })
            | Self::Basic(BasicShape::Triangle { transform, .. })
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform.post_concat(Transform::from_skew(kx, ky));
            }
            Self::Basic(BasicShape::Fill { .. }) => (),
        }
    }

    pub fn flip(&mut self, f: f32) {
        match self {
            Self::Basic(BasicShape::Square { transform, .. })
            | Self::Basic(BasicShape::Circle { transform, .. })
            | Self::Basic(BasicShape::Triangle { transform, .. })
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform.post_rotate(f).post_scale(-1.0, 1.0).post_rotate(-f);
            }
            Self::Basic(BasicShape::Fill { .. }) => (),
        }
    }

    pub fn fliph(&mut self) {
        match self {
            Self::Basic(BasicShape::Square { transform, .. })
            | Self::Basic(BasicShape::Circle { transform, .. })
            | Self::Basic(BasicShape::Triangle { transform, .. })
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform.post_scale(-1.0, 1.0);
            }
            Self::Basic(BasicShape::Fill { .. }) => (),
        }
    }

    pub fn flipv(&mut self) {
        match self {
            Self::Basic(BasicShape::Square { transform, .. })
            | Self::Basic(BasicShape::Circle { transform, .. })
            | Self::Basic(BasicShape::Triangle { transform, .. })
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform.post_scale(1.0, -1.0);
            }
            Self::Basic(BasicShape::Fill { .. }) => (),
        }
    }

    pub fn flipd(&mut self) {
        match self {
            Self::Basic(BasicShape::Square { transform, .. })
            | Self::Basic(BasicShape::Circle { transform, .. })
            | Self::Basic(BasicShape::Triangle { transform, .. })
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform.post_scale(-1.0, -1.0);
            }
            Self::Basic(BasicShape::Fill { .. }) => (),
        }
    }

    pub fn set_hue(&mut self, hue: f32) {
        match self {
            Self::Basic(BasicShape::Square { color, .. })
            | Self::Basic(BasicShape::Circle { color, .. })
            | Self::Basic(BasicShape::Triangle { color, .. })
            | Self::Basic(BasicShape::Fill { color })
            | Self::Composite { color, .. }
            | Self::Collection { color, .. } => {
                color.hue = hue.into();
            }
        }
    }

    pub fn set_saturation(&mut self, saturation: f32) {
        match self {
            Self::Basic(BasicShape::Square { color, .. })
            | Self::Basic(BasicShape::Circle { color, .. })
            | Self::Basic(BasicShape::Triangle { color, .. })
            | Self::Basic(BasicShape::Fill { color })
            | Self::Composite { color, .. }
            | Self::Collection { color, .. } => {
                color.saturation = saturation;
            }
        }
    }

    pub fn set_lightness(&mut self, lightness: f32) {
        match self {
            Self::Basic(BasicShape::Square { color, .. })
            | Self::Basic(BasicShape::Circle { color, .. })
            | Self::Basic(BasicShape::Triangle { color, .. })
            | Self::Basic(BasicShape::Fill { color })
            | Self::Composite { color, .. }
            | Self::Collection { color, .. } => {
                color.lightness = lightness;
            }
        }
    }

    pub fn set_alpha(&mut self, alpha: f32) {
        match self {
            Self::Basic(BasicShape::Square { color, .. })
            | Self::Basic(BasicShape::Circle { color, .. })
            | Self::Basic(BasicShape::Triangle { color, .. })
            | Self::Basic(BasicShape::Fill { color })
            | Self::Composite { color, .. }
            | Self::Collection { color, .. } => {
                color.alpha = alpha;
            }
        }
    }
}

pub fn unwrap_shape(shape: Arc<Mutex<Shape>>) -> Shape {
    #[cfg(feature = "std")]
    return Arc::try_unwrap(shape).unwrap().into_inner().unwrap();
    #[cfg(feature = "no-std")]
    return Arc::try_unwrap(shape).unwrap().into_inner();
}

pub fn lock_shape<'a>(shape: &'a Arc<Mutex<Shape>>) -> MutexGuard<'a, Shape> {
    #[cfg(feature = "std")]
    return shape.lock().unwrap();
    #[cfg(feature = "no-std")]
    return shape.lock();
}
