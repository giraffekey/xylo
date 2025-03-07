#[cfg(feature = "std")]
use std::rc::Rc;

#[cfg(feature = "no-std")]
use alloc::rc::Rc;

use crate::shape::{BasicShape, PathSegment, Shape};

use anyhow::Result;
use palette::{blend::Blend, rgb::Rgba, FromColor};
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

fn solid_color_paint<'a>(color: Rgba<f32>) -> Paint<'a> {
    let color = Color::from_rgba(color.red, color.green, color.blue, color.alpha).unwrap();
    Paint {
        shader: Shader::SolidColor(color),
        ..Paint::default()
    }
}

fn convert_shape(
    data: &mut Vec<ShapeData>,
    shape: Shape,
    parent_transform: Transform,
    parent_zindex: Option<f32>,
    parent_color: Rgba<f32>,
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
            let zindex = parent_zindex.unwrap_or(zindex.unwrap_or(0.0));
            let color = Rgba::from_color(color).overlay(parent_color);
            let paint = solid_color_paint(color);
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
            let zindex = parent_zindex.unwrap_or(zindex.unwrap_or(0.0));
            let color = Rgba::from_color(color).overlay(parent_color);
            let paint = solid_color_paint(color);
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
            let zindex = parent_zindex.unwrap_or(zindex.unwrap_or(0.0));
            let color = Rgba::from_color(color).overlay(parent_color);
            let paint = solid_color_paint(color);
            data.push(ShapeData::FillPath {
                path,
                transform,
                zindex,
                paint,
            });
        }
        Shape::Basic(BasicShape::Fill { zindex, color }) => {
            let zindex = parent_zindex.unwrap_or(zindex.unwrap_or(0.0));
            let color = Rgba::from_color(*color).overlay(parent_color);
            let color = Color::from_rgba(color.red, color.green, color.blue, color.alpha).unwrap();
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
                let zindex = parent_zindex.unwrap_or(zindex.unwrap_or(0.0));
                let color = Rgba::from_color(color).overlay(parent_color);
                let paint = solid_color_paint(color);

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
            zindex,
            color,
        } => {
            let transform = transform.post_concat(parent_transform);
            let zindex = match parent_zindex {
                Some(_) => parent_zindex,
                None => zindex,
            };
            let color = Rgba::from_color(color).overlay(parent_color);
            let a = Rc::try_unwrap(a).unwrap().into_inner();
            convert_shape(data, a, transform, zindex, color)?;
            let b = Rc::try_unwrap(b).unwrap().into_inner();
            convert_shape(data, b, transform, zindex, color)?;
        }
        Shape::Collection {
            shapes,
            transform,
            zindex,
            color,
        } => {
            let transform = transform.post_concat(parent_transform);
            let zindex = match parent_zindex {
                Some(_) => parent_zindex,
                None => zindex,
            };
            let color = Rgba::from_color(color).overlay(parent_color);
            for shape in shapes {
                let shape = Rc::try_unwrap(shape).unwrap().into_inner();
                convert_shape(data, shape, transform, zindex, color)?;
            }
        }
    }
    Ok(())
}

pub fn render(shape: Shape, width: u32, height: u32) -> Result<Pixmap> {
    let mut data = Vec::new();
    let transform = Transform::from_translate(width as f32 / 2.0, height as f32 / 2.0);
    let color = Rgba::new(1.0, 1.0, 1.0, 0.0);
    convert_shape(&mut data, shape, transform, None, color)?;
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
                pixmap.fill_path(&path, &paint, FillRule::Winding, transform, None);
            }
            ShapeData::Fill { color, .. } => {
                pixmap.fill(color);
            }
        }
    }
    Ok(pixmap)
}
