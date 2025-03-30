#[cfg(feature = "std")]
use std::rc::Rc;

#[cfg(feature = "no-std")]
use alloc::{rc::Rc, vec::Vec};

use core::cell::RefCell;
use palette::{rgb::Rgb, FromColor, Hsl, Hsla, RgbHue};
use tiny_skia::{BlendMode, Transform};

pub static IDENTITY: Transform = Transform {
    sx: 1.0,
    kx: 0.0,
    ky: 0.0,
    sy: 1.0,
    tx: 0.0,
    ty: 0.0,
};

pub static WHITE: Hsla<f32> = Hsla::new_const(RgbHue::new(360.0), 1.0, 1.0, 1.0);

pub static SQUARE: BasicShape = BasicShape::Square {
    x: -1.0,
    y: -1.0,
    width: 2.0,
    height: 2.0,
    transform: IDENTITY,
    zindex: None,
    color: WHITE,
    blend_mode: BlendMode::SourceOver,
    anti_alias: true,
};

pub static CIRCLE: BasicShape = BasicShape::Circle {
    x: 0.0,
    y: 0.0,
    radius: 1.0,
    transform: IDENTITY,
    zindex: None,
    color: WHITE,
    blend_mode: BlendMode::SourceOver,
    anti_alias: true,
};

pub static TRIANGLE: BasicShape = BasicShape::Triangle {
    points: [-1.0, 0.577350269, 1.0, 0.577350269, 0.0, -1.154700538],
    transform: IDENTITY,
    zindex: None,
    color: WHITE,
    blend_mode: BlendMode::SourceOver,
    anti_alias: true,
};

pub static FILL: BasicShape = BasicShape::Fill {
    zindex: None,
    color: WHITE,
};

pub static EMPTY: BasicShape = BasicShape::Empty;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HslaChange {
    pub hue: Option<RgbHue<f32>>,
    pub saturation: Option<f32>,
    pub lightness: Option<f32>,
    pub alpha: Option<f32>,
}

impl Default for HslaChange {
    fn default() -> HslaChange {
        HslaChange {
            hue: None,
            saturation: None,
            lightness: None,
            alpha: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathSegment {
    MoveTo(f32, f32),
    LineTo(f32, f32),
    QuadTo(f32, f32, f32, f32),
    CubicTo(f32, f32, f32, f32, f32, f32),
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BasicShape {
    Square {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        transform: Transform,
        zindex: Option<f32>,
        color: Hsla<f32>,
        blend_mode: BlendMode,
        anti_alias: bool,
    },
    Circle {
        x: f32,
        y: f32,
        radius: f32,
        transform: Transform,
        zindex: Option<f32>,
        color: Hsla<f32>,
        blend_mode: BlendMode,
        anti_alias: bool,
    },
    Triangle {
        points: [f32; 6],
        transform: Transform,
        zindex: Option<f32>,
        color: Hsla<f32>,
        blend_mode: BlendMode,
        anti_alias: bool,
    },
    Fill {
        zindex: Option<f32>,
        color: Hsla<f32>,
    },
    Empty,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Basic(BasicShape),
    Path {
        segments: Vec<PathSegment>,
        transform: Transform,
        zindex: Option<f32>,
        color: Hsla<f32>,
        blend_mode: BlendMode,
        anti_alias: bool,
    },
    Composite {
        a: Rc<RefCell<Shape>>,
        b: Rc<RefCell<Shape>>,
        transform: Transform,
        zindex_overwrite: Option<f32>,
        zindex_shift: Option<f32>,
        color_overwrite: HslaChange,
        color_shift: HslaChange,
        blend_mode_overwrite: Option<BlendMode>,
        anti_alias_overwrite: Option<bool>,
    },
    Collection {
        shapes: Vec<Rc<RefCell<Shape>>>,
        transform: Transform,
        zindex_overwrite: Option<f32>,
        zindex_shift: Option<f32>,
        color_overwrite: HslaChange,
        color_shift: HslaChange,
        blend_mode_overwrite: Option<BlendMode>,
        anti_alias_overwrite: Option<bool>,
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
            | Self::Path { zindex, .. } => {
                *zindex = Some(z);
            }
            Self::Composite {
                zindex_overwrite,
                zindex_shift,
                ..
            }
            | Self::Collection {
                zindex_overwrite,
                zindex_shift,
                ..
            } => {
                *zindex_overwrite = Some(z);
                *zindex_shift = None;
            }
            Self::Basic(BasicShape::Empty) => (),
        }
    }

    pub fn shift_zindex(&mut self, z: f32) {
        match self {
            Self::Basic(BasicShape::Square { zindex, .. })
            | Self::Basic(BasicShape::Circle { zindex, .. })
            | Self::Basic(BasicShape::Triangle { zindex, .. })
            | Self::Basic(BasicShape::Fill { zindex, .. })
            | Self::Path { zindex, .. } => {
                *zindex.get_or_insert(0.0) += z;
            }
            Self::Composite { zindex_shift, .. } | Self::Collection { zindex_shift, .. } => {
                *zindex_shift.get_or_insert(0.0) += z;
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
            | Self::Path { color, .. } => {
                color.hue = hue.into();
                color.saturation = saturation;
                color.lightness = lightness;
            }
            Self::Composite {
                color_overwrite,
                color_shift,
                ..
            }
            | Self::Collection {
                color_overwrite,
                color_shift,
                ..
            } => {
                color_overwrite.hue = Some(hue.into());
                color_overwrite.saturation = Some(saturation);
                color_overwrite.lightness = Some(lightness);
                *color_shift = HslaChange::default();
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
            | Self::Path { color, .. } => {
                color.hue = hue.into();
                color.saturation = saturation;
                color.lightness = lightness;
                color.alpha = alpha;
            }
            Self::Composite {
                color_overwrite,
                color_shift,
                ..
            }
            | Self::Collection {
                color_overwrite,
                color_shift,
                ..
            } => {
                color_overwrite.hue = Some(hue.into());
                color_overwrite.saturation = Some(saturation);
                color_overwrite.lightness = Some(lightness);
                color_overwrite.alpha = Some(alpha);
                *color_shift = HslaChange::default();
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
            | Self::Path { color, .. } => {
                color.hue = hue.into();
            }
            Self::Composite {
                color_overwrite,
                color_shift,
                ..
            }
            | Self::Collection {
                color_overwrite,
                color_shift,
                ..
            } => {
                color_overwrite.hue = Some(hue.into());
                *color_shift = HslaChange::default();
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
            | Self::Path { color, .. } => {
                color.saturation = saturation;
            }
            Self::Composite {
                color_overwrite,
                color_shift,
                ..
            }
            | Self::Collection {
                color_overwrite,
                color_shift,
                ..
            } => {
                color_overwrite.saturation = Some(saturation);
                *color_shift = HslaChange::default();
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
            | Self::Path { color, .. } => {
                color.lightness = lightness;
            }
            Self::Composite {
                color_overwrite,
                color_shift,
                ..
            }
            | Self::Collection {
                color_overwrite,
                color_shift,
                ..
            } => {
                color_overwrite.lightness = Some(lightness);
                *color_shift = HslaChange::default();
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
            | Self::Path { color, .. } => {
                color.alpha = alpha;
            }
            Self::Composite {
                color_overwrite,
                color_shift,
                ..
            }
            | Self::Collection {
                color_overwrite,
                color_shift,
                ..
            } => {
                color_overwrite.alpha = Some(alpha);
                *color_shift = HslaChange::default();
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
            | Self::Path { color, .. } => {
                color.hue += hue;
            }
            Self::Composite { color_shift, .. } | Self::Collection { color_shift, .. } => {
                *color_shift.hue.get_or_insert(RgbHue::new(0.0)) += hue;
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
            | Self::Path { color, .. } => {
                color.saturation += saturation;
            }
            Self::Composite { color_shift, .. } | Self::Collection { color_shift, .. } => {
                *color_shift.saturation.get_or_insert(0.0) += saturation;
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
            | Self::Path { color, .. } => {
                color.lightness += lightness;
            }
            Self::Composite { color_shift, .. } | Self::Collection { color_shift, .. } => {
                *color_shift.lightness.get_or_insert(0.0) += lightness;
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
            | Self::Path { color, .. } => {
                color.alpha += alpha;
            }
            Self::Composite { color_shift, .. } | Self::Collection { color_shift, .. } => {
                *color_shift.alpha.get_or_insert(0.0) += alpha;
            }
            Self::Basic(BasicShape::Empty) => (),
        }
    }

    pub fn set_hex(&mut self, hex: [u8; 3]) {
        let new_color = Rgb::from(hex);
        let new_color: Rgb<f32> = new_color.into();
        let new_color = Hsl::from_color(new_color);
        match self {
            Self::Basic(BasicShape::Square { color, .. })
            | Self::Basic(BasicShape::Circle { color, .. })
            | Self::Basic(BasicShape::Triangle { color, .. })
            | Self::Basic(BasicShape::Fill { color, .. })
            | Self::Path { color, .. } => {
                color.hue = new_color.hue;
                color.saturation = new_color.saturation;
                color.lightness = new_color.lightness;
            }
            Self::Composite {
                color_overwrite,
                color_shift,
                ..
            }
            | Self::Collection {
                color_overwrite,
                color_shift,
                ..
            } => {
                color_overwrite.hue = Some(new_color.hue);
                color_overwrite.saturation = Some(new_color.saturation);
                color_overwrite.lightness = Some(new_color.lightness);
                *color_shift = HslaChange::default();
            }
            Self::Basic(BasicShape::Empty) => (),
        }
    }

    pub fn set_blend_mode(&mut self, b: BlendMode) {
        match self {
            Self::Basic(BasicShape::Square { blend_mode, .. })
            | Self::Basic(BasicShape::Circle { blend_mode, .. })
            | Self::Basic(BasicShape::Triangle { blend_mode, .. })
            | Self::Path { blend_mode, .. } => {
                *blend_mode = b;
            }
            Self::Composite {
                blend_mode_overwrite,
                ..
            }
            | Self::Collection {
                blend_mode_overwrite,
                ..
            } => {
                *blend_mode_overwrite = Some(b);
            }
            Self::Basic(BasicShape::Fill { .. }) | Self::Basic(BasicShape::Empty) => (),
        }
    }

    pub fn set_anti_alias(&mut self, a: bool) {
        match self {
            Self::Basic(BasicShape::Square { anti_alias, .. })
            | Self::Basic(BasicShape::Circle { anti_alias, .. })
            | Self::Basic(BasicShape::Triangle { anti_alias, .. })
            | Self::Path { anti_alias, .. } => {
                *anti_alias = a;
            }
            Self::Composite {
                anti_alias_overwrite,
                ..
            }
            | Self::Collection {
                anti_alias_overwrite,
                ..
            } => {
                *anti_alias_overwrite = Some(a);
            }
            Self::Basic(BasicShape::Fill { .. }) | Self::Basic(BasicShape::Empty) => (),
        }
    }
}
