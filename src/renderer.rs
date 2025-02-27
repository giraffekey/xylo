use crate::shape::{unwrap_shape, BasicShape, Shape};

use anyhow::Result;
use palette::{blend::Blend, rgb::Rgba, FromColor};
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Rect, Shader, Transform};

fn solid_color_paint<'a>(color: Rgba<f32>) -> Paint<'a> {
    let color = Color::from_rgba(color.red, color.green, color.blue, color.alpha).unwrap();
    Paint {
        shader: Shader::SolidColor(color),
        ..Paint::default()
    }
}

fn render_shape(
    pixmap: &mut Pixmap,
    shape: Shape,
    parent_transform: Transform,
    parent_color: Rgba<f32>,
) -> Result<()> {
    match shape {
        Shape::Basic(BasicShape::Square {
            x,
            y,
            width,
            height,
            transform,
            color,
        }) => {
            let path = PathBuilder::from_rect(Rect::from_xywh(x, y, width, height).unwrap());
            let transform = transform.post_concat(parent_transform);
            let color = Rgba::from_color(color).overlay(parent_color);
            let paint = solid_color_paint(color);
            pixmap.fill_path(&path, &paint, FillRule::Winding, transform, None);
        }
        Shape::Basic(BasicShape::Circle {
            x,
            y,
            radius,
            transform,
            color,
        }) => {
            let path = PathBuilder::from_circle(x, y, radius).unwrap();
            let transform = transform.post_concat(parent_transform);
            let color = Rgba::from_color(color).overlay(parent_color);
            let paint = solid_color_paint(color);
            pixmap.fill_path(&path, &paint, FillRule::Winding, transform, None);
        }
        Shape::Basic(BasicShape::Triangle {
            points,
            transform,
            color,
        }) => {
            let mut pb = PathBuilder::new();
            pb.move_to(points[0], points[1]);
            pb.line_to(points[2], points[3]);
            pb.line_to(points[4], points[5]);
            pb.close();
            let path = pb.finish().unwrap();

            let transform = transform.post_concat(parent_transform);
            let color = Rgba::from_color(color).overlay(parent_color);
            let paint = solid_color_paint(color);
            pixmap.fill_path(&path, &paint, FillRule::Winding, transform, None);
        }
        Shape::Basic(BasicShape::Fill { color }) => {
            let color = Rgba::from_color(*color).overlay(parent_color);
            let color = Color::from_rgba(color.red, color.green, color.blue, color.alpha).unwrap();
            pixmap.fill(color);
        }
        Shape::Composite {
            a,
            b,
            transform,
            color,
        } => {
            let transform = transform.post_concat(parent_transform);
            let color = Rgba::from_color(color).overlay(parent_color);
            let a = unwrap_shape(a)?;
            render_shape(pixmap, a, transform, color)?;
            let b = unwrap_shape(b)?;
            render_shape(pixmap, b, transform, color)?;
        }
        Shape::Collection {
            shapes,
            transform,
            color,
        } => {
            let transform = transform.post_concat(parent_transform);
            let color = Rgba::from_color(color).overlay(parent_color);
            for shape in shapes {
                let shape = unwrap_shape(shape)?;
                render_shape(pixmap, shape, transform, color)?;
            }
        }
    }
    Ok(())
}

pub fn render(shape: Shape, width: u32, height: u32) -> Result<Pixmap> {
    let mut pixmap = Pixmap::new(width, height).unwrap();
    let transform =
        Transform::from_translate(pixmap.width() as f32 / 2.0, pixmap.height() as f32 / 2.0);
    render_shape(&mut pixmap, shape, transform, Rgba::new(1.0, 1.0, 1.0, 0.0))?;
    Ok(pixmap)
}
