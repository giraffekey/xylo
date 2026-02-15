#[cfg(feature = "std")]
use std::rc::Rc;

#[cfg(feature = "alloc")]
use alloc::{rc::Rc, string::String, vec::Vec};

#[cfg(all(feature = "std", feature = "io"))]
use std::{fs, io::Cursor};

#[cfg(feature = "io")]
use {
    asdf_pixel_sort::{sort_with_options, Options},
    fontdue::{Font, FontSettings},
    image::{imageops, DynamicImage, ImageBuffer, ImageFormat, ImageReader, Pixel},
    imageproc::contrast::{
        adaptive_threshold, equalize_histogram, match_histogram, stretch_contrast, threshold,
    },
    imageproc::distance_transform::{distance_transform, euclidean_squared_distance_transform},
    imageproc::edges::canny,
    imageproc::filter::{
        bilateral_filter, box_filter, gaussian_blur_f32, horizontal_filter, laplacian_filter,
        median_filter, separable_filter, separable_filter_equal, sharpen3x3, sharpen_gaussian,
        vertical_filter,
    },
    imageproc::geometric_transformations::{
        rotate, rotate_about_center, translate, warp, Projection,
    },
    imageproc::gradients::{
        horizontal_prewitt, horizontal_scharr, horizontal_sobel, prewitt_gradients,
        sobel_gradients, vertical_prewitt, vertical_scharr, vertical_sobel,
    },
    imageproc::integral_image::{integral_image, integral_squared_image},
    imageproc::morphology::{close, dilate, erode, open},
    imageproc::noise::{gaussian_noise, salt_and_pepper_noise},
    imageproc::suppress::suppress_non_maximum,
    tiny_skia::IntSize,
};

use crate::error::Result;
use crate::shape::{
    BasicShape, Color, ColorChange, Gradient, HslaChange, ImageOp, ImagePath, PathSegment, Pattern,
    Shape, Style, IDENTITY, WHITE,
};

use core::{cell::RefCell, ops::Add};
use palette::{rgb::Rgba, FromColor};
use tiny_skia::{
    BlendMode, FillRule, FilterQuality, GradientStop, LinearGradient, Mask, MaskType, Paint, Path,
    PathBuilder, Pixmap, PixmapPaint, RadialGradient, Rect, Shader, SpreadMode, Stroke, Transform,
};

#[derive(Debug, Clone)]
enum ShapeData<'a> {
    FillPath {
        path: Path,
        transform: Transform,
        fill_rule: FillRule,
        paint: Paint<'a>,
        zindex: f32,
        mask: Option<Vec<ShapeData<'a>>>,
        pattern: Option<(Vec<ShapeData<'a>>, SpreadMode)>,
    },
    StrokePath {
        path: Path,
        transform: Transform,
        stroke: Stroke,
        paint: Paint<'a>,
        zindex: f32,
        mask: Option<Vec<ShapeData<'a>>>,
        pattern: Option<(Vec<ShapeData<'a>>, SpreadMode)>,
    },
    Image {
        path: ImagePath,
        ops: Vec<ImageOp>,
        transform: Transform,
        paint: PixmapPaint,
        zindex: f32,
        mask: Option<Vec<ShapeData<'a>>>,
    },
    Text {
        font: String,
        text: String,
        size: f32,
        ops: Vec<ImageOp>,
        transform: Transform,
        paint: PixmapPaint,
        zindex: f32,
        mask: Option<Vec<ShapeData<'a>>>,
    },
    Fill {
        color: tiny_skia::Color,
        zindex: f32,
    },
    FillPaint {
        paint: Paint<'a>,
        zindex: f32,
    },
}

impl ShapeData<'_> {
    pub fn zindex(&self) -> f32 {
        match self {
            ShapeData::FillPath { zindex, .. }
            | ShapeData::StrokePath { zindex, .. }
            | ShapeData::Image { zindex, .. }
            | ShapeData::Text { zindex, .. }
            | ShapeData::Fill { zindex, .. }
            | ShapeData::FillPaint { zindex, .. } => *zindex,
        }
    }
}

fn convert_color(color: Rgba<f32>) -> tiny_skia::Color {
    tiny_skia::Color::from_rgba(
        color.red.clamp(0.0, 1.0),
        color.green.clamp(0.0, 1.0),
        color.blue.clamp(0.0, 1.0),
        color.alpha.clamp(0.0, 1.0),
    )
    .unwrap()
}

fn solid_paint<'a>(color: Rgba<f32>, blend_mode: BlendMode, anti_alias: bool) -> Paint<'a> {
    Paint {
        shader: Shader::SolidColor(convert_color(color)),
        blend_mode,
        anti_alias,
        force_hq_pipeline: false,
    }
}

fn gradient_paint<'a>(gradient: Gradient, blend_mode: BlendMode, anti_alias: bool) -> Paint<'a> {
    if gradient.stops.is_empty()
        || gradient.radius.is_none() && gradient.start == gradient.end
        || gradient.radius.is_some() && gradient.radius.unwrap() <= 0.0
    {
        return solid_paint(Rgba::from_color(WHITE), blend_mode, anti_alias);
    }

    let stops = gradient
        .stops
        .iter()
        .map(|(pos, color)| GradientStop::new(*pos, convert_color(Rgba::from_color(*color))))
        .collect();

    let shader = match gradient.radius {
        Some(radius) => RadialGradient::new(
            gradient.start.into(),
            gradient.end.into(),
            radius,
            stops,
            gradient.spread_mode,
            gradient.transform,
        )
        .unwrap(),
        None => LinearGradient::new(
            gradient.start.into(),
            gradient.end.into(),
            stops,
            gradient.spread_mode,
            gradient.transform,
        )
        .unwrap(),
    };

    Paint {
        shader,
        blend_mode,
        anti_alias,
        force_hq_pipeline: false,
    }
}

fn overwrite_zindex(
    zindex: Option<f32>,
    zindex_overwrite: Option<f32>,
    zindex_shift: Option<f32>,
) -> f32 {
    zindex_overwrite.or(zindex).unwrap_or(0.0) + zindex_shift.unwrap_or(0.0)
}

fn overwrite_color(color: Color, color_overwrite: ColorChange, color_shift: HslaChange) -> Color {
    match color_overwrite {
        ColorChange::Hsla(overwrite) => match color {
            Color::Solid(mut color) => {
                color.hue =
                    overwrite.hue.unwrap_or(color.hue) + color_shift.hue.unwrap_or(0.0.into());
                color.saturation = overwrite.saturation.unwrap_or(color.saturation)
                    + color_shift.saturation.unwrap_or(0.0.into());
                color.lightness = overwrite.lightness.unwrap_or(color.lightness)
                    + color_shift.lightness.unwrap_or(0.0.into());
                color.alpha = overwrite.alpha.unwrap_or(color.alpha)
                    + color_shift.alpha.unwrap_or(0.0.into());
                Color::Solid(color)
            }
            Color::Gradient(mut gradient) => {
                for (_, color) in &mut gradient.stops {
                    color.hue =
                        overwrite.hue.unwrap_or(color.hue) + color_shift.hue.unwrap_or(0.0.into());
                    color.saturation = overwrite.saturation.unwrap_or(color.saturation)
                        + color_shift.saturation.unwrap_or(0.0.into());
                    color.lightness = overwrite.lightness.unwrap_or(color.lightness)
                        + color_shift.lightness.unwrap_or(0.0.into());
                    color.alpha = overwrite.alpha.unwrap_or(color.alpha)
                        + color_shift.alpha.unwrap_or(0.0.into());
                }
                Color::Gradient(gradient)
            }
        },
        ColorChange::Gradient(gradient) => Color::Gradient(gradient),
    }
}

fn overwrite_blend_mode(
    blend_mode: BlendMode,
    blend_mode_overwrite: Option<BlendMode>,
) -> BlendMode {
    blend_mode_overwrite.unwrap_or(blend_mode)
}

fn overwrite_anti_alias(anti_alias: bool, anti_alias_overwrite: Option<bool>) -> bool {
    anti_alias_overwrite.unwrap_or(anti_alias)
}

fn overwrite_style(style: Style, style_overwrite: Option<Style>) -> Style {
    style_overwrite.unwrap_or(style)
}

fn overwrite_mask(
    mask: Option<Rc<RefCell<Shape>>>,
    mask_overwrite: Option<Rc<RefCell<Shape>>>,
) -> Option<Rc<RefCell<Shape>>> {
    mask_overwrite.or(mask)
}

fn overwrite_pattern(
    pattern: Option<Pattern>,
    pattern_overwrite: Option<Pattern>,
) -> Option<Pattern> {
    pattern_overwrite.or(pattern)
}

fn combine_shift<T: Add<Output = T> + Copy>(shift: Option<T>, curr: Option<T>) -> Option<T> {
    shift.map(|s| curr.map_or(s, |c| c + s)).or(curr)
}

fn resolve_zindex_overwrites(
    zindex_overwrite: Option<f32>,
    zindex_shift: Option<f32>,
    curr_zindex_overwrite: Option<f32>,
    curr_zindex_shift: Option<f32>,
) -> (Option<f32>, Option<f32>) {
    let zindex_overwrite = zindex_overwrite.or(curr_zindex_overwrite);
    let zindex_shift = combine_shift(zindex_shift, curr_zindex_shift);
    (zindex_overwrite, zindex_shift)
}

fn resolve_color_overwrites(
    color_overwrite: ColorChange,
    color_shift: HslaChange,
    curr_color_overwrite: ColorChange,
    curr_color_shift: HslaChange,
) -> (ColorChange, HslaChange) {
    let color_overwrite = match color_overwrite {
        ColorChange::Hsla(overwrite) => match curr_color_overwrite {
            ColorChange::Hsla(curr_overwrite) => ColorChange::Hsla(HslaChange {
                hue: overwrite.hue.or(curr_overwrite.hue),
                saturation: overwrite.saturation.or(curr_overwrite.saturation),
                lightness: overwrite.lightness.or(curr_overwrite.lightness),
                alpha: overwrite.alpha.or(curr_overwrite.alpha),
            }),
            ColorChange::Gradient(mut gradient) => {
                if overwrite.hue.is_some()
                    && overwrite.saturation.is_some()
                    && overwrite.lightness.is_some()
                {
                    ColorChange::Hsla(overwrite)
                } else {
                    for (_, color) in &mut gradient.stops {
                        color.hue = overwrite.hue.unwrap_or(color.hue);
                        color.saturation = overwrite.saturation.unwrap_or(color.saturation);
                        color.lightness = overwrite.lightness.unwrap_or(color.lightness);
                        color.alpha = overwrite.alpha.unwrap_or(color.alpha);
                    }
                    ColorChange::Gradient(gradient)
                }
            }
        },
        ColorChange::Gradient(gradient) => ColorChange::Gradient(gradient),
    };
    let color_shift = HslaChange {
        hue: combine_shift(color_shift.hue, curr_color_shift.hue),
        saturation: combine_shift(color_shift.saturation, curr_color_shift.saturation),
        lightness: combine_shift(color_shift.lightness, curr_color_shift.lightness),
        alpha: combine_shift(color_shift.alpha, curr_color_shift.alpha),
    };
    (color_overwrite, color_shift)
}

fn resolve_blend_mode_overwrite(
    blend_mode_overwrite: Option<BlendMode>,
    curr_blend_mode_overwrite: Option<BlendMode>,
) -> Option<BlendMode> {
    blend_mode_overwrite.or(curr_blend_mode_overwrite)
}

fn resolve_anti_alias_overwrite(
    anti_alias_overwrite: Option<bool>,
    curr_anti_alias_overwrite: Option<bool>,
) -> Option<bool> {
    anti_alias_overwrite.or(curr_anti_alias_overwrite)
}

fn resolve_style_overwrite(
    style_overwrite: Option<Style>,
    curr_style_overwrite: Option<Style>,
) -> Option<Style> {
    style_overwrite.or(curr_style_overwrite)
}

fn resolve_mask_overwrite(
    mask_overwrite: Option<Rc<RefCell<Shape>>>,
    curr_mask_overwrite: Option<Rc<RefCell<Shape>>>,
) -> Option<Rc<RefCell<Shape>>> {
    mask_overwrite.or(curr_mask_overwrite)
}

fn resolve_pattern_overwrite(
    pattern_overwrite: Option<Pattern>,
    curr_pattern_overwrite: Option<Pattern>,
) -> Option<Pattern> {
    pattern_overwrite.or(curr_pattern_overwrite)
}

fn convert_shape_rec(
    data: &mut Vec<ShapeData>,
    shape: Rc<RefCell<Shape>>,
    parent_transform: Transform,
    zindex_overwrite: Option<f32>,
    zindex_shift: Option<f32>,
    color_overwrite: ColorChange,
    color_shift: HslaChange,
    blend_mode_overwrite: Option<BlendMode>,
    anti_alias_overwrite: Option<bool>,
    style_overwrite: Option<Style>,
    mask_overwrite: Option<Rc<RefCell<Shape>>>,
    pattern_overwrite: Option<Pattern>,
) -> Result<()> {
    match &*shape.borrow() {
        Shape::Basic(
            BasicShape::Square {
                x,
                y,
                width: w,
                height: h,
                transform,
                zindex,
                color,
                blend_mode,
                anti_alias,
                style,
            },
            mask,
            pattern,
        ) => {
            let path = PathBuilder::from_rect(Rect::from_xywh(*x, *y, *w, *h).unwrap());
            let transform = transform.post_concat(parent_transform);
            let zindex = overwrite_zindex(*zindex, zindex_overwrite, zindex_shift);
            let color = overwrite_color(color.clone(), color_overwrite, color_shift);
            let blend_mode = overwrite_blend_mode(*blend_mode, blend_mode_overwrite);
            let anti_alias = overwrite_anti_alias(*anti_alias, anti_alias_overwrite);
            let style = overwrite_style(style.clone(), style_overwrite);
            let paint = match color {
                Color::Solid(color) => solid_paint(Rgba::from_color(color), blend_mode, anti_alias),
                Color::Gradient(gradient) => gradient_paint(gradient, blend_mode, anti_alias),
            };

            let mask = overwrite_mask(mask.clone(), mask_overwrite);
            let mask = mask.map(|shape| {
                let mut mask_data = Vec::new();
                convert_shape(&mut mask_data, shape, parent_transform).unwrap();
                mask_data
            });

            let pattern = overwrite_pattern(pattern.clone(), pattern_overwrite);
            let pattern = pattern.map(|pattern| {
                let mut pattern_data = Vec::new();
                convert_shape(&mut pattern_data, pattern.pattern, parent_transform).unwrap();
                (pattern_data, pattern.spread_mode)
            });

            data.push(match style {
                Style::Fill(fill_rule) => ShapeData::FillPath {
                    path,
                    transform,
                    fill_rule,
                    paint,
                    zindex,
                    mask,
                    pattern,
                },
                Style::Stroke(stroke) => ShapeData::StrokePath {
                    path,
                    transform,
                    stroke,
                    paint,
                    zindex,
                    mask,
                    pattern,
                },
            });
        }
        Shape::Basic(
            BasicShape::Circle {
                x,
                y,
                radius,
                transform,
                zindex,
                color,
                blend_mode,
                anti_alias,
                style,
            },
            mask,
            pattern,
        ) => {
            let path = PathBuilder::from_circle(*x, *y, *radius).unwrap();
            let transform = transform.post_concat(parent_transform);
            let zindex = overwrite_zindex(*zindex, zindex_overwrite, zindex_shift);
            let color = overwrite_color(color.clone(), color_overwrite, color_shift);
            let blend_mode = overwrite_blend_mode(*blend_mode, blend_mode_overwrite);
            let anti_alias = overwrite_anti_alias(*anti_alias, anti_alias_overwrite);
            let style = overwrite_style(style.clone(), style_overwrite);
            let paint = match color {
                Color::Solid(color) => solid_paint(Rgba::from_color(color), blend_mode, anti_alias),
                Color::Gradient(gradient) => gradient_paint(gradient, blend_mode, anti_alias),
            };

            let mask = overwrite_mask(mask.clone(), mask_overwrite);
            let mask = mask.map(|shape| {
                let mut mask_data = Vec::new();
                convert_shape(&mut mask_data, shape, parent_transform).unwrap();
                mask_data
            });

            let pattern = overwrite_pattern(pattern.clone(), pattern_overwrite);
            let pattern = pattern.map(|pattern| {
                let mut pattern_data = Vec::new();
                convert_shape(&mut pattern_data, pattern.pattern, parent_transform).unwrap();
                (pattern_data, pattern.spread_mode)
            });

            data.push(match style {
                Style::Fill(fill_rule) => ShapeData::FillPath {
                    path,
                    transform,
                    fill_rule,
                    paint,
                    zindex,
                    mask,
                    pattern,
                },
                Style::Stroke(stroke) => ShapeData::StrokePath {
                    path,
                    transform,
                    stroke,
                    paint,
                    zindex,
                    mask,
                    pattern,
                },
            });
        }
        Shape::Basic(
            BasicShape::Triangle {
                points,
                transform,
                zindex,
                color,
                blend_mode,
                anti_alias,
                style,
            },
            mask,
            pattern,
        ) => {
            let mut pb = PathBuilder::new();
            pb.move_to(points[0], points[1]);
            pb.line_to(points[2], points[3]);
            pb.line_to(points[4], points[5]);
            pb.close();
            let path = pb.finish().unwrap();

            let transform = transform.post_concat(parent_transform);
            let zindex = overwrite_zindex(*zindex, zindex_overwrite, zindex_shift);
            let color = overwrite_color(color.clone(), color_overwrite, color_shift);
            let blend_mode = overwrite_blend_mode(*blend_mode, blend_mode_overwrite);
            let anti_alias = overwrite_anti_alias(*anti_alias, anti_alias_overwrite);
            let style = overwrite_style(style.clone(), style_overwrite);
            let paint = match color {
                Color::Solid(color) => solid_paint(Rgba::from_color(color), blend_mode, anti_alias),
                Color::Gradient(gradient) => gradient_paint(gradient, blend_mode, anti_alias),
            };

            let mask = overwrite_mask(mask.clone(), mask_overwrite);
            let mask = mask.map(|shape| {
                let mut mask_data = Vec::new();
                convert_shape(&mut mask_data, shape, parent_transform).unwrap();
                mask_data
            });

            let pattern = overwrite_pattern(pattern.clone(), pattern_overwrite);
            let pattern = pattern.map(|pattern| {
                let mut pattern_data = Vec::new();
                convert_shape(&mut pattern_data, pattern.pattern, parent_transform).unwrap();
                (pattern_data, pattern.spread_mode)
            });

            data.push(match style {
                Style::Fill(fill_rule) => ShapeData::FillPath {
                    path,
                    transform,
                    fill_rule,
                    paint,
                    zindex,
                    mask,
                    pattern,
                },
                Style::Stroke(stroke) => ShapeData::StrokePath {
                    path,
                    transform,
                    stroke,
                    paint,
                    zindex,
                    mask,
                    pattern,
                },
            });
        }
        Shape::Image {
            path,
            ops,
            transform,
            zindex,
            opacity,
            blend_mode,
            quality,
            mask,
        } => {
            let transform = transform.post_concat(parent_transform);
            let zindex = overwrite_zindex(*zindex, zindex_overwrite, zindex_shift);
            let blend_mode = overwrite_blend_mode(*blend_mode, blend_mode_overwrite);
            let paint = PixmapPaint {
                opacity: *opacity,
                blend_mode,
                quality: *quality,
            };

            let mask = overwrite_mask(mask.clone(), mask_overwrite);
            let mask = mask.map(|shape| {
                let mut mask_data = Vec::new();
                convert_shape(&mut mask_data, shape, parent_transform).unwrap();
                mask_data
            });

            data.push(ShapeData::Image {
                path: path.clone(),
                ops: ops.clone(),
                transform,
                paint,
                zindex,
                mask,
            });
        }
        Shape::Text {
            font,
            text,
            size,
            ops,
            transform,
            zindex,
            opacity,
            blend_mode,
            quality,
            mask,
        } => {
            let transform = transform.post_concat(parent_transform);
            let zindex = overwrite_zindex(*zindex, zindex_overwrite, zindex_shift);
            let blend_mode = overwrite_blend_mode(*blend_mode, blend_mode_overwrite);
            let paint = PixmapPaint {
                opacity: *opacity,
                blend_mode,
                quality: *quality,
            };

            let mask = overwrite_mask(mask.clone(), mask_overwrite);
            let mask = mask.map(|shape| {
                let mut mask_data = Vec::new();
                convert_shape(&mut mask_data, shape, parent_transform).unwrap();
                mask_data
            });

            data.push(ShapeData::Text {
                font: font.clone(),
                text: text.clone(),
                size: *size,
                ops: ops.clone(),
                transform,
                paint,
                zindex,
                mask,
            });
        }
        Shape::Basic(BasicShape::Fill { zindex, color }, _, _) => {
            let zindex = zindex.unwrap_or(0.0);
            let color = overwrite_color(color.clone(), color_overwrite, color_shift);

            data.push(match color {
                Color::Solid(color) => {
                    let color = convert_color(Rgba::from_color(color));
                    ShapeData::Fill { zindex, color }
                }
                Color::Gradient(gradient) => {
                    let paint = gradient_paint(gradient, BlendMode::SourceOver, true);
                    ShapeData::FillPaint { zindex, paint }
                }
            });
        }
        Shape::Basic(BasicShape::Empty, _, _) => (),
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
        } => {
            let mut pb = PathBuilder::new();
            for segment in segments {
                match segment {
                    PathSegment::MoveTo(x, y) => pb.move_to(*x, *y),
                    PathSegment::LineTo(x, y) => pb.line_to(*x, *y),
                    PathSegment::QuadTo(x1, y1, x, y) => pb.quad_to(*x1, *y1, *x, *y),
                    PathSegment::CubicTo(x1, y1, x2, y2, x, y) => {
                        pb.cubic_to(*x1, *y1, *x2, *y2, *x, *y)
                    }
                    PathSegment::Close => pb.close(),
                }
            }
            let path = pb.finish();

            if let Some(path) = path {
                let transform = transform.post_concat(parent_transform);
                let zindex = overwrite_zindex(*zindex, zindex_overwrite, zindex_shift);
                let color = overwrite_color(color.clone(), color_overwrite, color_shift);
                let blend_mode = overwrite_blend_mode(*blend_mode, blend_mode_overwrite);
                let anti_alias = overwrite_anti_alias(*anti_alias, anti_alias_overwrite);
                let style = overwrite_style(style.clone(), style_overwrite);
                let paint = match color {
                    Color::Solid(color) => {
                        solid_paint(Rgba::from_color(color), blend_mode, anti_alias)
                    }
                    Color::Gradient(gradient) => gradient_paint(gradient, blend_mode, anti_alias),
                };

                let mask = overwrite_mask(mask.clone(), mask_overwrite);
                let mask = mask.map(|shape| {
                    let mut mask_data = Vec::new();
                    convert_shape(&mut mask_data, shape, parent_transform).unwrap();
                    mask_data
                });

                let pattern = overwrite_pattern(pattern.clone(), pattern_overwrite);
                let pattern = pattern.map(|pattern| {
                    let mut pattern_data = Vec::new();
                    convert_shape(&mut pattern_data, pattern.pattern, parent_transform).unwrap();
                    (pattern_data, pattern.spread_mode)
                });

                data.push(match style {
                    Style::Fill(fill_rule) => ShapeData::FillPath {
                        path,
                        transform,
                        fill_rule,
                        paint,
                        zindex,
                        mask,
                        pattern,
                    },
                    Style::Stroke(stroke) => ShapeData::StrokePath {
                        path,
                        transform,
                        stroke,
                        paint,
                        zindex,
                        mask,
                        pattern,
                    },
                });
            }
        }
        Shape::Composite {
            a,
            b,
            transform,
            zindex_overwrite: curr_zindex_overwrite,
            zindex_shift: curr_zindex_shift,
            color_overwrite: curr_color_overwrite,
            color_shift: curr_color_shift,
            blend_mode_overwrite: curr_blend_mode_overwrite,
            anti_alias_overwrite: curr_anti_alias_overwrite,
            style_overwrite: curr_style_overwrite,
            mask_overwrite: curr_mask_overwrite,
            pattern_overwrite: curr_pattern_overwrite,
        } => {
            let transform = transform.post_concat(parent_transform);
            let (zindex_overwrite, zindex_shift) = resolve_zindex_overwrites(
                zindex_overwrite,
                zindex_shift,
                *curr_zindex_overwrite,
                *curr_zindex_shift,
            );
            let (color_overwrite, color_shift) = resolve_color_overwrites(
                color_overwrite,
                color_shift,
                curr_color_overwrite.clone(),
                *curr_color_shift,
            );
            let blend_mode_overwrite =
                resolve_blend_mode_overwrite(blend_mode_overwrite, *curr_blend_mode_overwrite);
            let anti_alias_overwrite =
                resolve_anti_alias_overwrite(anti_alias_overwrite, *curr_anti_alias_overwrite);
            let style_overwrite =
                resolve_style_overwrite(style_overwrite, curr_style_overwrite.clone());
            let mask_overwrite =
                resolve_mask_overwrite(mask_overwrite, curr_mask_overwrite.clone());
            let pattern_overwrite =
                resolve_pattern_overwrite(pattern_overwrite, curr_pattern_overwrite.clone());

            convert_shape_rec(
                data,
                a.clone(),
                transform,
                zindex_overwrite,
                zindex_shift,
                color_overwrite.clone(),
                color_shift,
                blend_mode_overwrite,
                anti_alias_overwrite,
                style_overwrite.clone(),
                mask_overwrite.clone(),
                pattern_overwrite.clone(),
            )?;
            convert_shape_rec(
                data,
                b.clone(),
                transform,
                zindex_overwrite,
                zindex_shift,
                color_overwrite,
                color_shift,
                blend_mode_overwrite,
                anti_alias_overwrite,
                style_overwrite,
                mask_overwrite,
                pattern_overwrite,
            )?;
        }
        Shape::Collection {
            shapes,
            transform,
            zindex_overwrite: curr_zindex_overwrite,
            zindex_shift: curr_zindex_shift,
            color_overwrite: curr_color_overwrite,
            color_shift: curr_color_shift,
            blend_mode_overwrite: curr_blend_mode_overwrite,
            anti_alias_overwrite: curr_anti_alias_overwrite,
            style_overwrite: curr_style_overwrite,
            mask_overwrite: curr_mask_overwrite,
            pattern_overwrite: curr_pattern_overwrite,
        } => {
            let transform = transform.post_concat(parent_transform);
            let (zindex_overwrite, zindex_shift) = resolve_zindex_overwrites(
                zindex_overwrite,
                zindex_shift,
                *curr_zindex_overwrite,
                *curr_zindex_shift,
            );
            let (color_overwrite, color_shift) = resolve_color_overwrites(
                color_overwrite,
                color_shift,
                curr_color_overwrite.clone(),
                *curr_color_shift,
            );
            let blend_mode_overwrite =
                resolve_blend_mode_overwrite(blend_mode_overwrite, *curr_blend_mode_overwrite);
            let anti_alias_overwrite =
                resolve_anti_alias_overwrite(anti_alias_overwrite, *curr_anti_alias_overwrite);
            let style_overwrite =
                resolve_style_overwrite(style_overwrite, curr_style_overwrite.clone());
            let mask_overwrite =
                resolve_mask_overwrite(mask_overwrite, curr_mask_overwrite.clone());
            let pattern_overwrite =
                resolve_pattern_overwrite(pattern_overwrite, curr_pattern_overwrite.clone());

            for shape in shapes {
                convert_shape_rec(
                    data,
                    shape.clone(),
                    transform,
                    zindex_overwrite,
                    zindex_shift,
                    color_overwrite.clone(),
                    color_shift,
                    blend_mode_overwrite,
                    anti_alias_overwrite,
                    style_overwrite.clone(),
                    mask_overwrite.clone(),
                    pattern_overwrite.clone(),
                )?;
            }
        }
    }
    Ok(())
}

fn convert_shape(
    data: &mut Vec<ShapeData>,
    shape: Rc<RefCell<Shape>>,
    transform: Transform,
) -> Result<()> {
    convert_shape_rec(
        data,
        shape,
        transform,
        None,
        None,
        ColorChange::default(),
        HslaChange::default(),
        None,
        None,
        None,
        None,
        None,
    )
}

fn render_to_pixmap(
    shape_data: ShapeData,
    pixmap: &mut Pixmap,
    width: u32,
    height: u32,
) -> Result<()> {
    match shape_data.clone() {
        ShapeData::FillPath {
            path,
            transform,
            fill_rule,
            paint,
            mask,
            pattern,
            ..
        } => {
            let mask = mask.map(|data| {
                let mut pixmap = Pixmap::new(width, height).unwrap();
                for shape_data in data {
                    render_to_pixmap(shape_data, &mut pixmap, width, height).unwrap();
                }
                Mask::from_pixmap(pixmap.as_ref(), MaskType::Luminance)
            });

            match pattern {
                Some((data, spread_mode)) => {
                    let mut pattern_pixmap = Pixmap::new(width, height).unwrap();
                    for shape_data in data {
                        render_to_pixmap(shape_data, &mut pattern_pixmap, width, height)?;
                    }
                    let shader = tiny_skia::Pattern::new(
                        pattern_pixmap.as_ref(),
                        spread_mode,
                        FilterQuality::Nearest,
                        1.0,
                        IDENTITY,
                    );
                    let paint = Paint { shader, ..paint };

                    let transform = transform
                        .post_scale(1.0, -1.0)
                        .post_translate(width as f32 / 2.0, height as f32 / 2.0);
                    pixmap.fill_path(&path, &paint, fill_rule, transform, mask.as_ref());
                }
                None => {
                    let transform = transform
                        .post_scale(1.0, -1.0)
                        .post_translate(width as f32 / 2.0, height as f32 / 2.0);
                    pixmap.fill_path(&path, &paint, fill_rule, transform, mask.as_ref());
                }
            }
        }
        ShapeData::StrokePath {
            path,
            transform,
            stroke,
            paint,
            mask,
            pattern,
            ..
        } => {
            let mask = mask.map(|data| {
                let mut pixmap = Pixmap::new(width, height).unwrap();
                for shape_data in data {
                    render_to_pixmap(shape_data, &mut pixmap, width, height).unwrap();
                }
                Mask::from_pixmap(pixmap.as_ref(), MaskType::Luminance)
            });

            match pattern {
                Some((data, spread_mode)) => {
                    let mut pattern_pixmap = Pixmap::new(width, height).unwrap();
                    for shape_data in data {
                        render_to_pixmap(shape_data, &mut pattern_pixmap, width, height).unwrap();
                    }
                    let shader = tiny_skia::Pattern::new(
                        pattern_pixmap.as_ref(),
                        spread_mode,
                        FilterQuality::Nearest,
                        1.0,
                        IDENTITY,
                    );
                    let paint = Paint { shader, ..paint };

                    let transform = transform
                        .post_scale(1.0, -1.0)
                        .post_translate(width as f32 / 2.0, height as f32 / 2.0);
                    pixmap.stroke_path(&path, &paint, &stroke, transform, mask.as_ref());
                }
                None => {
                    let transform = transform
                        .post_scale(1.0, -1.0)
                        .post_translate(width as f32 / 2.0, height as f32 / 2.0);
                    pixmap.stroke_path(&path, &paint, &stroke, transform, mask.as_ref());
                }
            }
        }
        #[cfg(all(feature = "std", feature = "io"))]
        ShapeData::Image {
            ops,
            transform,
            paint,
            mask,
            ..
        }
        | ShapeData::Text {
            ops,
            transform,
            paint,
            mask,
            ..
        } => {
            let mut image = match shape_data {
                ShapeData::Image { path, .. } => match path {
                    ImagePath::File(path) => {
                        ImageReader::open(path).unwrap().decode().unwrap().flipv()
                    }
                    ImagePath::Shape(shape) => {
                        let pixmap = render(shape.clone(), width, height).unwrap();
                        ImageReader::with_format(
                            Cursor::new(pixmap.encode_png().unwrap()),
                            ImageFormat::Png,
                        )
                        .decode()
                        .unwrap()
                    }
                },
                ShapeData::Text {
                    font, text, size, ..
                } => render_font(font, text, size),
                _ => unreachable!(),
            };

            for op in ops {
                apply_image_op(&mut image, op, width, height);
            }

            let image_width = image.width();
            let image_height = image.height();
            let image = Pixmap::from_vec(
                image.into_bytes(),
                IntSize::from_wh(image_width, image_height).unwrap(),
            )
            .unwrap();

            let mask = mask.map(|data| {
                let mut pixmap = Pixmap::new(width, height).unwrap();
                for shape_data in data {
                    render_to_pixmap(shape_data, &mut pixmap, width, height).unwrap();
                }
                Mask::from_pixmap(pixmap.as_ref(), MaskType::Luminance)
            });

            let transform = transform
                .post_scale(1.0, -1.0)
                .post_translate(width as f32 / 2.0, height as f32 / 2.0);
            pixmap.draw_pixmap(
                -(image_width as i32 / 2),
                -(image_height as i32 / 2),
                image.as_ref(),
                &paint,
                transform,
                mask.as_ref(),
            );
        }
        #[cfg(not(all(feature = "std", feature = "io")))]
        ShapeData::Image { .. } | ShapeData::Text { .. } => return Err(crate::Error::NoIO),
        ShapeData::Fill { color, .. } => {
            pixmap.fill(color);
        }
        ShapeData::FillPaint { paint, .. } => {
            let path = PathBuilder::from_rect(
                Rect::from_xywh(
                    -(width as f32) / 2.0,
                    -(height as f32) / 2.0,
                    width as f32,
                    height as f32,
                )
                .unwrap(),
            );
            let transform = IDENTITY
                .post_scale(1.0, -1.0)
                .post_translate(width as f32 / 2.0, height as f32 / 2.0);
            pixmap.fill_path(&path, &paint, FillRule::Winding, transform, None);
        }
    }

    Ok(())
}

#[cfg(all(feature = "std", feature = "io"))]
fn render_font(path: String, text: String, size: f32) -> DynamicImage {
    let font = fs::read(path).unwrap();
    let font = Font::from_bytes(font, FontSettings::default()).unwrap();

    let mut bitmaps = Vec::new();
    let mut width = 0;
    let mut height = 0;

    for c in text.chars() {
        let (metrics, bitmap) = font.rasterize(c, size);
        width += (metrics.width as u32).max(metrics.advance_width as u32);
        height = height.max(metrics.height as u32);
        bitmaps.push((metrics, bitmap));
    }

    let mut image = ImageBuffer::new(width, height);

    let mut x_offset = 0;
    for (metrics, bitmap) in bitmaps {
        for (i, &value) in bitmap.iter().enumerate() {
            let x =
                (x_offset as i32 + metrics.xmin + i as i32 % metrics.width as i32).max(0) as u32;
            let y = height - (metrics.ymin + i as i32 / metrics.width as i32).max(0) as u32 - 1;
            image.put_pixel(x, y, image::Rgba([value as u8; 4]));
        }

        x_offset += metrics.advance_width as u32;
    }

    image.into()
}

#[cfg(all(feature = "std", feature = "io"))]
fn apply_image_op(image: &mut DynamicImage, op: ImageOp, width: u32, height: u32) {
    match op {
        ImageOp::Brighten(value) => *image = image.brighten(value),
        ImageOp::Contrast(c) => *image = image.adjust_contrast(c),
        ImageOp::Grayscale => *image = image.grayscale().into_rgba8().into(),
        ImageOp::GrayscaleAlpha => {
            let img: DynamicImage = imageops::colorops::grayscale_alpha(image).into();
            *image = img.into_rgba8().into();
        }
        ImageOp::Huerotate(value) => *image = image.huerotate(value),
        ImageOp::Invert => image.invert(),
        ImageOp::Blur(sigma) => *image = image.blur(sigma),
        ImageOp::FastBlur(sigma) => *image = image.fast_blur(sigma),
        ImageOp::Crop(x, y, width, height) => {
            *image = image.crop_imm(x, y, width, height);
        }
        ImageOp::Filter3x3(kernel) => *image = image.filter3x3(&kernel),
        ImageOp::FlipHorizontal => *image = image.fliph(),
        ImageOp::FlipVertical => *image = image.flipv(),
        ImageOp::HorizontalGradient(start, end) => imageops::horizontal_gradient(
            image,
            image::Rgba::from_slice(&start),
            image::Rgba::from_slice(&end),
        ),
        ImageOp::VerticalGradient(start, end) => imageops::vertical_gradient(
            image,
            image::Rgba::from_slice(&start),
            image::Rgba::from_slice(&end),
        ),
        ImageOp::Overlay(top, x, y) => {
            let top = render(top, width, height).unwrap();
            let top =
                ImageReader::with_format(Cursor::new(top.encode_png().unwrap()), ImageFormat::Png)
                    .decode()
                    .unwrap();
            imageops::overlay(image, &top, x, y);
        }
        ImageOp::Replace(top, x, y) => {
            let top = render(top, width, height).unwrap();
            let top =
                ImageReader::with_format(Cursor::new(top.encode_png().unwrap()), ImageFormat::Png)
                    .decode()
                    .unwrap();
            imageops::replace(image, &top, x, y);
        }
        ImageOp::Resize(width, height, filter) => {
            *image = image.resize(width, height, filter);
        }
        ImageOp::Rotate90 => *image = image.rotate90(),
        ImageOp::Rotate180 => *image = image.rotate180(),
        ImageOp::Rotate270 => *image = image.rotate270(),
        ImageOp::Thumbnail(width, height) => *image = image.thumbnail(width, height),
        ImageOp::Tile(top) => {
            let top = render(top, width, height).unwrap();
            let top =
                ImageReader::with_format(Cursor::new(top.encode_png().unwrap()), ImageFormat::Png)
                    .decode()
                    .unwrap();
            imageops::tile(image, &top);
        }
        ImageOp::Unsharpen(sigma, threshold) => *image = image.unsharpen(sigma, threshold),
        ImageOp::AdaptiveThreshold(block_radius) => {
            let luma8 = image.clone().into_luma8();
            let buf = adaptive_threshold(&luma8, block_radius);
            image
                .as_mut_rgba8()
                .unwrap()
                .enumerate_pixels_mut()
                .for_each(|(x, y, pixel)| {
                    let adjust = buf.get_pixel(x, y)[0] as i16 - luma8.get_pixel(x, y)[0] as i16;
                    pixel[0] = (pixel[0] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[1] = (pixel[1] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[2] = (pixel[2] as i16 + adjust).clamp(0, 255) as u8;
                });
        }
        ImageOp::EqualizeHistogram => {
            let luma8 = image.clone().into_luma8();
            let buf = equalize_histogram(&luma8);
            image
                .as_mut_rgba8()
                .unwrap()
                .enumerate_pixels_mut()
                .for_each(|(x, y, pixel)| {
                    let adjust = buf.get_pixel(x, y)[0] as i16 - luma8.get_pixel(x, y)[0] as i16;
                    pixel[0] = (pixel[0] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[1] = (pixel[1] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[2] = (pixel[2] as i16 + adjust).clamp(0, 255) as u8;
                });
        }
        ImageOp::MatchHistogram(target) => {
            let target = render(target, width, height).unwrap();
            let target = ImageReader::with_format(
                Cursor::new(target.encode_png().unwrap()),
                ImageFormat::Png,
            )
            .decode()
            .unwrap();

            let luma8 = image.clone().into_luma8();
            let buf = match_histogram(&luma8, &target.into_luma8());
            image
                .as_mut_rgba8()
                .unwrap()
                .enumerate_pixels_mut()
                .for_each(|(x, y, pixel)| {
                    let adjust = buf.get_pixel(x, y)[0] as i16 - luma8.get_pixel(x, y)[0] as i16;
                    pixel[0] = (pixel[0] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[1] = (pixel[1] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[2] = (pixel[2] as i16 + adjust).clamp(0, 255) as u8;
                });
        }
        ImageOp::StretchContrast(input_lower, input_upper, output_lower, output_upper) => {
            let luma8 = image.clone().into_luma8();
            let buf =
                stretch_contrast(&luma8, input_lower, input_upper, output_lower, output_upper);
            image
                .as_mut_rgba8()
                .unwrap()
                .enumerate_pixels_mut()
                .for_each(|(x, y, pixel)| {
                    let adjust = buf.get_pixel(x, y)[0] as i16 - luma8.get_pixel(x, y)[0] as i16;
                    pixel[0] = (pixel[0] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[1] = (pixel[1] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[2] = (pixel[2] as i16 + adjust).clamp(0, 255) as u8;
                });
        }
        ImageOp::Threshold(t, t_type) => {
            let luma8 = image.clone().into_luma8();
            let buf = threshold(&luma8, t, t_type.into());
            image
                .as_mut_rgba8()
                .unwrap()
                .enumerate_pixels_mut()
                .for_each(|(x, y, pixel)| {
                    let adjust = buf.get_pixel(x, y)[0] as i16;
                    pixel[0] = (pixel[0] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[1] = (pixel[1] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[2] = (pixel[2] as i16 + adjust).clamp(0, 255) as u8;
                });
        }
        ImageOp::DistanceTransform(norm) => {
            let luma8 = image.clone().into_luma8();
            let buf = distance_transform(&luma8, norm);
            image
                .as_mut_rgba8()
                .unwrap()
                .enumerate_pixels_mut()
                .for_each(|(x, y, pixel)| {
                    let adjust = buf.get_pixel(x, y)[0] as i16;
                    pixel[0] = (pixel[0] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[1] = (pixel[1] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[2] = (pixel[2] as i16 + adjust).clamp(0, 255) as u8;
                });
        }
        ImageOp::EuclideanSquaredDistanceTransform => {
            let luma8 = image.clone().into_luma8();
            let buf = euclidean_squared_distance_transform(&luma8);
            image
                .as_mut_rgba8()
                .unwrap()
                .enumerate_pixels_mut()
                .for_each(|(x, y, pixel)| {
                    let adjust = buf.get_pixel(x, y)[0];
                    pixel[0] = (pixel[0] as f64 + adjust).clamp(0.0, 255.0) as u8;
                    pixel[1] = (pixel[1] as f64 + adjust).clamp(0.0, 255.0) as u8;
                    pixel[2] = (pixel[2] as f64 + adjust).clamp(0.0, 255.0) as u8;
                });
        }
        ImageOp::Canny(low_threshold, high_threshold) => {
            let luma8 = image.clone().into_luma8();
            let buf = canny(&luma8, low_threshold, high_threshold);
            image
                .as_mut_rgba8()
                .unwrap()
                .enumerate_pixels_mut()
                .for_each(|(x, y, pixel)| {
                    let adjust = buf.get_pixel(x, y)[0] as i16;
                    pixel[0] = (pixel[0] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[1] = (pixel[1] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[2] = (pixel[2] as i16 + adjust).clamp(0, 255) as u8;
                });
        }
        ImageOp::BilateralFilter(window_size, sigma_color, sigma_spatial) => {
            let luma8 = image.clone().into_luma8();
            let buf = bilateral_filter(&luma8, window_size, sigma_color, sigma_spatial);
            image
                .as_mut_rgba8()
                .unwrap()
                .enumerate_pixels_mut()
                .for_each(|(x, y, pixel)| {
                    let adjust = buf.get_pixel(x, y)[0] as i16 - luma8.get_pixel(x, y)[0] as i16;
                    pixel[0] = (pixel[0] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[1] = (pixel[1] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[2] = (pixel[2] as i16 + adjust).clamp(0, 255) as u8;
                });
        }
        ImageOp::BoxFilter(x_radius, y_radius) => {
            let luma8 = image.clone().into_luma8();
            let buf = box_filter(&luma8, x_radius, y_radius);
            image
                .as_mut_rgba8()
                .unwrap()
                .enumerate_pixels_mut()
                .for_each(|(x, y, pixel)| {
                    let adjust = buf.get_pixel(x, y)[0] as i16 - luma8.get_pixel(x, y)[0] as i16;
                    pixel[0] = (pixel[0] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[1] = (pixel[1] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[2] = (pixel[2] as i16 + adjust).clamp(0, 255) as u8;
                });
        }
        ImageOp::GaussianBlur(sigma) => {
            *image = gaussian_blur_f32(image.as_rgba8().unwrap(), sigma).into()
        }
        ImageOp::SharpenGaussian(sigma, amount) => {
            let luma8 = image.clone().into_luma8();
            let buf = sharpen_gaussian(&luma8, sigma, amount);
            image
                .as_mut_rgba8()
                .unwrap()
                .enumerate_pixels_mut()
                .for_each(|(x, y, pixel)| {
                    let adjust = buf.get_pixel(x, y)[0] as i16 - luma8.get_pixel(x, y)[0] as i16;
                    pixel[0] = (pixel[0] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[1] = (pixel[1] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[2] = (pixel[2] as i16 + adjust).clamp(0, 255) as u8;
                });
        }
        ImageOp::HorizontalFilter(kernel) => {
            *image = horizontal_filter(image.as_rgba8().unwrap(), &kernel).into()
        }
        ImageOp::VerticalFilter(kernel) => {
            *image = vertical_filter(image.as_rgba8().unwrap(), &kernel).into()
        }
        ImageOp::LaplacianFilter => {
            let luma8 = image.clone().into_luma8();
            let buf = laplacian_filter(&luma8);
            image
                .as_mut_rgba8()
                .unwrap()
                .enumerate_pixels_mut()
                .for_each(|(x, y, pixel)| {
                    let adjust = luma8.get_pixel(x, y)[0] as i16 + buf.get_pixel(x, y)[0];
                    pixel[0] = (pixel[0] as i16 + adjust) as u8;
                    pixel[1] = (pixel[1] as i16 + adjust) as u8;
                    pixel[2] = (pixel[2] as i16 + adjust) as u8;
                });
        }
        ImageOp::MedianFilter(x_radius, y_radius) => {
            *image = median_filter(image.as_rgba8().unwrap(), x_radius, y_radius).into()
        }
        ImageOp::SeparableFilter(h_kernel, v_kernel) => {
            *image = separable_filter(image.as_rgba8().unwrap(), &h_kernel, &v_kernel).into()
        }
        ImageOp::SeparableFilterEqual(kernel) => {
            *image = separable_filter_equal(image.as_rgba8().unwrap(), &kernel).into()
        }
        ImageOp::Sharpen3x3 => {
            let luma8 = image.clone().into_luma8();
            let buf = sharpen3x3(&luma8);
            image
                .as_mut_rgba8()
                .unwrap()
                .enumerate_pixels_mut()
                .for_each(|(x, y, pixel)| {
                    let adjust = buf.get_pixel(x, y)[0] as i16 - luma8.get_pixel(x, y)[0] as i16;
                    pixel[0] = (pixel[0] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[1] = (pixel[1] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[2] = (pixel[2] as i16 + adjust).clamp(0, 255) as u8;
                });
        }
        ImageOp::Rotate(cx, cy, angle, interp) => {
            *image = rotate(
                image.as_rgba8().unwrap(),
                (cx, cy),
                angle.to_radians(),
                interp,
                image::Rgba([0; 4]),
            )
            .into()
        }
        ImageOp::RotateAboutCenter(angle, interp) => {
            *image = rotate_about_center(
                image.as_rgba8().unwrap(),
                angle.to_radians(),
                interp,
                image::Rgba([0; 4]),
            )
            .into()
        }
        ImageOp::Translate(tx, ty) => {
            *image = translate(image.as_rgba8().unwrap(), (tx, ty)).into()
        }
        ImageOp::Warp(transform, interp) => {
            *image = warp(
                image.as_rgba8().unwrap(),
                &Projection::from_matrix(transform).unwrap(),
                interp,
                image::Rgba([0; 4]),
            )
            .into()
        }
        ImageOp::HorizontalPrewitt => {
            let luma8 = image.clone().into_luma8();
            let buf = horizontal_prewitt(&luma8);
            image
                .as_mut_rgba8()
                .unwrap()
                .enumerate_pixels_mut()
                .for_each(|(x, y, pixel)| {
                    let adjust = buf.get_pixel(x, y)[0];
                    pixel[0] = (pixel[0] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[1] = (pixel[1] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[2] = (pixel[2] as i16 + adjust).clamp(0, 255) as u8;
                });
        }
        ImageOp::HorizontalScharr => {
            let luma8 = image.clone().into_luma8();
            let buf = horizontal_scharr(&luma8);
            image
                .as_mut_rgba8()
                .unwrap()
                .enumerate_pixels_mut()
                .for_each(|(x, y, pixel)| {
                    let adjust = buf.get_pixel(x, y)[0];
                    pixel[0] = (pixel[0] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[1] = (pixel[1] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[2] = (pixel[2] as i16 + adjust).clamp(0, 255) as u8;
                });
        }
        ImageOp::HorizontalSobel => {
            let luma8 = image.clone().into_luma8();
            let buf = horizontal_sobel(&luma8);
            image
                .as_mut_rgba8()
                .unwrap()
                .enumerate_pixels_mut()
                .for_each(|(x, y, pixel)| {
                    let adjust = buf.get_pixel(x, y)[0];
                    pixel[0] = (pixel[0] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[1] = (pixel[1] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[2] = (pixel[2] as i16 + adjust).clamp(0, 255) as u8;
                });
        }
        ImageOp::VerticalPrewitt => {
            let luma8 = image.clone().into_luma8();
            let buf = vertical_prewitt(&luma8);
            image
                .as_mut_rgba8()
                .unwrap()
                .enumerate_pixels_mut()
                .for_each(|(x, y, pixel)| {
                    let adjust = buf.get_pixel(x, y)[0];
                    pixel[0] = (pixel[0] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[1] = (pixel[1] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[2] = (pixel[2] as i16 + adjust).clamp(0, 255) as u8;
                });
        }
        ImageOp::VerticalScharr => {
            let luma8 = image.clone().into_luma8();
            let buf = vertical_scharr(&luma8);
            image
                .as_mut_rgba8()
                .unwrap()
                .enumerate_pixels_mut()
                .for_each(|(x, y, pixel)| {
                    let adjust = buf.get_pixel(x, y)[0];
                    pixel[0] = (pixel[0] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[1] = (pixel[1] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[2] = (pixel[2] as i16 + adjust).clamp(0, 255) as u8;
                });
        }
        ImageOp::VerticalSobel => {
            let luma8 = image.clone().into_luma8();
            let buf = vertical_sobel(&luma8);
            image
                .as_mut_rgba8()
                .unwrap()
                .enumerate_pixels_mut()
                .for_each(|(x, y, pixel)| {
                    let adjust = buf.get_pixel(x, y)[0];
                    pixel[0] = (pixel[0] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[1] = (pixel[1] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[2] = (pixel[2] as i16 + adjust).clamp(0, 255) as u8;
                });
        }
        ImageOp::PrewittGradients => {
            let luma8 = image.clone().into_luma8();
            let buf = DynamicImage::from(prewitt_gradients(&luma8)).into_luma8();
            image
                .as_mut_rgba8()
                .unwrap()
                .enumerate_pixels_mut()
                .for_each(|(x, y, pixel)| {
                    let adjust = buf.get_pixel(x, y)[0] as i16;
                    pixel[0] = (pixel[0] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[1] = (pixel[1] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[2] = (pixel[2] as i16 + adjust).clamp(0, 255) as u8;
                });
        }
        ImageOp::SobelGradients => {
            let luma8 = image.clone().into_luma8();
            let buf = DynamicImage::from(sobel_gradients(&luma8)).into_luma8();
            image
                .as_mut_rgba8()
                .unwrap()
                .enumerate_pixels_mut()
                .for_each(|(x, y, pixel)| {
                    let adjust = buf.get_pixel(x, y)[0] as i16;
                    pixel[0] = (pixel[0] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[1] = (pixel[1] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[2] = (pixel[2] as i16 + adjust).clamp(0, 255) as u8;
                });
        }
        ImageOp::IntegralImage => {
            *image = DynamicImage::ImageRgba8(integral_image(image.as_rgba8().unwrap()))
        }
        ImageOp::IntegralSquaredImage => {
            *image = DynamicImage::ImageRgba8(integral_squared_image(image.as_rgba8().unwrap()))
        }
        ImageOp::RedChannel => {
            image
                .as_mut_rgba8()
                .unwrap()
                .pixels_mut()
                .for_each(|pixel| {
                    pixel[1] = 0;
                    pixel[2] = 0;
                });
        }
        ImageOp::GreenChannel => {
            image
                .as_mut_rgba8()
                .unwrap()
                .pixels_mut()
                .for_each(|pixel| {
                    pixel[0] = 0;
                    pixel[2] = 0;
                });
        }
        ImageOp::BlueChannel => {
            image
                .as_mut_rgba8()
                .unwrap()
                .pixels_mut()
                .for_each(|pixel| {
                    pixel[0] = 0;
                    pixel[1] = 0;
                });
        }
        ImageOp::Close(norm, k) => {
            let luma8 = image.clone().into_luma8();
            let buf = close(&luma8, norm, k);
            image
                .as_mut_rgba8()
                .unwrap()
                .enumerate_pixels_mut()
                .for_each(|(x, y, pixel)| {
                    let adjust = buf.get_pixel(x, y)[0] as i16 - luma8.get_pixel(x, y)[0] as i16;
                    pixel[0] = (pixel[0] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[1] = (pixel[1] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[2] = (pixel[2] as i16 + adjust).clamp(0, 255) as u8;
                });
        }
        ImageOp::Dilate(norm, k) => {
            let luma8 = image.clone().into_luma8();
            let buf = dilate(&luma8, norm, k);
            image
                .as_mut_rgba8()
                .unwrap()
                .enumerate_pixels_mut()
                .for_each(|(x, y, pixel)| {
                    let adjust = buf.get_pixel(x, y)[0] as i16 - luma8.get_pixel(x, y)[0] as i16;
                    pixel[0] = (pixel[0] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[1] = (pixel[1] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[2] = (pixel[2] as i16 + adjust).clamp(0, 255) as u8;
                });
        }
        ImageOp::Erode(norm, k) => {
            let luma8 = image.clone().into_luma8();
            let buf = erode(&luma8, norm, k);
            image
                .as_mut_rgba8()
                .unwrap()
                .enumerate_pixels_mut()
                .for_each(|(x, y, pixel)| {
                    let adjust = buf.get_pixel(x, y)[0] as i16 - luma8.get_pixel(x, y)[0] as i16;
                    pixel[0] = (pixel[0] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[1] = (pixel[1] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[2] = (pixel[2] as i16 + adjust).clamp(0, 255) as u8;
                });
        }
        ImageOp::Open(norm, k) => {
            let luma8 = image.clone().into_luma8();
            let buf = open(&luma8, norm, k);
            image
                .as_mut_rgba8()
                .unwrap()
                .enumerate_pixels_mut()
                .for_each(|(x, y, pixel)| {
                    let adjust = buf.get_pixel(x, y)[0] as i16 - luma8.get_pixel(x, y)[0] as i16;
                    pixel[0] = (pixel[0] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[1] = (pixel[1] as i16 + adjust).clamp(0, 255) as u8;
                    pixel[2] = (pixel[2] as i16 + adjust).clamp(0, 255) as u8;
                });
        }
        ImageOp::GaussianNoise(mean, stddev, seed) => {
            *image = gaussian_noise(image.as_rgba8().unwrap(), mean, stddev, seed).into()
        }
        ImageOp::SaltAndPepperNoise(rate, seed) => {
            *image = salt_and_pepper_noise(image.as_rgba8().unwrap(), rate, seed).into()
        }
        ImageOp::SuppressNonMaximum(radius) => {
            let luma8 = image.clone().into_luma8();
            let buf = suppress_non_maximum(&luma8, radius);
            image
                .as_mut_rgba8()
                .unwrap()
                .enumerate_pixels_mut()
                .for_each(|(x, y, pixel)| {
                    if buf.get_pixel(x, y)[0] == 0 {
                        pixel[0] = 0;
                        pixel[1] = 0;
                        pixel[2] = 0;
                    }
                });
        }
        ImageOp::PixelSort(mode, direction) => {
            let mut rgb_image = image.clone().into_rgb8();
            sort_with_options(&mut rgb_image, &Options { mode, direction });
            let img: DynamicImage = rgb_image.into();
            *image = img.into_rgba8().into();
        }
    }
}

pub fn render(shape: Rc<RefCell<Shape>>, width: u32, height: u32) -> Result<Pixmap> {
    let mut data = Vec::new();
    convert_shape(&mut data, shape, IDENTITY)?;
    data.sort_by(|a, b| a.zindex().partial_cmp(&b.zindex()).unwrap());

    let mut pixmap = Pixmap::new(width, height).unwrap();
    for shape_data in data {
        render_to_pixmap(shape_data, &mut pixmap, width, height)?;
    }

    Ok(pixmap)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "alloc")]
    use alloc::vec;

    use palette::Hsla;

    // Helper function to create a simple shape for testing
    fn create_test_shape() -> Rc<RefCell<Shape>> {
        Rc::new(RefCell::new(Shape::Basic(
            BasicShape::Square {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
                transform: Transform::identity(),
                zindex: Some(0.0),
                color: Color::Solid(Hsla::new(0.0, 1.0, 0.5, 1.0)),
                blend_mode: BlendMode::SourceOver,
                anti_alias: true,
                style: Style::Fill(FillRule::Winding),
            },
            None,
            None,
        )))
    }

    #[test]
    fn test_convert_color() {
        let palette_color = Rgba::new(1.0, 0.5, 0.25, 0.75);
        let skia_color = convert_color(palette_color);

        assert_eq!(skia_color.red(), 1.0);
        assert_eq!(skia_color.green(), 0.5);
        assert_eq!(skia_color.blue(), 0.25);
        assert_eq!(skia_color.alpha(), 0.75);
    }

    #[test]
    fn test_solid_paint() {
        let color = Rgba::new(0.1, 0.2, 0.3, 0.4);
        let paint = solid_paint(color, BlendMode::Multiply, false);

        assert_eq!(paint.blend_mode, BlendMode::Multiply);
        assert!(!paint.anti_alias);
        if let Shader::SolidColor(c) = paint.shader {
            assert_eq!(c.red(), 0.1);
            assert_eq!(c.green(), 0.2);
            assert_eq!(c.blue(), 0.3);
            assert_eq!(c.alpha(), 0.4);
        } else {
            panic!("Expected solid color shader");
        }
    }

    #[test]
    fn test_gradient_paint() {
        let gradient = Gradient {
            start: (0.0, 0.0),
            end: (100.0, 100.0),
            radius: None,
            stops: vec![
                (0.0, Hsla::new(0.0, 1.0, 0.5, 1.0)),
                (1.0, Hsla::new(120.0, 0.8, 0.6, 0.8)),
            ],
            spread_mode: SpreadMode::Pad,
            transform: Transform::identity(),
        };

        let paint = gradient_paint(gradient, BlendMode::Screen, true);

        assert_eq!(paint.blend_mode, BlendMode::Screen);
        assert!(paint.anti_alias);
        assert!(matches!(paint.shader, Shader::LinearGradient(_)));
    }

    #[test]
    fn test_convert_shape_basic() {
        let shape = create_test_shape();
        let mut data = Vec::new();
        convert_shape(&mut data, shape, Transform::identity()).unwrap();

        assert_eq!(data.len(), 1);
        if let ShapeData::FillPath { path, paint, .. } = &data[0] {
            assert!(path.bounds().width() > 0.0);
            assert!(path.bounds().height() > 0.0);
            if let Shader::SolidColor(c) = paint.shader {
                assert_eq!(c.red(), 1.0); // Red for HSL(0, 1.0, 0.5)
            } else {
                panic!("Expected solid color shader");
            }
        } else {
            panic!("Expected FillPath variant");
        }
    }

    #[test]
    fn test_render_basic_shape() {
        let shape = create_test_shape();
        let pixmap = render(shape, 200, 200).unwrap();

        // Basic checks that rendering produced something
        assert_eq!(pixmap.width(), 200);
        assert_eq!(pixmap.height(), 200);
        assert!(pixmap.data().iter().any(|&b| b != 0)); // Not all pixels are transparent
    }

    #[test]
    fn test_zindex_ordering() {
        let mut data = vec![
            ShapeData::Fill {
                color: tiny_skia::Color::from_rgba(1.0, 0.0, 0.0, 1.0).unwrap(),
                zindex: 1.0,
            },
            ShapeData::Fill {
                color: tiny_skia::Color::from_rgba(0.0, 1.0, 0.0, 1.0).unwrap(),
                zindex: 0.0,
            },
        ];

        // Sort by zindex
        data.sort_by(|a, b| a.zindex().partial_cmp(&b.zindex()).unwrap());

        assert_eq!(data[0].zindex(), 0.0);
        assert_eq!(data[1].zindex(), 1.0);
    }

    #[test]
    fn test_color_overwrites() {
        let original = Color::Solid(Hsla::new(0.0, 1.0, 0.5, 1.0));
        let overwrite = ColorChange::Hsla(HslaChange {
            hue: Some(120.0.into()),
            ..Default::default()
        });
        let shift = HslaChange {
            lightness: Some(0.1.into()),
            ..Default::default()
        };

        let result = overwrite_color(original, overwrite, shift);

        if let Color::Solid(hsla) = result {
            assert_eq!(hsla.hue, 120.0);
            assert_eq!(hsla.lightness, 0.6); // 0.5 + 0.1
        } else {
            panic!("Expected solid color");
        }
    }

    #[test]
    #[cfg(all(feature = "std", feature = "io"))]
    fn test_image_rendering() {
        let shape = Rc::new(RefCell::new(Shape::Image {
            path: ImagePath::File("test.png".into()),
            ops: vec![],
            transform: Transform::identity(),
            zindex: Some(0.0),
            opacity: 1.0,
            blend_mode: BlendMode::SourceOver,
            quality: FilterQuality::Nearest,
            mask: None,
        }));

        // This test just verifies the image path is handled without panic
        let result = render(shape, 100, 100);
        assert!(result.is_ok());
    }

    #[test]
    fn test_composite_shape_rendering() {
        let shape_a = create_test_shape();
        let shape_b = Rc::new(RefCell::new(Shape::Basic(
            BasicShape::Circle {
                x: 50.0,
                y: 50.0,
                radius: 25.0,
                transform: Transform::identity(),
                zindex: Some(1.0),
                color: Color::Solid(Hsla::new(120.0, 1.0, 0.5, 0.5)),
                blend_mode: BlendMode::SourceOver,
                anti_alias: true,
                style: Style::Fill(FillRule::Winding),
            },
            None,
            None,
        )));

        let composite = Rc::new(RefCell::new(Shape::Composite {
            a: shape_a,
            b: shape_b,
            transform: Transform::identity(),
            zindex_overwrite: None,
            zindex_shift: None,
            color_overwrite: ColorChange::default(),
            color_shift: HslaChange::default(),
            blend_mode_overwrite: None,
            anti_alias_overwrite: None,
            style_overwrite: None,
            mask_overwrite: None,
            pattern_overwrite: None,
        }));

        let pixmap = render(composite, 200, 200).unwrap();
        assert_eq!(pixmap.width(), 200);
        assert_eq!(pixmap.height(), 200);
    }

    #[test]
    fn test_path_shape_rendering() {
        let path_shape = Rc::new(RefCell::new(Shape::Path {
            segments: vec![
                PathSegment::MoveTo(0.0, 0.0),
                PathSegment::LineTo(100.0, 0.0),
                PathSegment::LineTo(50.0, 100.0),
                PathSegment::Close,
            ],
            transform: Transform::identity(),
            zindex: Some(0.0),
            color: Color::Solid(Hsla::new(240.0, 1.0, 0.5, 1.0)),
            blend_mode: BlendMode::SourceOver,
            anti_alias: true,
            style: Style::Stroke(Stroke {
                width: 2.0,
                ..Default::default()
            }),
            mask: None,
            pattern: None,
        }));

        let pixmap = render(path_shape, 200, 200).unwrap();
        assert_eq!(pixmap.width(), 200);
        assert_eq!(pixmap.height(), 200);
    }
}
