#[cfg(feature = "std")]
use std::rc::Rc;

#[cfg(feature = "no-std")]
use alloc::{rc::Rc, string::String, vec, vec::Vec};

#[cfg(feature = "io")]
use {
    image::imageops::FilterType, imageproc::distance_transform::Norm,
    imageproc::geometric_transformations::Interpolation,
};

#[cfg(not(feature = "io"))]
use crate::parser::{FilterType, Norm};

use crate::parser::{SortDirection, SortMode, ThresholdType};

use core::cell::RefCell;
use palette::{rgb::Rgb, FromColor, Hsl, Hsla, RgbHue};
use tiny_skia::{
    BlendMode, FillRule, FilterQuality, LineCap, LineJoin, SpreadMode, Stroke, StrokeDash,
    Transform,
};

#[cfg(not(feature = "io"))]
type Interpolation = ();

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
    color: Color::Solid(WHITE),
    blend_mode: BlendMode::SourceOver,
    anti_alias: true,
    style: Style::Fill(FillRule::Winding),
};

pub static CIRCLE: BasicShape = BasicShape::Circle {
    x: 0.0,
    y: 0.0,
    radius: 1.0,
    transform: IDENTITY,
    zindex: None,
    color: Color::Solid(WHITE),
    blend_mode: BlendMode::SourceOver,
    anti_alias: true,
    style: Style::Fill(FillRule::Winding),
};

pub static TRIANGLE: BasicShape = BasicShape::Triangle {
    points: [-1.0, 0.577350269, 1.0, 0.577350269, 0.0, -1.154700538],
    transform: IDENTITY,
    zindex: None,
    color: Color::Solid(WHITE),
    blend_mode: BlendMode::SourceOver,
    anti_alias: true,
    style: Style::Fill(FillRule::Winding),
};

pub static FILL: BasicShape = BasicShape::Fill {
    zindex: None,
    color: Color::Solid(WHITE),
};

pub static EMPTY: BasicShape = BasicShape::Empty;

#[derive(Debug, Clone, PartialEq)]
pub struct Gradient {
    pub start: (f32, f32),
    pub end: (f32, f32),
    pub radius: Option<f32>,
    pub stops: Vec<(f32, Hsla<f32>)>,
    pub spread_mode: SpreadMode,
    pub transform: Transform,
}

impl Gradient {
    pub fn linear(start_x: f32, start_y: f32, end_x: f32, end_y: f32) -> Self {
        Self {
            start: (start_x, start_y),
            end: (end_x, end_y),
            radius: None,
            stops: Vec::new(),
            spread_mode: SpreadMode::Pad,
            transform: IDENTITY,
        }
    }

    pub fn radial(start_x: f32, start_y: f32, end_x: f32, end_y: f32, radius: f32) -> Self {
        Self {
            start: (start_x, start_y),
            end: (end_x, end_y),
            radius: Some(radius),
            stops: Vec::new(),
            spread_mode: SpreadMode::Pad,
            transform: IDENTITY,
        }
    }

    pub fn set_start(&mut self, x: f32, y: f32) {
        self.start = (x, y);
    }

    pub fn set_end(&mut self, x: f32, y: f32) {
        self.end = (x, y);
    }

    pub fn set_radius(&mut self, radius: Option<f32>) {
        self.radius = radius;
    }

    pub fn set_stop_hsl(&mut self, pos: f32, h: f32, s: f32, l: f32) {
        match self.stops.iter_mut().find(|(p, _)| *p == pos) {
            Some((_, color)) => {
                color.hue = h.into();
                color.saturation = s;
                color.lightness = l;
            }
            None => {
                self.stops.push((pos, Hsla::new(RgbHue::new(h), s, l, 1.0)));
                self.stops
                    .sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
            }
        }
    }

    pub fn set_stop_hsla(&mut self, pos: f32, h: f32, s: f32, l: f32, a: f32) {
        match self.stops.iter_mut().find(|(p, _)| *p == pos) {
            Some((_, color)) => {
                color.hue = h.into();
                color.saturation = s;
                color.lightness = l;
                color.alpha = a;
            }
            None => {
                self.stops.push((pos, Hsla::new(RgbHue::new(h), s, l, a)));
                self.stops
                    .sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
            }
        }
    }

    pub fn set_stop_hex(&mut self, pos: f32, hex: [u8; 3]) {
        let color = Rgb::from(hex);
        let color: Rgb<f32> = color.into();
        let color = Hsl::from_color(color);
        self.set_stop_hsl(pos, color.hue.into(), color.saturation, color.lightness);
    }

    pub fn set_spread_mode(&mut self, spread_mode: SpreadMode) {
        self.spread_mode = spread_mode;
    }

    pub fn translate(&mut self, tx: f32, ty: f32) {
        self.transform = self.transform.post_translate(tx, ty);
    }

    pub fn rotate(&mut self, r: f32) {
        self.transform = self.transform.post_rotate(r);
    }

    pub fn rotate_at(&mut self, r: f32, tx: f32, ty: f32) {
        self.transform = self.transform.post_rotate_at(r, tx, ty);
    }

    pub fn scale(&mut self, sx: f32, sy: f32) {
        self.transform = self.transform.post_scale(sx, sy);
    }

    pub fn skew(&mut self, kx: f32, ky: f32) {
        self.transform = self.transform.post_concat(Transform::from_skew(kx, ky));
    }

    pub fn flip(&mut self, f: f32) {
        self.transform = self
            .transform
            .post_rotate(f)
            .post_scale(-1.0, 1.0)
            .post_rotate(-f);
    }

    pub fn fliph(&mut self) {
        self.transform = self.transform.post_scale(-1.0, 1.0);
    }

    pub fn flipv(&mut self) {
        self.transform = self.transform.post_scale(1.0, -1.0);
    }

    pub fn flipd(&mut self) {
        self.transform = self.transform.post_scale(-1.0, -1.0);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Color {
    Solid(Hsla<f32>),
    Gradient(Gradient),
}

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

#[derive(Debug, Clone, PartialEq)]
pub enum ColorChange {
    Hsla(HslaChange),
    Gradient(Gradient),
}

impl Default for ColorChange {
    fn default() -> ColorChange {
        ColorChange::Hsla(HslaChange::default())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Style {
    Fill(FillRule),
    Stroke(Stroke),
}

impl Default for Style {
    fn default() -> Style {
        Style::Fill(FillRule::Winding)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pattern {
    pub pattern: Rc<RefCell<Shape>>,
    pub spread_mode: SpreadMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathSegment {
    MoveTo(f32, f32),
    LineTo(f32, f32),
    QuadTo(f32, f32, f32, f32),
    CubicTo(f32, f32, f32, f32, f32, f32),
    Close,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImagePath {
    File(String),
    Shape(Rc<RefCell<Shape>>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImageOp {
    Brighten(i32),
    Contrast(f32),
    Grayscale,
    GrayscaleAlpha,
    Huerotate(i32),
    Invert,
    Blur(f32),
    FastBlur(f32),
    Crop(u32, u32, u32, u32),
    Filter3x3([f32; 9]),
    FlipHorizontal,
    FlipVertical,
    HorizontalGradient([u8; 4], [u8; 4]),
    VerticalGradient([u8; 4], [u8; 4]),
    Overlay(Rc<RefCell<Shape>>, i64, i64),
    Replace(Rc<RefCell<Shape>>, i64, i64),
    Resize(u32, u32, FilterType),
    Rotate90,
    Rotate180,
    Rotate270,
    Thumbnail(u32, u32),
    Tile(Rc<RefCell<Shape>>),
    Unsharpen(f32, i32),
    AdaptiveThreshold(u32),
    EqualizeHistogram,
    MatchHistogram(Rc<RefCell<Shape>>),
    StretchContrast(u8, u8, u8, u8),
    Threshold(u8, ThresholdType),
    DistanceTransform(Norm),
    EuclideanSquaredDistanceTransform,
    Canny(f32, f32),
    BilateralFilter(u32, f32, f32),
    BoxFilter(u32, u32),
    GaussianBlur(f32),
    SharpenGaussian(f32, f32),
    HorizontalFilter(Vec<f32>),
    VerticalFilter(Vec<f32>),
    LaplacianFilter,
    MedianFilter(u32, u32),
    SeparableFilter(Vec<f32>, Vec<f32>),
    SeparableFilterEqual(Vec<f32>),
    Sharpen3x3,
    Rotate(f32, f32, f32, Interpolation),
    RotateAboutCenter(f32, Interpolation),
    Translate(i32, i32),
    Warp([f32; 9], Interpolation),
    HorizontalPrewitt,
    HorizontalScharr,
    HorizontalSobel,
    VerticalPrewitt,
    VerticalScharr,
    VerticalSobel,
    PrewittGradients,
    SobelGradients,
    IntegralImage,
    IntegralSquaredImage,
    RedChannel,
    GreenChannel,
    BlueChannel,
    Close(Norm, u8),
    Dilate(Norm, u8),
    Erode(Norm, u8),
    Open(Norm, u8),
    GaussianNoise(f64, f64, u64),
    SaltAndPepperNoise(f64, u64),
    SuppressNonMaximum(u32),
    PixelSort(SortMode, SortDirection),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BasicShape {
    Square {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        transform: Transform,
        zindex: Option<f32>,
        color: Color,
        blend_mode: BlendMode,
        anti_alias: bool,
        style: Style,
    },
    Circle {
        x: f32,
        y: f32,
        radius: f32,
        transform: Transform,
        zindex: Option<f32>,
        color: Color,
        blend_mode: BlendMode,
        anti_alias: bool,
        style: Style,
    },
    Triangle {
        points: [f32; 6],
        transform: Transform,
        zindex: Option<f32>,
        color: Color,
        blend_mode: BlendMode,
        anti_alias: bool,
        style: Style,
    },
    Fill {
        zindex: Option<f32>,
        color: Color,
    },
    Empty,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Basic(BasicShape, Option<Rc<RefCell<Shape>>>, Option<Pattern>),
    Path {
        segments: Vec<PathSegment>,
        transform: Transform,
        zindex: Option<f32>,
        color: Color,
        blend_mode: BlendMode,
        anti_alias: bool,
        style: Style,
        mask: Option<Rc<RefCell<Shape>>>,
        pattern: Option<Pattern>,
    },
    Image {
        path: ImagePath,
        ops: Vec<ImageOp>,
        transform: Transform,
        zindex: Option<f32>,
        opacity: f32,
        blend_mode: BlendMode,
        quality: FilterQuality,
        mask: Option<Rc<RefCell<Shape>>>,
    },
    Text {
        font: String,
        text: String,
        size: f32,
        ops: Vec<ImageOp>,
        transform: Transform,
        zindex: Option<f32>,
        opacity: f32,
        blend_mode: BlendMode,
        quality: FilterQuality,
        mask: Option<Rc<RefCell<Shape>>>,
    },
    Composite {
        a: Rc<RefCell<Shape>>,
        b: Rc<RefCell<Shape>>,
        transform: Transform,
        zindex_overwrite: Option<f32>,
        zindex_shift: Option<f32>,
        color_overwrite: ColorChange,
        color_shift: HslaChange,
        blend_mode_overwrite: Option<BlendMode>,
        anti_alias_overwrite: Option<bool>,
        style_overwrite: Option<Style>,
        mask_overwrite: Option<Rc<RefCell<Shape>>>,
        pattern_overwrite: Option<Pattern>,
    },
    Collection {
        shapes: Vec<Rc<RefCell<Shape>>>,
        transform: Transform,
        zindex_overwrite: Option<f32>,
        zindex_shift: Option<f32>,
        color_overwrite: ColorChange,
        color_shift: HslaChange,
        blend_mode_overwrite: Option<BlendMode>,
        anti_alias_overwrite: Option<bool>,
        style_overwrite: Option<Style>,
        mask_overwrite: Option<Rc<RefCell<Shape>>>,
        pattern_overwrite: Option<Pattern>,
    },
}

impl Shape {
    pub fn square() -> Self {
        Self::Basic(SQUARE.clone(), None, None)
    }

    pub fn circle() -> Self {
        Self::Basic(CIRCLE.clone(), None, None)
    }

    pub fn triangle() -> Self {
        Self::Basic(TRIANGLE.clone(), None, None)
    }

    pub fn fill() -> Self {
        Self::Basic(FILL.clone(), None, None)
    }

    pub fn empty() -> Self {
        Self::Basic(EMPTY.clone(), None, None)
    }

    pub fn path(segments: Vec<PathSegment>) -> Self {
        Self::Path {
            segments,
            transform: IDENTITY,
            zindex: None,
            color: Color::Solid(WHITE),
            blend_mode: BlendMode::SourceOver,
            anti_alias: true,
            style: Style::default(),
            mask: None,
            pattern: None,
        }
    }

    pub fn image(path: ImagePath) -> Self {
        Self::Image {
            path,
            ops: Vec::new(),
            transform: IDENTITY,
            zindex: None,
            opacity: 1.0,
            blend_mode: BlendMode::SourceOver,
            quality: FilterQuality::Nearest,
            mask: None,
        }
    }

    pub fn text(font: String, text: String, size: f32) -> Self {
        Self::Text {
            font,
            text,
            size,
            ops: Vec::new(),
            transform: IDENTITY,
            zindex: None,
            opacity: 1.0,
            blend_mode: BlendMode::SourceOver,
            quality: FilterQuality::Nearest,
            mask: None,
        }
    }

    pub fn composite(a: Rc<RefCell<Shape>>, b: Rc<RefCell<Shape>>) -> Self {
        Self::Composite {
            a: a.clone(),
            b,
            transform: IDENTITY,
            zindex_overwrite: None,
            zindex_shift: None,
            color_overwrite: ColorChange::default(),
            color_shift: HslaChange::default(),
            blend_mode_overwrite: None,
            anti_alias_overwrite: None,
            style_overwrite: None,
            mask_overwrite: None,
            pattern_overwrite: None,
        }
    }

    pub fn collection(shapes: Vec<Rc<RefCell<Shape>>>) -> Self {
        Self::Collection {
            shapes,
            transform: IDENTITY,
            zindex_overwrite: None,
            zindex_shift: None,
            color_overwrite: ColorChange::default(),
            color_shift: HslaChange::default(),
            blend_mode_overwrite: None,
            anti_alias_overwrite: None,
            style_overwrite: None,
            mask_overwrite: None,
            pattern_overwrite: None,
        }
    }

    pub fn translate(&mut self, tx: f32, ty: f32) {
        match self {
            Self::Basic(BasicShape::Square { transform, .. }, _, _)
            | Self::Basic(BasicShape::Circle { transform, .. }, _, _)
            | Self::Basic(BasicShape::Triangle { transform, .. }, _, _)
            | Self::Path { transform, .. }
            | Self::Image { transform, .. }
            | Self::Text { transform, .. }
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform.post_translate(tx, ty);
            }
            Self::Basic(BasicShape::Fill { .. }, _, _) | Self::Basic(BasicShape::Empty, _, _) => (),
        }

        match self {
            Self::Basic(_, mask, pattern) => {
                if let Some(shape) = mask {
                    shape.borrow_mut().translate(tx, ty);
                }

                if let Some(pattern) = pattern {
                    pattern.pattern.borrow_mut().translate(tx, ty);
                }
            }
            _ => (),
        }
    }

    pub fn rotate(&mut self, r: f32) {
        match self {
            Self::Basic(BasicShape::Square { transform, .. }, _, _)
            | Self::Basic(BasicShape::Circle { transform, .. }, _, _)
            | Self::Basic(BasicShape::Triangle { transform, .. }, _, _)
            | Self::Path { transform, .. }
            | Self::Image { transform, .. }
            | Self::Text { transform, .. }
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform.post_rotate(r);
            }
            Self::Basic(BasicShape::Fill { .. }, _, _) | Self::Basic(BasicShape::Empty, _, _) => (),
        }

        match self {
            Self::Basic(_, mask, pattern) => {
                if let Some(shape) = mask {
                    shape.borrow_mut().rotate(r);
                }

                if let Some(pattern) = pattern {
                    pattern.pattern.borrow_mut().rotate(r);
                }
            }
            _ => (),
        }
    }

    pub fn rotate_at(&mut self, r: f32, tx: f32, ty: f32) {
        match self {
            Self::Basic(BasicShape::Square { transform, .. }, _, _)
            | Self::Basic(BasicShape::Circle { transform, .. }, _, _)
            | Self::Basic(BasicShape::Triangle { transform, .. }, _, _)
            | Self::Path { transform, .. }
            | Self::Image { transform, .. }
            | Self::Text { transform, .. }
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform.post_rotate_at(r, tx, ty);
            }
            Self::Basic(BasicShape::Fill { .. }, _, _) | Self::Basic(BasicShape::Empty, _, _) => (),
        }

        match self {
            Self::Basic(_, mask, pattern) => {
                if let Some(shape) = mask {
                    shape.borrow_mut().rotate_at(r, tx, ty);
                }

                if let Some(pattern) = pattern {
                    pattern.pattern.borrow_mut().rotate_at(r, tx, ty);
                }
            }
            _ => (),
        }
    }

    pub fn scale(&mut self, sx: f32, sy: f32) {
        match self {
            Self::Basic(BasicShape::Square { transform, .. }, _, _)
            | Self::Basic(BasicShape::Circle { transform, .. }, _, _)
            | Self::Basic(BasicShape::Triangle { transform, .. }, _, _)
            | Self::Path { transform, .. }
            | Self::Image { transform, .. }
            | Self::Text { transform, .. }
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform.post_scale(sx, sy);
            }
            Self::Basic(BasicShape::Fill { .. }, _, _) | Self::Basic(BasicShape::Empty, _, _) => (),
        }

        match self {
            Self::Basic(_, mask, pattern) => {
                if let Some(shape) = mask {
                    shape.borrow_mut().scale(sx, sy);
                }

                if let Some(pattern) = pattern {
                    pattern.pattern.borrow_mut().scale(sx, sy);
                }
            }
            _ => (),
        }
    }

    pub fn skew(&mut self, kx: f32, ky: f32) {
        match self {
            Self::Basic(BasicShape::Square { transform, .. }, _, _)
            | Self::Basic(BasicShape::Circle { transform, .. }, _, _)
            | Self::Basic(BasicShape::Triangle { transform, .. }, _, _)
            | Self::Path { transform, .. }
            | Self::Image { transform, .. }
            | Self::Text { transform, .. }
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform.post_concat(Transform::from_skew(kx, ky));
            }
            Self::Basic(BasicShape::Fill { .. }, _, _) | Self::Basic(BasicShape::Empty, _, _) => (),
        }

        match self {
            Self::Basic(_, mask, pattern) => {
                if let Some(shape) = mask {
                    shape.borrow_mut().skew(kx, ky);
                }

                if let Some(pattern) = pattern {
                    pattern.pattern.borrow_mut().skew(kx, ky);
                }
            }
            _ => (),
        }
    }

    pub fn flip(&mut self, f: f32) {
        match self {
            Self::Basic(BasicShape::Square { transform, .. }, _, _)
            | Self::Basic(BasicShape::Circle { transform, .. }, _, _)
            | Self::Basic(BasicShape::Triangle { transform, .. }, _, _)
            | Self::Path { transform, .. }
            | Self::Image { transform, .. }
            | Self::Text { transform, .. }
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform
                    .post_rotate(f)
                    .post_scale(-1.0, 1.0)
                    .post_rotate(-f);
            }
            Self::Basic(BasicShape::Fill { .. }, _, _) | Self::Basic(BasicShape::Empty, _, _) => (),
        }

        match self {
            Self::Basic(_, mask, pattern) => {
                if let Some(shape) = mask {
                    shape.borrow_mut().flip(f);
                }

                if let Some(pattern) = pattern {
                    pattern.pattern.borrow_mut().flip(f);
                }
            }
            _ => (),
        }
    }

    pub fn fliph(&mut self) {
        match self {
            Self::Basic(BasicShape::Square { transform, .. }, _, _)
            | Self::Basic(BasicShape::Circle { transform, .. }, _, _)
            | Self::Basic(BasicShape::Triangle { transform, .. }, _, _)
            | Self::Path { transform, .. }
            | Self::Image { transform, .. }
            | Self::Text { transform, .. }
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform.post_scale(-1.0, 1.0);
            }
            Self::Basic(BasicShape::Fill { .. }, _, _) | Self::Basic(BasicShape::Empty, _, _) => (),
        }

        match self {
            Self::Basic(_, mask, pattern) => {
                if let Some(shape) = mask {
                    shape.borrow_mut().fliph();
                }

                if let Some(pattern) = pattern {
                    pattern.pattern.borrow_mut().fliph();
                }
            }
            _ => (),
        }
    }

    pub fn flipv(&mut self) {
        match self {
            Self::Basic(BasicShape::Square { transform, .. }, _, _)
            | Self::Basic(BasicShape::Circle { transform, .. }, _, _)
            | Self::Basic(BasicShape::Triangle { transform, .. }, _, _)
            | Self::Path { transform, .. }
            | Self::Image { transform, .. }
            | Self::Text { transform, .. }
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform.post_scale(1.0, -1.0);
            }
            Self::Basic(BasicShape::Fill { .. }, _, _) | Self::Basic(BasicShape::Empty, _, _) => (),
        }

        match self {
            Self::Basic(_, mask, pattern) => {
                if let Some(shape) = mask {
                    shape.borrow_mut().flipv();
                }

                if let Some(pattern) = pattern {
                    pattern.pattern.borrow_mut().flipv();
                }
            }
            _ => (),
        }
    }

    pub fn flipd(&mut self) {
        match self {
            Self::Basic(BasicShape::Square { transform, .. }, _, _)
            | Self::Basic(BasicShape::Circle { transform, .. }, _, _)
            | Self::Basic(BasicShape::Triangle { transform, .. }, _, _)
            | Self::Path { transform, .. }
            | Self::Image { transform, .. }
            | Self::Text { transform, .. }
            | Self::Composite { transform, .. }
            | Self::Collection { transform, .. } => {
                *transform = transform.post_scale(-1.0, -1.0);
            }
            Self::Basic(BasicShape::Fill { .. }, _, _) | Self::Basic(BasicShape::Empty, _, _) => (),
        }

        match self {
            Self::Basic(_, mask, pattern) => {
                if let Some(shape) = mask {
                    shape.borrow_mut().flipd();
                }

                if let Some(pattern) = pattern {
                    pattern.pattern.borrow_mut().flipd();
                }
            }
            _ => (),
        }
    }

    pub fn set_zindex(&mut self, z: f32) {
        match self {
            Self::Basic(BasicShape::Square { zindex, .. }, _, _)
            | Self::Basic(BasicShape::Circle { zindex, .. }, _, _)
            | Self::Basic(BasicShape::Triangle { zindex, .. }, _, _)
            | Self::Basic(BasicShape::Fill { zindex, .. }, _, _)
            | Self::Path { zindex, .. }
            | Self::Image { zindex, .. }
            | Self::Text { zindex, .. } => {
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
            Self::Basic(BasicShape::Empty, _, _) => (),
        }
    }

    pub fn shift_zindex(&mut self, z: f32) {
        match self {
            Self::Basic(BasicShape::Square { zindex, .. }, _, _)
            | Self::Basic(BasicShape::Circle { zindex, .. }, _, _)
            | Self::Basic(BasicShape::Triangle { zindex, .. }, _, _)
            | Self::Basic(BasicShape::Fill { zindex, .. }, _, _)
            | Self::Path { zindex, .. }
            | Self::Image { zindex, .. }
            | Self::Text { zindex, .. } => {
                *zindex.get_or_insert(0.0) += z;
            }
            Self::Composite { zindex_shift, .. } | Self::Collection { zindex_shift, .. } => {
                *zindex_shift.get_or_insert(0.0) += z;
            }
            Self::Basic(BasicShape::Empty, _, _) => (),
        }
    }

    pub fn set_color(&mut self, c: Color) {
        match self {
            Self::Basic(BasicShape::Square { color, .. }, _, _)
            | Self::Basic(BasicShape::Circle { color, .. }, _, _)
            | Self::Basic(BasicShape::Triangle { color, .. }, _, _)
            | Self::Basic(BasicShape::Fill { color, .. }, _, _)
            | Self::Path { color, .. } => {
                *color = c;
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
                match c {
                    Color::Solid(c) => {
                        let mut overwrite = HslaChange::default();
                        overwrite.hue = Some(c.hue);
                        overwrite.saturation = Some(c.saturation);
                        overwrite.lightness = Some(c.lightness);
                        overwrite.alpha = Some(c.alpha);
                        *color_overwrite = ColorChange::Hsla(overwrite);
                    }
                    Color::Gradient(g) => *color_overwrite = ColorChange::Gradient(g),
                }
                *color_shift = HslaChange::default();
            }
            Self::Basic(BasicShape::Empty, _, _) | Self::Image { .. } | Self::Text { .. } => (),
        }
    }

    pub fn set_hsl(&mut self, h: f32, s: f32, l: f32) {
        match self {
            Self::Basic(BasicShape::Square { color, .. }, _, _)
            | Self::Basic(BasicShape::Circle { color, .. }, _, _)
            | Self::Basic(BasicShape::Triangle { color, .. }, _, _)
            | Self::Basic(BasicShape::Fill { color, .. }, _, _)
            | Self::Path { color, .. } => match color {
                Color::Solid(color) => {
                    color.hue = h.into();
                    color.saturation = s;
                    color.lightness = l;
                }
                Color::Gradient(_) => *color = Color::Solid(Hsla::new(RgbHue::new(h), s, l, 1.0)),
            },
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
                match color_overwrite {
                    ColorChange::Hsla(overwrite) => {
                        overwrite.hue = Some(h.into());
                        overwrite.saturation = Some(s);
                        overwrite.lightness = Some(l);
                    }
                    ColorChange::Gradient(_) => {
                        let mut overwrite = HslaChange::default();
                        overwrite.hue = Some(h.into());
                        overwrite.saturation = Some(s);
                        overwrite.lightness = Some(l);
                        *color_overwrite = ColorChange::Hsla(overwrite);
                    }
                }
                *color_shift = HslaChange::default();
            }
            Self::Basic(BasicShape::Empty, _, _) | Self::Image { .. } | Self::Text { .. } => (),
        }
    }

    pub fn set_hsla(&mut self, h: f32, s: f32, l: f32, a: f32) {
        match self {
            Self::Basic(BasicShape::Square { color, .. }, _, _)
            | Self::Basic(BasicShape::Circle { color, .. }, _, _)
            | Self::Basic(BasicShape::Triangle { color, .. }, _, _)
            | Self::Basic(BasicShape::Fill { color, .. }, _, _)
            | Self::Path { color, .. } => match color {
                Color::Solid(color) => {
                    color.hue = h.into();
                    color.saturation = s;
                    color.lightness = l;
                    color.alpha = a;
                }
                Color::Gradient(_) => *color = Color::Solid(Hsla::new(RgbHue::new(h), s, l, a)),
            },
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
                match color_overwrite {
                    ColorChange::Hsla(overwrite) => {
                        overwrite.hue = Some(h.into());
                        overwrite.saturation = Some(s);
                        overwrite.lightness = Some(l);
                        overwrite.alpha = Some(a);
                    }
                    ColorChange::Gradient(_) => {
                        let mut overwrite = HslaChange::default();
                        overwrite.hue = Some(h.into());
                        overwrite.saturation = Some(s);
                        overwrite.lightness = Some(l);
                        overwrite.alpha = Some(a);
                        *color_overwrite = ColorChange::Hsla(overwrite);
                    }
                }
                *color_shift = HslaChange::default();
            }
            Self::Basic(BasicShape::Empty, _, _) | Self::Image { .. } | Self::Text { .. } => (),
        }
    }

    pub fn set_hue(&mut self, h: f32) {
        match self {
            Self::Basic(BasicShape::Square { color, .. }, _, _)
            | Self::Basic(BasicShape::Circle { color, .. }, _, _)
            | Self::Basic(BasicShape::Triangle { color, .. }, _, _)
            | Self::Basic(BasicShape::Fill { color, .. }, _, _)
            | Self::Path { color, .. } => match color {
                Color::Solid(color) => {
                    color.hue = h.into();
                }
                Color::Gradient(gradient) => {
                    for (_, color) in &mut gradient.stops {
                        color.hue = h.into();
                    }
                }
            },
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
                match color_overwrite {
                    ColorChange::Hsla(overwrite) => {
                        overwrite.hue = Some(h.into());
                    }
                    ColorChange::Gradient(gradient) => {
                        for (_, color) in &mut gradient.stops {
                            color.hue = h.into();
                        }
                    }
                }
                *color_shift = HslaChange::default();
            }
            Self::Basic(BasicShape::Empty, _, _) | Self::Image { .. } | Self::Text { .. } => (),
        }
    }

    pub fn set_saturation(&mut self, s: f32) {
        match self {
            Self::Basic(BasicShape::Square { color, .. }, _, _)
            | Self::Basic(BasicShape::Circle { color, .. }, _, _)
            | Self::Basic(BasicShape::Triangle { color, .. }, _, _)
            | Self::Basic(BasicShape::Fill { color, .. }, _, _)
            | Self::Path { color, .. } => match color {
                Color::Solid(color) => {
                    color.saturation = s;
                }
                Color::Gradient(gradient) => {
                    for (_, color) in &mut gradient.stops {
                        color.saturation = s;
                    }
                }
            },
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
                match color_overwrite {
                    ColorChange::Hsla(overwrite) => {
                        overwrite.saturation = Some(s);
                    }
                    ColorChange::Gradient(gradient) => {
                        for (_, color) in &mut gradient.stops {
                            color.saturation = s;
                        }
                    }
                }
                *color_shift = HslaChange::default();
            }
            Self::Basic(BasicShape::Empty, _, _) | Self::Image { .. } | Self::Text { .. } => (),
        }
    }

    pub fn set_lightness(&mut self, l: f32) {
        match self {
            Self::Basic(BasicShape::Square { color, .. }, _, _)
            | Self::Basic(BasicShape::Circle { color, .. }, _, _)
            | Self::Basic(BasicShape::Triangle { color, .. }, _, _)
            | Self::Basic(BasicShape::Fill { color, .. }, _, _)
            | Self::Path { color, .. } => match color {
                Color::Solid(color) => {
                    color.lightness = l;
                }
                Color::Gradient(gradient) => {
                    for (_, color) in &mut gradient.stops {
                        color.lightness = l;
                    }
                }
            },
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
                match color_overwrite {
                    ColorChange::Hsla(overwrite) => {
                        overwrite.lightness = Some(l);
                    }
                    ColorChange::Gradient(gradient) => {
                        for (_, color) in &mut gradient.stops {
                            color.lightness = l;
                        }
                    }
                }
                *color_shift = HslaChange::default();
            }
            Self::Basic(BasicShape::Empty, _, _) | Self::Image { .. } | Self::Text { .. } => (),
        }
    }

    pub fn set_alpha(&mut self, a: f32) {
        match self {
            Self::Basic(BasicShape::Square { color, .. }, _, _)
            | Self::Basic(BasicShape::Circle { color, .. }, _, _)
            | Self::Basic(BasicShape::Triangle { color, .. }, _, _)
            | Self::Basic(BasicShape::Fill { color, .. }, _, _)
            | Self::Path { color, .. } => match color {
                Color::Solid(color) => {
                    color.alpha = a;
                }
                Color::Gradient(gradient) => {
                    for (_, color) in &mut gradient.stops {
                        color.alpha = a;
                    }
                }
            },
            Self::Image { opacity, .. } | Self::Text { opacity, .. } => *opacity = a,
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
                match color_overwrite {
                    ColorChange::Hsla(overwrite) => {
                        overwrite.alpha = Some(a);
                    }
                    ColorChange::Gradient(gradient) => {
                        for (_, color) in &mut gradient.stops {
                            color.alpha = a;
                        }
                    }
                }
                *color_shift = HslaChange::default();
            }
            Self::Basic(BasicShape::Empty, _, _) => (),
        }
    }

    pub fn shift_hue(&mut self, n: f32) {
        match self {
            Self::Basic(BasicShape::Square { color, .. }, _, _)
            | Self::Basic(BasicShape::Circle { color, .. }, _, _)
            | Self::Basic(BasicShape::Triangle { color, .. }, _, _)
            | Self::Basic(BasicShape::Fill { color, .. }, _, _)
            | Self::Path { color, .. } => match color {
                Color::Solid(color) => {
                    color.hue += n;
                }
                Color::Gradient(gradient) => {
                    for (_, color) in &mut gradient.stops {
                        color.hue += n;
                    }
                }
            },
            Self::Composite { color_shift, .. } | Self::Collection { color_shift, .. } => {
                *color_shift.hue.get_or_insert(RgbHue::new(0.0)) += n;
            }
            Self::Basic(BasicShape::Empty, _, _) | Self::Image { .. } | Self::Text { .. } => (),
        }
    }

    pub fn shift_saturation(&mut self, n: f32) {
        match self {
            Self::Basic(BasicShape::Square { color, .. }, _, _)
            | Self::Basic(BasicShape::Circle { color, .. }, _, _)
            | Self::Basic(BasicShape::Triangle { color, .. }, _, _)
            | Self::Basic(BasicShape::Fill { color, .. }, _, _)
            | Self::Path { color, .. } => match color {
                Color::Solid(color) => {
                    color.saturation += n;
                }
                Color::Gradient(gradient) => {
                    for (_, color) in &mut gradient.stops {
                        color.saturation += n;
                    }
                }
            },
            Self::Composite { color_shift, .. } | Self::Collection { color_shift, .. } => {
                *color_shift.saturation.get_or_insert(0.0) += n;
            }
            Self::Basic(BasicShape::Empty, _, _) | Self::Image { .. } | Self::Text { .. } => (),
        }
    }

    pub fn shift_lightness(&mut self, n: f32) {
        match self {
            Self::Basic(BasicShape::Square { color, .. }, _, _)
            | Self::Basic(BasicShape::Circle { color, .. }, _, _)
            | Self::Basic(BasicShape::Triangle { color, .. }, _, _)
            | Self::Basic(BasicShape::Fill { color, .. }, _, _)
            | Self::Path { color, .. } => match color {
                Color::Solid(color) => {
                    color.lightness += n;
                }
                Color::Gradient(gradient) => {
                    for (_, color) in &mut gradient.stops {
                        color.lightness += n;
                    }
                }
            },
            Self::Composite { color_shift, .. } | Self::Collection { color_shift, .. } => {
                *color_shift.lightness.get_or_insert(0.0) += n;
            }
            Self::Basic(BasicShape::Empty, _, _) | Self::Image { .. } | Self::Text { .. } => (),
        }
    }

    pub fn shift_alpha(&mut self, n: f32) {
        match self {
            Self::Basic(BasicShape::Square { color, .. }, _, _)
            | Self::Basic(BasicShape::Circle { color, .. }, _, _)
            | Self::Basic(BasicShape::Triangle { color, .. }, _, _)
            | Self::Basic(BasicShape::Fill { color, .. }, _, _)
            | Self::Path { color, .. } => match color {
                Color::Solid(color) => {
                    color.alpha += n;
                }
                Color::Gradient(gradient) => {
                    for (_, color) in &mut gradient.stops {
                        color.alpha += n;
                    }
                }
            },
            Self::Image { opacity, .. } | Self::Text { opacity, .. } => *opacity += n,
            Self::Composite { color_shift, .. } | Self::Collection { color_shift, .. } => {
                *color_shift.alpha.get_or_insert(0.0) += n;
            }
            Self::Basic(BasicShape::Empty, _, _) => (),
        }
    }

    pub fn set_hex(&mut self, hex: [u8; 3]) {
        let color = Rgb::from(hex);
        let color: Rgb<f32> = color.into();
        let color = Hsl::from_color(color);
        self.set_hsl(color.hue.into(), color.saturation, color.lightness);
    }

    pub fn set_blend_mode(&mut self, b: BlendMode) {
        match self {
            Self::Basic(BasicShape::Square { blend_mode, .. }, _, _)
            | Self::Basic(BasicShape::Circle { blend_mode, .. }, _, _)
            | Self::Basic(BasicShape::Triangle { blend_mode, .. }, _, _)
            | Self::Path { blend_mode, .. }
            | Self::Image { blend_mode, .. }
            | Self::Text { blend_mode, .. } => {
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
            Self::Basic(BasicShape::Fill { .. }, _, _) | Self::Basic(BasicShape::Empty, _, _) => (),
        }
    }

    pub fn set_anti_alias(&mut self, a: bool) {
        match self {
            Self::Basic(BasicShape::Square { anti_alias, .. }, _, _)
            | Self::Basic(BasicShape::Circle { anti_alias, .. }, _, _)
            | Self::Basic(BasicShape::Triangle { anti_alias, .. }, _, _)
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
            Self::Basic(BasicShape::Fill { .. }, _, _)
            | Self::Basic(BasicShape::Empty, _, _)
            | Self::Image { .. }
            | Self::Text { .. } => (),
        }
    }

    pub fn set_fill_rule(&mut self, fill_rule: FillRule) {
        match self {
            Self::Basic(BasicShape::Square { style, .. }, _, _)
            | Self::Basic(BasicShape::Circle { style, .. }, _, _)
            | Self::Basic(BasicShape::Triangle { style, .. }, _, _)
            | Self::Path { style, .. } => {
                *style = Style::Fill(fill_rule);
            }
            Self::Composite {
                style_overwrite, ..
            }
            | Self::Collection {
                style_overwrite, ..
            } => {
                *style_overwrite = Some(Style::Fill(fill_rule));
            }
            Self::Basic(BasicShape::Fill { .. }, _, _)
            | Self::Basic(BasicShape::Empty, _, _)
            | Self::Image { .. }
            | Self::Text { .. } => (),
        }
    }

    pub fn set_stroke_width(&mut self, width: f32) {
        match self {
            Self::Basic(BasicShape::Square { style, .. }, _, _)
            | Self::Basic(BasicShape::Circle { style, .. }, _, _)
            | Self::Basic(BasicShape::Triangle { style, .. }, _, _)
            | Self::Path { style, .. } => match style {
                Style::Stroke(stroke) => stroke.width = width,
                Style::Fill(_) => {
                    *style = Style::Stroke(Stroke {
                        width,
                        ..Stroke::default()
                    });
                }
            },
            Self::Composite {
                style_overwrite, ..
            }
            | Self::Collection {
                style_overwrite, ..
            } => match style_overwrite {
                Some(Style::Stroke(stroke)) => stroke.width = width,
                Some(Style::Fill(_)) | None => {
                    *style_overwrite = Some(Style::Stroke(Stroke {
                        width,
                        ..Stroke::default()
                    }));
                }
            },
            Self::Basic(BasicShape::Fill { .. }, _, _)
            | Self::Basic(BasicShape::Empty, _, _)
            | Self::Image { .. }
            | Self::Text { .. } => (),
        }
    }

    pub fn set_miter_limit(&mut self, miter_limit: f32) {
        match self {
            Self::Basic(BasicShape::Square { style, .. }, _, _)
            | Self::Basic(BasicShape::Circle { style, .. }, _, _)
            | Self::Basic(BasicShape::Triangle { style, .. }, _, _)
            | Self::Path { style, .. } => match style {
                Style::Stroke(stroke) => stroke.miter_limit = miter_limit,
                Style::Fill(_) => {
                    *style = Style::Stroke(Stroke {
                        miter_limit,
                        ..Stroke::default()
                    });
                }
            },
            Self::Composite {
                style_overwrite, ..
            }
            | Self::Collection {
                style_overwrite, ..
            } => match style_overwrite {
                Some(Style::Stroke(stroke)) => stroke.miter_limit = miter_limit,
                Some(Style::Fill(_)) | None => {
                    *style_overwrite = Some(Style::Stroke(Stroke {
                        miter_limit,
                        ..Stroke::default()
                    }));
                }
            },
            Self::Basic(BasicShape::Fill { .. }, _, _)
            | Self::Basic(BasicShape::Empty, _, _)
            | Self::Image { .. }
            | Self::Text { .. } => (),
        }
    }

    pub fn set_line_cap(&mut self, line_cap: LineCap) {
        match self {
            Self::Basic(BasicShape::Square { style, .. }, _, _)
            | Self::Basic(BasicShape::Circle { style, .. }, _, _)
            | Self::Basic(BasicShape::Triangle { style, .. }, _, _)
            | Self::Path { style, .. } => match style {
                Style::Stroke(stroke) => stroke.line_cap = line_cap,
                Style::Fill(_) => {
                    *style = Style::Stroke(Stroke {
                        line_cap,
                        ..Stroke::default()
                    });
                }
            },
            Self::Composite {
                style_overwrite, ..
            }
            | Self::Collection {
                style_overwrite, ..
            } => match style_overwrite {
                Some(Style::Stroke(stroke)) => stroke.line_cap = line_cap,
                Some(Style::Fill(_)) | None => {
                    *style_overwrite = Some(Style::Stroke(Stroke {
                        line_cap,
                        ..Stroke::default()
                    }));
                }
            },
            Self::Basic(BasicShape::Fill { .. }, _, _)
            | Self::Basic(BasicShape::Empty, _, _)
            | Self::Image { .. }
            | Self::Text { .. } => (),
        }
    }

    pub fn set_line_join(&mut self, line_join: LineJoin) {
        match self {
            Self::Basic(BasicShape::Square { style, .. }, _, _)
            | Self::Basic(BasicShape::Circle { style, .. }, _, _)
            | Self::Basic(BasicShape::Triangle { style, .. }, _, _)
            | Self::Path { style, .. } => match style {
                Style::Stroke(stroke) => stroke.line_join = line_join,
                Style::Fill(_) => {
                    *style = Style::Stroke(Stroke {
                        line_join,
                        ..Stroke::default()
                    });
                }
            },
            Self::Composite {
                style_overwrite, ..
            }
            | Self::Collection {
                style_overwrite, ..
            } => match style_overwrite {
                Some(Style::Stroke(stroke)) => stroke.line_join = line_join,
                Some(Style::Fill(_)) | None => {
                    *style_overwrite = Some(Style::Stroke(Stroke {
                        line_join,
                        ..Stroke::default()
                    }));
                }
            },
            Self::Basic(BasicShape::Fill { .. }, _, _)
            | Self::Basic(BasicShape::Empty, _, _)
            | Self::Image { .. }
            | Self::Text { .. } => (),
        }
    }

    pub fn set_dash(&mut self, dash: Option<StrokeDash>) {
        match self {
            Self::Basic(BasicShape::Square { style, .. }, _, _)
            | Self::Basic(BasicShape::Circle { style, .. }, _, _)
            | Self::Basic(BasicShape::Triangle { style, .. }, _, _)
            | Self::Path { style, .. } => match style {
                Style::Stroke(stroke) => stroke.dash = dash,
                Style::Fill(_) => {
                    *style = Style::Stroke(Stroke {
                        dash,
                        ..Stroke::default()
                    });
                }
            },
            Self::Composite {
                style_overwrite, ..
            }
            | Self::Collection {
                style_overwrite, ..
            } => match style_overwrite {
                Some(Style::Stroke(stroke)) => stroke.dash = dash,
                Some(Style::Fill(_)) | None => {
                    *style_overwrite = Some(Style::Stroke(Stroke {
                        dash,
                        ..Stroke::default()
                    }));
                }
            },
            Self::Basic(BasicShape::Fill { .. }, _, _)
            | Self::Basic(BasicShape::Empty, _, _)
            | Self::Image { .. }
            | Self::Text { .. } => (),
        }
    }

    pub fn set_mask(&mut self, shape: Rc<RefCell<Shape>>) {
        match self {
            Self::Basic(_, mask, _)
            | Self::Path { mask, .. }
            | Self::Image { mask, .. }
            | Self::Text { mask, .. } => {
                *mask = Some(shape);
            }
            Self::Composite { mask_overwrite, .. } | Self::Collection { mask_overwrite, .. } => {
                *mask_overwrite = Some(shape);
            }
        }
    }

    pub fn set_pattern(&mut self, pattern: Rc<RefCell<Shape>>, spread_mode: SpreadMode) {
        let pat = Pattern {
            pattern,
            spread_mode,
        };
        match self {
            Self::Basic(_, _, pattern) | Self::Path { pattern, .. } => {
                *pattern = Some(pat);
            }
            Self::Composite {
                pattern_overwrite, ..
            }
            | Self::Collection {
                pattern_overwrite, ..
            } => {
                *pattern_overwrite = Some(pat);
            }
            Self::Image { .. } | Self::Text { .. } => (),
        }
    }

    pub fn set_image_quality(&mut self, q: FilterQuality) {
        match self {
            Self::Image { quality, .. } | Self::Text { quality, .. } => *quality = q,
            _ => (),
        }
    }

    pub fn add_image_op(&mut self, op: ImageOp) {
        match self {
            Self::Image { ops, .. } | Self::Text { ops, .. } => ops.push(op),
            _ => {
                *self = Self::Image {
                    path: ImagePath::Shape(Rc::new(RefCell::new(self.clone()))),
                    ops: vec![op],
                    transform: IDENTITY,
                    zindex: None,
                    opacity: 1.0,
                    blend_mode: BlendMode::SourceOver,
                    quality: FilterQuality::Nearest,
                    mask: None,
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tiny_skia::{BlendMode, FillRule, LineCap, SpreadMode, StrokeDash};

    #[test]
    fn test_basic_shapes() {
        // Test square creation
        let square = Shape::square();
        assert_eq!(square, Shape::Basic(SQUARE.clone(), None, None));

        // Test circle creation
        let circle = Shape::circle();
        assert_eq!(circle, Shape::Basic(CIRCLE.clone(), None, None));

        // Test triangle creation
        let triangle = Shape::triangle();
        assert_eq!(triangle, Shape::Basic(TRIANGLE.clone(), None, None));

        // Test fill creation
        let fill = Shape::fill();
        assert_eq!(fill, Shape::Basic(FILL.clone(), None, None));

        // Test empty creation
        let empty = Shape::empty();
        assert_eq!(empty, Shape::Basic(EMPTY.clone(), None, None));
    }

    #[test]
    fn test_path_creation() {
        let segments = vec![
            PathSegment::MoveTo(0.0, 0.0),
            PathSegment::LineTo(10.0, 0.0),
            PathSegment::Close,
        ];
        let path = Shape::path(segments.clone());
        assert_eq!(
            path,
            Shape::Path {
                segments,
                transform: IDENTITY,
                zindex: None,
                color: Color::Solid(WHITE),
                blend_mode: BlendMode::SourceOver,
                anti_alias: true,
                style: Style::default(),
                mask: None,
                pattern: None,
            }
        );
    }

    #[test]
    fn test_composite_shapes() {
        let square = Rc::new(RefCell::new(Shape::square()));
        let circle = Rc::new(RefCell::new(Shape::circle()));

        let composite = Shape::composite(square.clone(), circle.clone());
        assert_eq!(
            composite,
            Shape::Composite {
                a: square,
                b: circle,
                transform: IDENTITY,
                zindex_overwrite: None,
                zindex_shift: None,
                color_overwrite: ColorChange::default(),
                color_shift: HslaChange::default(),
                blend_mode_overwrite: None,
                anti_alias_overwrite: None,
                style_overwrite: None,
                mask_overwrite: None,
                pattern_overwrite: None,
            }
        );
    }

    #[test]
    fn test_collection_shapes() {
        let shapes = vec![
            Rc::new(RefCell::new(Shape::square())),
            Rc::new(RefCell::new(Shape::circle())),
        ];

        let collection = Shape::collection(shapes.clone());
        assert_eq!(
            collection,
            Shape::Collection {
                shapes,
                transform: IDENTITY,
                zindex_overwrite: None,
                zindex_shift: None,
                color_overwrite: ColorChange::default(),
                color_shift: HslaChange::default(),
                blend_mode_overwrite: None,
                anti_alias_overwrite: None,
                style_overwrite: None,
                mask_overwrite: None,
                pattern_overwrite: None,
            }
        );
    }

    #[test]
    fn test_transform_operations() {
        let mut shape = Shape::square();

        // Test translate
        shape.translate(10.0, 20.0);
        match shape {
            Shape::Basic(BasicShape::Square { transform, .. }, _, _) => {
                assert_eq!(transform.tx, 10.0);
                assert_eq!(transform.ty, 20.0);
            }
            _ => panic!("Unexpected shape type"),
        }

        // Test rotate
        shape.rotate(45.0);
        match shape {
            Shape::Basic(BasicShape::Square { transform, .. }, _, _) => {
                assert_eq!(transform.sx, 0.70710677);
            }
            _ => panic!("Unexpected shape type"),
        }

        // Test scale
        shape.scale(2.0, 3.0);
        match shape {
            Shape::Basic(BasicShape::Square { transform, .. }, _, _) => {
                assert_eq!(transform.sx, 2.0 * 0.70710677);
                assert_eq!(transform.sy, 3.0 * 0.70710677);
            }
            _ => panic!("Unexpected shape type"),
        }
    }

    #[test]
    fn test_color_operations() {
        let mut shape = Shape::square();

        // Test HSL color
        shape.set_hsl(120.0, 0.5, 0.5);
        match shape {
            Shape::Basic(
                BasicShape::Square {
                    color: Color::Solid(color),
                    ..
                },
                _,
                _,
            ) => {
                assert_eq!(color.hue, 120.0);
                assert_eq!(color.saturation, 0.5);
                assert_eq!(color.lightness, 0.5);
            }
            _ => panic!("Unexpected shape type"),
        }

        // Test HSLA color
        shape.set_hsla(180.0, 0.7, 0.3, 0.8);
        match shape {
            Shape::Basic(
                BasicShape::Square {
                    color: Color::Solid(color),
                    ..
                },
                _,
                _,
            ) => {
                assert_eq!(color.hue, 180.0);
                assert_eq!(color.saturation, 0.7);
                assert_eq!(color.lightness, 0.3);
                assert_eq!(color.alpha, 0.8);
            }
            _ => panic!("Unexpected shape type"),
        }

        // Test hex color
        shape.set_hex([0xFF, 0x00, 0x00]);
        match shape {
            Shape::Basic(
                BasicShape::Square {
                    color: Color::Solid(color),
                    ..
                },
                _,
                _,
            ) => {
                assert_eq!(color.hue, 0.0);
                assert_eq!(color.saturation, 1.0);
                assert_eq!(color.lightness, 0.5);
            }
            _ => panic!("Unexpected shape type"),
        }
    }

    #[test]
    fn test_style_operations() {
        let mut shape = Shape::square();

        // Test fill rule
        shape.set_fill_rule(FillRule::EvenOdd);
        match shape {
            Shape::Basic(
                BasicShape::Square {
                    style: Style::Fill(fill_rule),
                    ..
                },
                _,
                _,
            ) => {
                assert_eq!(fill_rule, FillRule::EvenOdd);
            }
            _ => panic!("Unexpected shape type"),
        }

        // Test stroke
        shape.set_stroke_width(2.0);
        match shape {
            Shape::Basic(
                BasicShape::Square {
                    style: Style::Stroke(ref stroke),
                    ..
                },
                _,
                _,
            ) => {
                assert_eq!(stroke.width, 2.0);
            }
            _ => panic!("Unexpected shape type"),
        }

        // Test line cap
        shape.set_line_cap(LineCap::Round);
        match shape {
            Shape::Basic(
                BasicShape::Square {
                    style: Style::Stroke(ref stroke),
                    ..
                },
                _,
                _,
            ) => {
                assert_eq!(stroke.line_cap, LineCap::Round);
            }
            _ => panic!("Unexpected shape type"),
        }

        // Test dash pattern
        let dash = StrokeDash::new(vec![5.0, 3.0], 1.0);
        shape.set_dash(dash.clone());
        match shape {
            Shape::Basic(
                BasicShape::Square {
                    style: Style::Stroke(stroke),
                    ..
                },
                _,
                _,
            ) => {
                assert_eq!(stroke.dash, dash);
            }
            _ => panic!("Unexpected shape type"),
        }
    }

    #[test]
    fn test_blend_mode() {
        let mut shape = Shape::square();
        shape.set_blend_mode(BlendMode::Multiply);
        match shape {
            Shape::Basic(BasicShape::Square { blend_mode, .. }, _, _) => {
                assert_eq!(blend_mode, BlendMode::Multiply);
            }
            _ => panic!("Unexpected shape type"),
        }
    }

    #[test]
    fn test_mask_and_pattern() {
        let mut shape = Shape::square();
        let mask = Rc::new(RefCell::new(Shape::circle()));
        let pattern = Rc::new(RefCell::new(Shape::triangle()));

        // Test mask
        shape.set_mask(mask.clone());
        match shape {
            Shape::Basic(_, Some(ref m), _) => {
                assert_eq!(*m.borrow(), *mask.borrow());
            }
            _ => panic!("Unexpected shape type"),
        }

        // Test pattern
        shape.set_pattern(pattern.clone(), SpreadMode::Repeat);
        match shape {
            Shape::Basic(_, _, Some(p)) => {
                assert_eq!(*p.pattern.borrow(), *pattern.borrow());
                assert_eq!(p.spread_mode, SpreadMode::Repeat);
            }
            _ => panic!("Unexpected shape type"),
        }
    }

    #[test]
    fn test_gradient() {
        let mut gradient = Gradient::linear(0.0, 0.0, 100.0, 100.0);

        // Test gradient stops
        gradient.set_stop_hsl(0.0, 0.0, 1.0, 0.5);
        gradient.set_stop_hsl(1.0, 120.0, 1.0, 0.5);
        assert_eq!(gradient.stops.len(), 2);
        assert_eq!(gradient.stops[0].0, 0.0);
        assert_eq!(gradient.stops[0].1.hue, 0.0);
        assert_eq!(gradient.stops[1].0, 1.0);
        assert_eq!(gradient.stops[1].1.hue, 120.0);

        // Test spread mode
        gradient.set_spread_mode(SpreadMode::Reflect);
        assert_eq!(gradient.spread_mode, SpreadMode::Reflect);
    }

    #[test]
    fn test_zindex_operations() {
        let mut shape = Shape::square();

        // Test set zindex
        shape.set_zindex(5.0);
        match shape {
            Shape::Basic(BasicShape::Square { zindex, .. }, _, _) => {
                assert_eq!(zindex, Some(5.0));
            }
            _ => panic!("Unexpected shape type"),
        }

        // Test shift zindex
        shape.shift_zindex(2.0);
        match shape {
            Shape::Basic(BasicShape::Square { zindex, .. }, _, _) => {
                assert_eq!(zindex, Some(7.0));
            }
            _ => panic!("Unexpected shape type"),
        }
    }

    #[test]
    fn test_composite_operations() {
        let square = Rc::new(RefCell::new(Shape::square()));
        let circle = Rc::new(RefCell::new(Shape::circle()));
        let mut composite = Shape::composite(square, circle);

        // Test color overwrite
        composite.set_color(Color::Solid(Hsla::new::<f32>(180.0.into(), 1.0, 0.5, 1.0)));
        match composite {
            Shape::Composite {
                ref color_overwrite,
                ..
            } => match color_overwrite {
                ColorChange::Hsla(overwrite) => {
                    assert_eq!(overwrite.hue.unwrap(), 180.0);
                    assert_eq!(overwrite.saturation.unwrap(), 1.0);
                    assert_eq!(overwrite.lightness.unwrap(), 0.5);
                }
                _ => panic!("Unexpected color change type"),
            },
            _ => panic!("Unexpected shape type"),
        }

        // Test blend mode overwrite
        composite.set_blend_mode(BlendMode::Multiply);
        match composite {
            Shape::Composite {
                blend_mode_overwrite,
                ..
            } => {
                assert_eq!(blend_mode_overwrite, Some(BlendMode::Multiply));
            }
            _ => panic!("Unexpected shape type"),
        }
    }
}
