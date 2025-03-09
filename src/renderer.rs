#[cfg(feature = "std")]
use std::rc::Rc;

#[cfg(feature = "no-std")]
use alloc::{rc::Rc, vec::Vec};

use crate::shape::{BasicShape, HslaChange, PathSegment, Shape, IDENTITY};

use anyhow::Result;
use core::ops::Add;
use palette::{rgb::Rgba, FromColor, Hsla};
use tiny_skia::{Color, FillRule, Paint, Path, PathBuilder, Pixmap, Rect, Shader, Transform};

#[derive(Debug)]
enum ShapeData<'a> {
    FillPath {
        path: Path,
        transform: Transform,
        zindex: f32,
        paint: Paint<'a>,
    },
    Fill {
        zindex: f32,
        color: Color,
    },
}

fn convert_color(color: Rgba<f32>) -> Color {
    Color::from_rgba(
        color.red.clamp(0.0, 1.0),
        color.green.clamp(0.0, 1.0),
        color.blue.clamp(0.0, 1.0),
        color.alpha.clamp(0.0, 1.0),
    )
    .unwrap()
}

fn solid_color_paint<'a>(color: Rgba<f32>) -> Paint<'a> {
    Paint {
        shader: Shader::SolidColor(convert_color(color)),
        ..Paint::default()
    }
}

fn overwrite_zindex(
    zindex: Option<f32>,
    zindex_overwrite: Option<f32>,
    zindex_shift: Option<f32>,
) -> f32 {
    match zindex_overwrite {
        Some(zindex) => zindex + zindex_shift.unwrap_or(0.0),
        None => zindex.unwrap_or(0.0) + zindex_shift.unwrap_or(0.0),
    }
}

fn overwrite_color(
    color: Hsla<f32>,
    color_overwrite: HslaChange,
    color_shift: HslaChange,
) -> Hsla<f32> {
    let hue = match color_overwrite.hue {
        Some(hue) => hue + color_shift.hue.unwrap_or(0.0.into()),
        None => color.hue + color_shift.hue.unwrap_or(0.0.into()),
    };
    let saturation = match color_overwrite.saturation {
        Some(saturation) => saturation + color_shift.saturation.unwrap_or(0.0),
        None => color.saturation + color_shift.saturation.unwrap_or(0.0),
    };
    let lightness = match color_overwrite.lightness {
        Some(lightness) => lightness + color_shift.lightness.unwrap_or(0.0),
        None => color.lightness + color_shift.lightness.unwrap_or(0.0),
    };
    let alpha = match color_overwrite.alpha {
        Some(alpha) => alpha + color_shift.alpha.unwrap_or(0.0),
        None => color.alpha + color_shift.alpha.unwrap_or(0.0),
    };
    Hsla::new(hue, saturation, lightness, alpha)
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
    color_overwrite: HslaChange,
    color_shift: HslaChange,
    curr_color_overwrite: HslaChange,
    curr_color_shift: HslaChange,
) -> (HslaChange, HslaChange) {
    let color_overwrite = HslaChange {
        hue: color_overwrite.hue.or(curr_color_overwrite.hue),
        saturation: color_overwrite
            .saturation
            .or(curr_color_overwrite.saturation),
        lightness: color_overwrite.lightness.or(curr_color_overwrite.lightness),
        alpha: color_overwrite.alpha.or(curr_color_overwrite.alpha),
    };
    let color_shift = HslaChange {
        hue: combine_shift(color_shift.hue, curr_color_shift.hue),
        saturation: combine_shift(color_shift.saturation, curr_color_shift.saturation),
        lightness: combine_shift(color_shift.lightness, curr_color_shift.lightness),
        alpha: combine_shift(color_shift.alpha, curr_color_shift.alpha),
    };
    (color_overwrite, color_shift)
}

fn convert_shape(
    data: &mut Vec<ShapeData>,
    shape: Shape,
    parent_transform: Transform,
    zindex_overwrite: Option<f32>,
    zindex_shift: Option<f32>,
    color_overwrite: HslaChange,
    color_shift: HslaChange,
) -> Result<()> {
    match shape {
        Shape::Basic(BasicShape::Square {
            x,
            y,
            width,
            height,
            transform,
            zindex,
            color,
        }) => {
            let path = PathBuilder::from_rect(Rect::from_xywh(x, y, width, height).unwrap());
            let transform = transform.post_concat(parent_transform);
            let zindex = overwrite_zindex(zindex, zindex_overwrite, zindex_shift);
            let color = overwrite_color(color, color_overwrite, color_shift);
            let paint = solid_color_paint(Rgba::from_color(*color));
            data.push(ShapeData::FillPath {
                path,
                transform,
                zindex,
                paint,
            });
        }
        Shape::Basic(BasicShape::Circle {
            x,
            y,
            radius,
            transform,
            zindex,
            color,
        }) => {
            let path = PathBuilder::from_circle(x, y, radius).unwrap();
            let transform = transform.post_concat(parent_transform);
            let zindex = overwrite_zindex(zindex, zindex_overwrite, zindex_shift);
            let color = overwrite_color(color, color_overwrite, color_shift);
            let paint = solid_color_paint(Rgba::from_color(*color));
            data.push(ShapeData::FillPath {
                path,
                transform,
                zindex,
                paint,
            });
        }
        Shape::Basic(BasicShape::Triangle {
            points,
            transform,
            zindex,
            color,
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
            let paint = solid_color_paint(Rgba::from_color(*color));
            data.push(ShapeData::FillPath {
                path,
                transform,
                zindex,
                paint,
            });
        }
        Shape::Basic(BasicShape::Fill { zindex, color }) => {
            let zindex = zindex.unwrap_or(0.0);
            let color = convert_color(Rgba::from_color(*color));
            data.push(ShapeData::Fill { zindex, color });
        }
        Shape::Basic(BasicShape::Empty) => (),
        Shape::Path {
            segments,
            transform,
            zindex,
            color,
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
                let paint = solid_color_paint(Rgba::from_color(*color));

                data.push(ShapeData::FillPath {
                    path,
                    transform,
                    zindex,
                    paint,
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

            let a = Rc::try_unwrap(a).unwrap().into_inner();
            convert_shape(
                data,
                a,
                transform,
                zindex_overwrite,
                zindex_shift,
                color_overwrite,
                color_shift,
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
            )?;
        }
        Shape::Collection {
            shapes,
            transform,
            zindex_overwrite: curr_zindex_overwrite,
            zindex_shift: curr_zindex_shift,
            color_overwrite: curr_color_overwrite,
            color_shift: curr_color_shift,
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

            for shape in shapes {
                let shape = Rc::try_unwrap(shape).unwrap().into_inner();
                convert_shape(
                    data,
                    shape,
                    transform,
                    zindex_overwrite,
                    zindex_shift,
                    color_overwrite,
                    color_shift,
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
        HslaChange::default(),
        HslaChange::default(),
    )?;
    data.sort_by(|a, b| match (a, b) {
        (ShapeData::FillPath { zindex: a, .. }, ShapeData::FillPath { zindex: b, .. })
        | (ShapeData::FillPath { zindex: a, .. }, ShapeData::Fill { zindex: b, .. })
        | (ShapeData::Fill { zindex: a, .. }, ShapeData::FillPath { zindex: b, .. })
        | (ShapeData::Fill { zindex: a, .. }, ShapeData::Fill { zindex: b, .. }) => {
            a.partial_cmp(b).unwrap()
        }
    });

    let mut pixmap = Pixmap::new(width, height).unwrap();
    for shape_data in data {
        match shape_data {
            ShapeData::FillPath {
                path,
                transform,
                paint,
                ..
            } => {
                let transform = transform
                    .post_scale(1.0, -1.0)
                    .post_translate(width as f32 / 2.0, height as f32 / 2.0);
                pixmap.fill_path(&path, &paint, FillRule::Winding, transform, None);
            }
            ShapeData::Fill { color, .. } => {
                pixmap.fill(color);
            }
        }
    }
    Ok(pixmap)
}
