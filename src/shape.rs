#[cfg(feature = "std")]
use std::rc::Rc;

#[cfg(feature = "no-std")]
use alloc::{rc::Rc, vec::Vec};

use core::cell::RefCell;
use palette::{rgb::Rgb, FromColor, Hsl, Hsla, RgbHue};
use tiny_skia::Transform;

pub static IDENTITY: Transform = Transform {
    sx: 1.0,
    kx: 0.0,
    ky: 0.0,
    sy: 1.0,
    tx: 0.0,
    ty: 0.0,
};

pub static WHITE: Hsla<f32> = Hsla::new_const(RgbHue::new(360.0), 1.0, 1.0, 1.0);

pub static TRANSPARENT: Hsla<f32> = Hsla::new_const(RgbHue::new(360.0), 1.0, 1.0, 0.0);

pub static SQUARE: BasicShape = BasicShape::Square {
    x: -1.0,
    y: -1.0,
    width: 2.0,
    height: 2.0,
    transform: IDENTITY,
    zindex: None,
    color: WHITE,
};

pub static CIRCLE: BasicShape = BasicShape::Circle {
    x: 0.0,
    y: 0.0,
    radius: 1.0,
    transform: IDENTITY,
    zindex: None,
    color: WHITE,
};

pub static TRIANGLE: BasicShape = BasicShape::Triangle {
    points: [-1.0, 0.577350269, 1.0, 0.577350269, 0.0, -1.154700538],
    transform: IDENTITY,
    zindex: None,
    color: WHITE,
};

pub static FILL: BasicShape = BasicShape::Fill {
    zindex: None,
    color: WHITE,
};

pub static EMPTY: BasicShape = BasicShape::Empty;

#[derive(Debug, Clone, Copy)]
pub enum PathSegment {
    MoveTo(f32, f32),
    LineTo(f32, f32),
    QuadTo(f32, f32, f32, f32),
    CubicTo(f32, f32, f32, f32, f32, f32),
    Close,
}

#[derive(Debug, Clone, Copy)]
pub enum BasicShape {
    Square {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        transform: Transform,
        zindex: Option<f32>,
        color: Hsla<f32>,
    },
    Circle {
        x: f32,
        y: f32,
        radius: f32,
        transform: Transform,
        zindex: Option<f32>,
        color: Hsla<f32>,
    },
    Triangle {
        points: [f32; 6],
        transform: Transform,
        zindex: Option<f32>,
        color: Hsla<f32>,
    },
    Fill {
        zindex: Option<f32>,
        color: Hsla<f32>,
    },
    Empty,
}

#[derive(Debug, Clone)]
pub enum Shape {
    Basic(BasicShape),
    Path {
        segments: Vec<PathSegment>,
        transform: Transform,
        zindex: Option<f32>,
        color: Hsla<f32>,
    },
    Composite {
        a: Rc<RefCell<Shape>>,
        b: Rc<RefCell<Shape>>,
        transform: Transform,
        zindex: Option<f32>,
        color: Hsla<f32>,
    },
    Collection {
        shapes: Vec<Rc<RefCell<Shape>>>,
        transform: Transform,
        zindex: Option<f32>,
        color: Hsla<f32>,
    },
}

impl Shape {
    pub fn translate(&mut self, tx: f32, ty: f32) {
        match self {
            Self::Basic(BasicShape::Square { transform, .. })
            | Self::Basic(BasicShape::Circle { transform, .. })
            | Self::Basic(BasicShape::Triangle { transform, .. })
            | Self::Path { transform, .. }
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform.post_translate(tx, ty);
            }
            Self::Basic(BasicShape::Fill { .. }) | Self::Basic(BasicShape::Empty) => (),
        }
    }

    pub fn rotate(&mut self, r: f32) {
        match self {
            Self::Basic(BasicShape::Square { transform, .. })
            | Self::Basic(BasicShape::Circle { transform, .. })
            | Self::Basic(BasicShape::Triangle { transform, .. })
            | Self::Path { transform, .. }
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform.post_rotate(r);
            }
            Self::Basic(BasicShape::Fill { .. }) | Self::Basic(BasicShape::Empty) => (),
        }
    }

    pub fn rotate_at(&mut self, r: f32, tx: f32, ty: f32) {
        match self {
            Self::Basic(BasicShape::Square { transform, .. })
            | Self::Basic(BasicShape::Circle { transform, .. })
            | Self::Basic(BasicShape::Triangle { transform, .. })
            | Self::Path { transform, .. }
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform.post_rotate_at(r, tx, ty);
            }
            Self::Basic(BasicShape::Fill { .. }) | Self::Basic(BasicShape::Empty) => (),
        }
    }

    pub fn scale(&mut self, sx: f32, sy: f32) {
        match self {
            Self::Basic(BasicShape::Square { transform, .. })
            | Self::Basic(BasicShape::Circle { transform, .. })
            | Self::Basic(BasicShape::Triangle { transform, .. })
            | Self::Path { transform, .. }
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform.post_scale(sx, sy);
            }
            Self::Basic(BasicShape::Fill { .. }) | Self::Basic(BasicShape::Empty) => (),
        }
    }

    pub fn skew(&mut self, kx: f32, ky: f32) {
        match self {
            Self::Basic(BasicShape::Square { transform, .. })
            | Self::Basic(BasicShape::Circle { transform, .. })
            | Self::Basic(BasicShape::Triangle { transform, .. })
            | Self::Path { transform, .. }
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform.post_concat(Transform::from_skew(kx, ky));
            }
            Self::Basic(BasicShape::Fill { .. }) | Self::Basic(BasicShape::Empty) => (),
        }
    }

    pub fn flip(&mut self, f: f32) {
        match self {
            Self::Basic(BasicShape::Square { transform, .. })
            | Self::Basic(BasicShape::Circle { transform, .. })
            | Self::Basic(BasicShape::Triangle { transform, .. })
            | Self::Path { transform, .. }
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform
                    .post_rotate(f)
                    .post_scale(-1.0, 1.0)
                    .post_rotate(-f);
            }
            Self::Basic(BasicShape::Fill { .. }) | Self::Basic(BasicShape::Empty) => (),
        }
    }

    pub fn fliph(&mut self) {
        match self {
            Self::Basic(BasicShape::Square { transform, .. })
            | Self::Basic(BasicShape::Circle { transform, .. })
            | Self::Basic(BasicShape::Triangle { transform, .. })
            | Self::Path { transform, .. }
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform.post_scale(-1.0, 1.0);
            }
            Self::Basic(BasicShape::Fill { .. }) | Self::Basic(BasicShape::Empty) => (),
        }
    }

    pub fn flipv(&mut self) {
        match self {
            Self::Basic(BasicShape::Square { transform, .. })
            | Self::Basic(BasicShape::Circle { transform, .. })
            | Self::Basic(BasicShape::Triangle { transform, .. })
            | Self::Path { transform, .. }
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform.post_scale(1.0, -1.0);
            }
            Self::Basic(BasicShape::Fill { .. }) | Self::Basic(BasicShape::Empty) => (),
        }
    }

    pub fn flipd(&mut self) {
        match self {
            Self::Basic(BasicShape::Square { transform, .. })
            | Self::Basic(BasicShape::Circle { transform, .. })
            | Self::Basic(BasicShape::Triangle { transform, .. })
            | Self::Path { transform, .. }
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform.post_scale(-1.0, -1.0);
            }
            Self::Basic(BasicShape::Fill { .. }) | Self::Basic(BasicShape::Empty) => (),
        }
    }

    pub fn set_zindex(&mut self, z: f32) {
        match self {
            Self::Basic(BasicShape::Square { zindex, .. })
            | Self::Basic(BasicShape::Circle { zindex, .. })
            | Self::Basic(BasicShape::Triangle { zindex, .. })
            | Self::Basic(BasicShape::Fill { zindex, .. })
            | Self::Path { zindex, .. }
            | Self::Composite { zindex, .. }
            | Self::Collection { zindex, .. } => {
                *zindex = Some(z);
            }
            Self::Basic(BasicShape::Empty) => (),
        }
    }

    pub fn set_hsl(&mut self, hue: f32, saturation: f32, lightness: f32) {
        match self {
            Self::Basic(BasicShape::Square { color, .. })
            | Self::Basic(BasicShape::Circle { color, .. })
            | Self::Basic(BasicShape::Triangle { color, .. })
            | Self::Basic(BasicShape::Fill { color, .. })
            | Self::Path { color, .. }
            | Self::Composite { color, .. }
            | Self::Collection { color, .. } => {
                color.hue = hue.into();
                color.saturation = saturation;
                color.lightness = lightness;
            }
            Self::Basic(BasicShape::Empty) => (),
        }
    }

    pub fn set_hsla(&mut self, hue: f32, saturation: f32, lightness: f32, alpha: f32) {
        match self {
            Self::Basic(BasicShape::Square { color, .. })
            | Self::Basic(BasicShape::Circle { color, .. })
            | Self::Basic(BasicShape::Triangle { color, .. })
            | Self::Basic(BasicShape::Fill { color, .. })
            | Self::Path { color, .. }
            | Self::Composite { color, .. }
            | Self::Collection { color, .. } => {
                color.hue = hue.into();
                color.saturation = saturation;
                color.lightness = lightness;
                color.alpha = alpha;
            }
            Self::Basic(BasicShape::Empty) => (),
        }
    }

    pub fn set_hue(&mut self, hue: f32) {
        match self {
            Self::Basic(BasicShape::Square { color, .. })
            | Self::Basic(BasicShape::Circle { color, .. })
            | Self::Basic(BasicShape::Triangle { color, .. })
            | Self::Basic(BasicShape::Fill { color, .. })
            | Self::Path { color, .. }
            | Self::Composite { color, .. }
            | Self::Collection { color, .. } => {
                color.hue = hue.into();
            }
            Self::Basic(BasicShape::Empty) => (),
        }
    }

    pub fn set_saturation(&mut self, saturation: f32) {
        match self {
            Self::Basic(BasicShape::Square { color, .. })
            | Self::Basic(BasicShape::Circle { color, .. })
            | Self::Basic(BasicShape::Triangle { color, .. })
            | Self::Basic(BasicShape::Fill { color, .. })
            | Self::Path { color, .. }
            | Self::Composite { color, .. }
            | Self::Collection { color, .. } => {
                color.saturation = saturation;
            }
            Self::Basic(BasicShape::Empty) => (),
        }
    }

    pub fn set_lightness(&mut self, lightness: f32) {
        match self {
            Self::Basic(BasicShape::Square { color, .. })
            | Self::Basic(BasicShape::Circle { color, .. })
            | Self::Basic(BasicShape::Triangle { color, .. })
            | Self::Basic(BasicShape::Fill { color, .. })
            | Self::Path { color, .. }
            | Self::Composite { color, .. }
            | Self::Collection { color, .. } => {
                color.lightness = lightness;
            }
            Self::Basic(BasicShape::Empty) => (),
        }
    }

    pub fn set_alpha(&mut self, alpha: f32) {
        match self {
            Self::Basic(BasicShape::Square { color, .. })
            | Self::Basic(BasicShape::Circle { color, .. })
            | Self::Basic(BasicShape::Triangle { color, .. })
            | Self::Basic(BasicShape::Fill { color, .. })
            | Self::Path { color, .. }
            | Self::Composite { color, .. }
            | Self::Collection { color, .. } => {
                color.alpha = alpha;
            }
            Self::Basic(BasicShape::Empty) => (),
        }
    }

    pub fn shift_hue(&mut self, hue: f32) {
        match self {
            Self::Basic(BasicShape::Square { color, .. })
            | Self::Basic(BasicShape::Circle { color, .. })
            | Self::Basic(BasicShape::Triangle { color, .. })
            | Self::Basic(BasicShape::Fill { color, .. })
            | Self::Path { color, .. }
            | Self::Composite { color, .. }
            | Self::Collection { color, .. } => {
                color.hue += hue;
            }
            Self::Basic(BasicShape::Empty) => (),
        }
    }

    pub fn shift_saturation(&mut self, saturation: f32) {
        match self {
            Self::Basic(BasicShape::Square { color, .. })
            | Self::Basic(BasicShape::Circle { color, .. })
            | Self::Basic(BasicShape::Triangle { color, .. })
            | Self::Basic(BasicShape::Fill { color, .. })
            | Self::Path { color, .. }
            | Self::Composite { color, .. }
            | Self::Collection { color, .. } => {
                color.saturation += saturation;
            }
            Self::Basic(BasicShape::Empty) => (),
        }
    }

    pub fn shift_lightness(&mut self, lightness: f32) {
        match self {
            Self::Basic(BasicShape::Square { color, .. })
            | Self::Basic(BasicShape::Circle { color, .. })
            | Self::Basic(BasicShape::Triangle { color, .. })
            | Self::Basic(BasicShape::Fill { color, .. })
            | Self::Path { color, .. }
            | Self::Composite { color, .. }
            | Self::Collection { color, .. } => {
                color.lightness += lightness;
            }
            Self::Basic(BasicShape::Empty) => (),
        }
    }

    pub fn shift_alpha(&mut self, alpha: f32) {
        match self {
            Self::Basic(BasicShape::Square { color, .. })
            | Self::Basic(BasicShape::Circle { color, .. })
            | Self::Basic(BasicShape::Triangle { color, .. })
            | Self::Basic(BasicShape::Fill { color, .. })
            | Self::Path { color, .. }
            | Self::Composite { color, .. }
            | Self::Collection { color, .. } => {
                color.alpha += alpha;
            }
            Self::Basic(BasicShape::Empty) => (),
        }
    }

    pub fn set_hex(&mut self, hex: [u8; 3]) {
        match self {
            Self::Basic(BasicShape::Square { color, .. })
            | Self::Basic(BasicShape::Circle { color, .. })
            | Self::Basic(BasicShape::Triangle { color, .. })
            | Self::Basic(BasicShape::Fill { color, .. })
            | Self::Path { color, .. }
            | Self::Composite { color, .. }
            | Self::Collection { color, .. } => {
                let new_color = Rgb::from(hex);
                let new_color: Rgb<f32> = new_color.into();
                let new_color = Hsl::from_color(new_color);
                color.hue = new_color.hue;
                color.saturation = new_color.saturation;
                color.lightness = new_color.lightness;
            }
            Self::Basic(BasicShape::Empty) => (),
        }
    }
}
