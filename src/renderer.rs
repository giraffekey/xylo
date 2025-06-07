#[cfg(feature = "std")]
use std::rc::Rc;

#[cfg(feature = "no-std")]
use alloc::{rc::Rc, vec::Vec};

use crate::error::Result;
use crate::shape::{
    BasicShape, Color, ColorChange, Gradient, HslaChange, PathSegment, Shape, Style, IDENTITY,
    WHITE,
};

use core::ops::Add;
use palette::{rgb::Rgba, FromColor};
use tiny_skia::{
    BlendMode, FillRule, GradientStop, LinearGradient, Paint, Path, PathBuilder, Pixmap,
    RadialGradient, Rect, Shader, Stroke, Transform,
};

#[derive(Debug)]
enum ShapeData<'a> {
    FillPath {
        path: Path,
        transform: Transform,
        fill_rule: FillRule,
        paint: Paint<'a>,
        zindex: f32,
    },
    StrokePath {
        path: Path,
        transform: Transform,
        stroke: Stroke,
        paint: Paint<'a>,
        zindex: f32,
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
            gradient.mode,
            gradient.transform,
        )
        .unwrap(),
        None => LinearGradient::new(
            gradient.start.into(),
            gradient.end.into(),
            stops,
            gradient.mode,
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

fn convert_shape(
    data: &mut Vec<ShapeData>,
    shape: Shape,
    parent_transform: Transform,
    zindex_overwrite: Option<f32>,
    zindex_shift: Option<f32>,
    color_overwrite: ColorChange,
    color_shift: HslaChange,
    blend_mode_overwrite: Option<BlendMode>,
    anti_alias_overwrite: Option<bool>,
    style_overwrite: Option<Style>,
) -> Result<()> {
    match shape {
        Shape::Basic(BasicShape::Square {
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
        }) => {
            let path = PathBuilder::from_rect(Rect::from_xywh(x, y, w, h).unwrap());
            let transform = transform.post_concat(parent_transform);
            let zindex = overwrite_zindex(zindex, zindex_overwrite, zindex_shift);
            let color = overwrite_color(color, color_overwrite, color_shift);
            let blend_mode = overwrite_blend_mode(blend_mode, blend_mode_overwrite);
            let anti_alias = overwrite_anti_alias(anti_alias, anti_alias_overwrite);
            let style = overwrite_style(style, style_overwrite);
            let paint = match color {
                Color::Solid(color) => solid_paint(Rgba::from_color(color), blend_mode, anti_alias),
                Color::Gradient(gradient) => gradient_paint(gradient, blend_mode, anti_alias),
            };

            match style {
                Style::Fill(fill_rule) => {
                    data.push(ShapeData::FillPath {
                        path,
                        transform,
                        fill_rule,
                        paint,
                        zindex,
                    });
                }
                Style::Stroke(stroke) => {
                    data.push(ShapeData::StrokePath {
                        path,
                        transform,
                        stroke,
                        paint,
                        zindex,
                    });
                }
            }
        }
        Shape::Basic(BasicShape::Circle {
            x,
            y,
            radius,
            transform,
            zindex,
            color,
            blend_mode,
            anti_alias,
            style,
        }) => {
            let path = PathBuilder::from_circle(x, y, radius).unwrap();
            let transform = transform.post_concat(parent_transform);
            let zindex = overwrite_zindex(zindex, zindex_overwrite, zindex_shift);
            let color = overwrite_color(color, color_overwrite, color_shift);
            let blend_mode = overwrite_blend_mode(blend_mode, blend_mode_overwrite);
            let anti_alias = overwrite_anti_alias(anti_alias, anti_alias_overwrite);
            let style = overwrite_style(style, style_overwrite);
            let paint = match color {
                Color::Solid(color) => solid_paint(Rgba::from_color(color), blend_mode, anti_alias),
                Color::Gradient(gradient) => gradient_paint(gradient, blend_mode, anti_alias),
            };

            match style {
                Style::Fill(fill_rule) => {
                    data.push(ShapeData::FillPath {
                        path,
                        transform,
                        fill_rule,
                        paint,
                        zindex,
                    });
                }
                Style::Stroke(stroke) => {
                    data.push(ShapeData::StrokePath {
                        path,
                        transform,
                        stroke,
                        paint,
                        zindex,
                    });
                }
            }
        }
        Shape::Basic(BasicShape::Triangle {
            points,
            transform,
            zindex,
            color,
            blend_mode,
            anti_alias,
            style,
        }) => {
            let mut pb = PathBuilder::new();
            pb.move_to(points[0], points[1]);
            pb.line_to(points[2], points[3]);
            pb.line_to(points[4], points[5]);
            pb.close();
            let path = pb.finish().unwrap();

            let transform = transform.post_concat(parent_transform);
            let zindex = overwrite_zindex(zindex, zindex_overwrite, zindex_shift);
            let color = overwrite_color(color, color_overwrite, color_shift);
            let blend_mode = overwrite_blend_mode(blend_mode, blend_mode_overwrite);
            let anti_alias = overwrite_anti_alias(anti_alias, anti_alias_overwrite);
            let style = overwrite_style(style, style_overwrite);
            let paint = match color {
                Color::Solid(color) => solid_paint(Rgba::from_color(color), blend_mode, anti_alias),
                Color::Gradient(gradient) => gradient_paint(gradient, blend_mode, anti_alias),
            };

            match style {
                Style::Fill(fill_rule) => {
                    data.push(ShapeData::FillPath {
                        path,
                        transform,
                        fill_rule,
                        paint,
                        zindex,
                    });
                }
                Style::Stroke(stroke) => {
                    data.push(ShapeData::StrokePath {
                        path,
                        transform,
                        stroke,
                        paint,
                        zindex,
                    });
                }
            }
        }
        Shape::Basic(BasicShape::Fill { zindex, color }) => {
            let zindex = zindex.unwrap_or(0.0);
            let color = overwrite_color(color, color_overwrite, color_shift);
            match color {
                Color::Solid(color) => {
                    let color = convert_color(Rgba::from_color(color));
                    data.push(ShapeData::Fill { zindex, color });
                }
                Color::Gradient(gradient) => {
                    let paint = gradient_paint(gradient, BlendMode::SourceOver, true);
                    data.push(ShapeData::FillPaint { zindex, paint });
                }
            }
        }
        Shape::Basic(BasicShape::Empty) => (),
        Shape::Path {
            segments,
            transform,
            zindex,
            color,
            blend_mode,
            anti_alias,
            style,
        } => {
            let mut pb = PathBuilder::new();
            for segment in segments {
                match segment {
                    PathSegment::MoveTo(x, y) => pb.move_to(x, y),
                    PathSegment::LineTo(x, y) => pb.line_to(x, y),
                    PathSegment::QuadTo(x1, y1, x, y) => pb.quad_to(x1, y1, x, y),
                    PathSegment::CubicTo(x1, y1, x2, y2, x, y) => pb.cubic_to(x1, y1, x2, y2, x, y),
                    PathSegment::Close => pb.close(),
                }
            }
            let path = pb.finish();

            if let Some(path) = path {
                let transform = transform.post_concat(parent_transform);
                let zindex = overwrite_zindex(zindex, zindex_overwrite, zindex_shift);
                let color = overwrite_color(color, color_overwrite, color_shift);
                let blend_mode = overwrite_blend_mode(blend_mode, blend_mode_overwrite);
                let anti_alias = overwrite_anti_alias(anti_alias, anti_alias_overwrite);
                let style = overwrite_style(style, style_overwrite);
                let paint = match color {
                    Color::Solid(color) => {
                        solid_paint(Rgba::from_color(color), blend_mode, anti_alias)
                    }
                    Color::Gradient(gradient) => gradient_paint(gradient, blend_mode, anti_alias),
                };

                match style {
                    Style::Fill(fill_rule) => {
                        data.push(ShapeData::FillPath {
                            path,
                            transform,
                            fill_rule,
                            paint,
                            zindex,
                        });
                    }
                    Style::Stroke(stroke) => {
                        data.push(ShapeData::StrokePath {
                            path,
                            transform,
                            stroke,
                            paint,
                            zindex,
                        });
                    }
                }
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
        } => {
            let transform = transform.post_concat(parent_transform);
            let (zindex_overwrite, zindex_shift) = resolve_zindex_overwrites(
                zindex_overwrite,
                zindex_shift,
                curr_zindex_overwrite,
                curr_zindex_shift,
            );
            let (color_overwrite, color_shift) = resolve_color_overwrites(
                color_overwrite,
                color_shift,
                curr_color_overwrite,
                curr_color_shift,
            );
            let blend_mode_overwrite =
                resolve_blend_mode_overwrite(blend_mode_overwrite, curr_blend_mode_overwrite);
            let anti_alias_overwrite =
                resolve_anti_alias_overwrite(anti_alias_overwrite, curr_anti_alias_overwrite);
            let style_overwrite = resolve_style_overwrite(style_overwrite, curr_style_overwrite);

            let a = Rc::try_unwrap(a).unwrap().into_inner();
            convert_shape(
                data,
                a,
                transform,
                zindex_overwrite,
                zindex_shift,
                color_overwrite.clone(),
                color_shift,
                blend_mode_overwrite,
                anti_alias_overwrite,
                style_overwrite.clone(),
            )?;
            let b = Rc::try_unwrap(b).unwrap().into_inner();
            convert_shape(
                data,
                b,
                transform,
                zindex_overwrite,
                zindex_shift,
                color_overwrite,
                color_shift,
                blend_mode_overwrite,
                anti_alias_overwrite,
                style_overwrite,
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
        } => {
            let transform = transform.post_concat(parent_transform);
            let (zindex_overwrite, zindex_shift) = resolve_zindex_overwrites(
                zindex_overwrite,
                zindex_shift,
                curr_zindex_overwrite,
                curr_zindex_shift,
            );
            let (color_overwrite, color_shift) = resolve_color_overwrites(
                color_overwrite,
                color_shift,
                curr_color_overwrite,
                curr_color_shift,
            );
            let blend_mode_overwrite =
                resolve_blend_mode_overwrite(blend_mode_overwrite, curr_blend_mode_overwrite);
            let anti_alias_overwrite =
                resolve_anti_alias_overwrite(anti_alias_overwrite, curr_anti_alias_overwrite);
            let style_overwrite = resolve_style_overwrite(style_overwrite, curr_style_overwrite);

            for shape in shapes {
                let shape = Rc::try_unwrap(shape).unwrap().into_inner();
                convert_shape(
                    data,
                    shape,
                    transform,
                    zindex_overwrite,
                    zindex_shift,
                    color_overwrite.clone(),
                    color_shift,
                    blend_mode_overwrite,
                    anti_alias_overwrite,
                    style_overwrite.clone(),
                )?;
            }
        }
    }
    Ok(())
}

pub fn render(shape: Shape, width: u32, height: u32) -> Result<Pixmap> {
    let mut data = Vec::new();
    convert_shape(
        &mut data,
        shape,
        IDENTITY,
        None,
        None,
        ColorChange::default(),
        HslaChange::default(),
        None,
        None,
        None,
    )?;
    data.sort_by(|a, b| match (a, b) {
        (ShapeData::FillPath { zindex: a, .. }, ShapeData::FillPath { zindex: b, .. })
        | (ShapeData::FillPath { zindex: a, .. }, ShapeData::StrokePath { zindex: b, .. })
        | (ShapeData::FillPath { zindex: a, .. }, ShapeData::Fill { zindex: b, .. })
        | (ShapeData::FillPath { zindex: a, .. }, ShapeData::FillPaint { zindex: b, .. })
        | (ShapeData::StrokePath { zindex: a, .. }, ShapeData::FillPath { zindex: b, .. })
        | (ShapeData::StrokePath { zindex: a, .. }, ShapeData::StrokePath { zindex: b, .. })
        | (ShapeData::StrokePath { zindex: a, .. }, ShapeData::Fill { zindex: b, .. })
        | (ShapeData::StrokePath { zindex: a, .. }, ShapeData::FillPaint { zindex: b, .. })
        | (ShapeData::Fill { zindex: a, .. }, ShapeData::FillPath { zindex: b, .. })
        | (ShapeData::Fill { zindex: a, .. }, ShapeData::StrokePath { zindex: b, .. })
        | (ShapeData::Fill { zindex: a, .. }, ShapeData::Fill { zindex: b, .. })
        | (ShapeData::Fill { zindex: a, .. }, ShapeData::FillPaint { zindex: b, .. })
        | (ShapeData::FillPaint { zindex: a, .. }, ShapeData::FillPath { zindex: b, .. })
        | (ShapeData::FillPaint { zindex: a, .. }, ShapeData::StrokePath { zindex: b, .. })
        | (ShapeData::FillPaint { zindex: a, .. }, ShapeData::Fill { zindex: b, .. })
        | (ShapeData::FillPaint { zindex: a, .. }, ShapeData::FillPaint { zindex: b, .. }) => {
            a.partial_cmp(b).unwrap()
        }
    });

    let half_width = width as f32 / 2.0;
    let half_height = height as f32 / 2.0;

    let mut pixmap = Pixmap::new(width, height).unwrap();
    for shape_data in data {
        match shape_data {
            ShapeData::FillPath {
                path,
                transform,
                fill_rule,
                paint,
                ..
            } => {
                let transform = transform
                    .post_scale(1.0, -1.0)
                    .post_translate(half_width, half_height);
                pixmap.fill_path(&path, &paint, fill_rule, transform, None);
            }
            ShapeData::StrokePath {
                path,
                transform,
                stroke,
                paint,
                ..
            } => {
                let transform = transform
                    .post_scale(1.0, -1.0)
                    .post_translate(half_width, half_height);
                pixmap.stroke_path(&path, &paint, &stroke, transform, None);
            }
            ShapeData::Fill { color, .. } => {
                pixmap.fill(color);
            }
            ShapeData::FillPaint { paint, .. } => {
                let path = PathBuilder::from_rect(
                    Rect::from_xywh(-half_width, -half_height, width as f32, height as f32)
                        .unwrap(),
                );
                let transform = IDENTITY
                    .post_scale(1.0, -1.0)
                    .post_translate(half_width, half_height);
                pixmap.fill_path(&path, &paint, FillRule::Winding, transform, None);
            }
        }
    }
    Ok(pixmap)
}
