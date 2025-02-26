#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};

use crate::compiler::{Shape, IDENTITY};

use palette::{blend::Blend, rgb::Rgba, FromColor, Hsla};
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Rect, Shader, Transform};

fn solid_color_paint<'a>(color: Rgba<f32>) -> Paint<'a> {
    Paint {
        shader: Shader::SolidColor(to_skia_color(color)),
        ..Paint::default()
    }
}

fn to_skia_color(color: Rgba<f32>) -> Color {
    Color::from_rgba(color.red, color.green, color.blue, color.alpha).unwrap()
}

fn render_shape(
    pixmap: &mut Pixmap,
    shape: Shape,
    parent_transform: Transform,
    parent_color: Rgba<f32>,
) {
    match shape {
        Shape::Square {
            x,
            y,
            width,
            height,
            transform,
            color,
        } => {
            let path = PathBuilder::from_rect(Rect::from_xywh(x, y, width, height).unwrap());
            let transform = transform.post_concat(parent_transform);
            let color = Rgba::from_color(color).overlay(parent_color);
            let paint = solid_color_paint(color);
            pixmap.fill_path(&path, &paint, FillRule::Winding, transform, None);
        }
        Shape::Circle {
            x,
            y,
            radius,
            transform,
            color,
        } => {
            let path = PathBuilder::from_circle(x, y, radius).unwrap();
            let transform = transform.post_concat(parent_transform);
            let color = Rgba::from_color(color).overlay(parent_color);
            let paint = solid_color_paint(color);
            pixmap.fill_path(&path, &paint, FillRule::Winding, transform, None);
        }
        Shape::Triangle {
            points,
            transform,
            color,
        } => {
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
        Shape::Fill { color } => {
            let color = Rgba::from_color(color).overlay(parent_color);
            let color = to_skia_color(color);
            pixmap.fill(color);
        }
        Shape::Composite {
            shapes,
            transform,
            color,
        } => {
            for shape in shapes {
                let transform = transform.post_concat(parent_transform);
                let color = Rgba::from_color(color).overlay(parent_color);
                render_shape(pixmap, *shape, transform, color);
            }
        }
    }
}

pub fn render(shape: Shape, width: u32, height: u32) -> Pixmap {
    let mut pixmap = Pixmap::new(width, height).unwrap();
    let transform =
        Transform::from_translate(pixmap.width() as f32 / 2.0, pixmap.height() as f32 / 2.0);
    render_shape(&mut pixmap, shape, transform, Rgba::new(1.0, 1.0, 1.0, 0.0));
    pixmap
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::{SQUARE, TRIANGLE};

    #[test]
    fn test() {
        let mut shapes = Vec::new();
        for i in 0..6 {
            let a = SQUARE
                .scale(100.0, 100.0)
                .translate(-10.0, -10.0)
                .rotate(45.0)
                .skew(0.1, 0.1)
                .hue(180.0)
                .saturation(0.3)
                .lightness(0.8);
            let b = TRIANGLE
                .rotate(90.0)
                .scale(50.0, 50.0)
                .translate(-100.0, -20.0)
                .hue(270.0)
                .saturation(0.5)
                .lightness(0.5);
            let shape = Shape::Composite {
                shapes: vec![a, b].leak(),
                transform: Transform::from_translate(i as f32 * 5.0, i as f32 * 5.0),
                color: Hsla::new::<f32>((i as f32 * 60.0).into(), 0.2, 0.2 + i as f32 * 0.1, 1.0),
            };
            shapes.push(shape);
        }
        let shape = Shape::Composite {
            shapes: shapes.leak(),
            transform: IDENTITY,
            color: Hsla::new::<f32>(360.0.into(), 0.0, 1.0, 0.0),
        };
        render(shape, 400, 400);
    }
}
