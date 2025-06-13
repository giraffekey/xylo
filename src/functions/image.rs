#[cfg(feature = "std")]
use std::rc::Rc;

#[cfg(feature = "no-std")]
use alloc::{rc::Rc, vec::Vec};

use crate::builtin_function;
use crate::error::{Error, Result};
use crate::functions::dedup_shape;
use crate::interpreter::{Data, Value};
use crate::shape::{ImageOp, ImagePath, Shape};

use core::cell::RefCell;
use rand_chacha::ChaCha8Rng;

builtin_function!(import_image => {
    [Value::String(path)] => {
        Value::Shape(Rc::new(RefCell::new(Shape::image(ImagePath::File(path.clone())))))
    }
});

builtin_function!(import_font => {
    [Value::String(path)] => {
        todo!()
    }
});

builtin_function!(text => {
    [Value::String(text), Value::String(font), size] => {
        let size = match size {
            Value::Integer(size) => *size as f32,
            Value::Float(size)   => *size,
            _ => return Err(Error::InvalidArgument("text".into())),
        };

        Value::Shape(Rc::new(RefCell::new(Shape::text(font.clone(), text.clone(), size))))
    }
});

builtin_function!(image_quality => {
    [Value::FilterQuality(quality), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().set_image_quality(*quality);
        Value::Shape(image)
    }
});

builtin_function!(brighten => {
    [Value::Integer(value), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Brighten(*value));
        Value::Shape(image)
    }
});

builtin_function!(contrast => {
    [contrast, Value::Shape(image)] => {
        let contrast = match contrast {
            Value::Integer(contrast) => *contrast as f32,
            Value::Float(contrast)   => *contrast,
            _ => return Err(Error::InvalidArgument("contrast".into())),
        };

        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Contrast(contrast));
        Value::Shape(image)
    }
});

builtin_function!(grayscale => {
    [Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Grayscale);
        Value::Shape(image)
    }
});

builtin_function!(grayscale_alpha => {
    [Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::GrayscaleAlpha);
        Value::Shape(image)
    }
});

builtin_function!(huerotate => {
    [Value::Integer(value), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Huerotate(*value));
        Value::Shape(image)
    }
});

builtin_function!(invert => {
    [Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Invert);
        Value::Shape(image)
    }
});

builtin_function!(blur => {
    [sigma, Value::Shape(image)] => {
        let sigma = match sigma {
            Value::Integer(sigma) => *sigma as f32,
            Value::Float(sigma)   => *sigma,
            _ => return Err(Error::InvalidArgument("blur".into())),
        };

        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Blur(sigma));
        Value::Shape(image)
    }
});

builtin_function!(fast_blur => {
    [sigma, Value::Shape(image)] => {
        let sigma = match sigma {
            Value::Integer(sigma) => *sigma as f32,
            Value::Float(sigma)   => *sigma,
            _ => return Err(Error::InvalidArgument("fast_blur".into())),
        };

        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::FastBlur(sigma));
        Value::Shape(image)
    }
});

builtin_function!(crop => {
    [Value::Integer(x), Value::Integer(y), Value::Integer(width), Value::Integer(height), Value::Shape(image)] => {
        if *x < 0 || *y < 0 || *width < 0 || *height < 0 {
            return Err(Error::NegativeNumber);
        }

        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Crop(*x as u32, *y as u32, *width as u32, *height as u32));
        Value::Shape(image)
    }
});

builtin_function!(filter3x3 => {
    [Value::List(kernel), Value::Shape(image)] => {
        if kernel.len() != 9 {
            return Err(Error::InvalidList);
        }

        let kernel = kernel.iter().map(|value| match value {
            Value::Float(n) => Ok(*n),
            _ => Err(Error::InvalidArgument("filter3x3".into())),
        }).collect::<Result<Vec<_>>>()?;

        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Filter3x3(kernel));
        Value::Shape(image)
    }
});

builtin_function!(fliph_image => {
    [Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::FlipHorizontal);
        Value::Shape(image)
    }
});

builtin_function!(flipv_image => {
    [Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::FlipVertical);
        Value::Shape(image)
    }
});

builtin_function!(flipd_image => {
    [Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::FlipHorizontal);
        image.borrow_mut().add_image_op(ImageOp::FlipVertical);
        Value::Shape(image)
    }
});

builtin_function!(gradienth => {
    [Value::Hex(start_color), start_alpha, Value::Hex(end_color), end_alpha, Value::Shape(image)] => {
        let start_alpha = match start_alpha {
            Value::Integer(start_alpha) => *start_alpha as f32,
            Value::Float(start_alpha)   => *start_alpha,
            _ => return Err(Error::InvalidArgument("gradienth".into())),
        };

        let end_alpha = match end_alpha {
            Value::Integer(end_alpha) => *end_alpha as f32,
            Value::Float(end_alpha)   => *end_alpha,
            _ => return Err(Error::InvalidArgument("gradienth".into())),
        };

        let start = [start_color[0], start_color[1], start_color[2], (start_alpha * 255.0).clamp(0.0, 255.0) as u8];
        let end = [end_color[0], end_color[1], end_color[2], (end_alpha * 255.0).clamp(0.0, 255.0) as u8];

        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::HorizontalGradient(start, end));
        Value::Shape(image)
    }
});

builtin_function!(gradientv => {
    [Value::Hex(start_color), start_alpha, Value::Hex(end_color), end_alpha, Value::Shape(image)] => {
        let start_alpha = match start_alpha {
            Value::Integer(start_alpha) => *start_alpha as f32,
            Value::Float(start_alpha)   => *start_alpha,
            _ => return Err(Error::InvalidArgument("gradientv".into())),
        };

        let end_alpha = match end_alpha {
            Value::Integer(end_alpha) => *end_alpha as f32,
            Value::Float(end_alpha)   => *end_alpha,
            _ => return Err(Error::InvalidArgument("gradientv".into())),
        };

        let start = [start_color[0], start_color[1], start_color[2], (start_alpha * 255.0).clamp(0.0, 255.0) as u8];
        let end = [end_color[0], end_color[1], end_color[2], (end_alpha * 255.0).clamp(0.0, 255.0) as u8];

        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::VerticalGradient(start, end));
        Value::Shape(image)
    }
});

builtin_function!(overlay => {
    [Value::Integer(x), Value::Integer(y), Value::Shape(top), Value::Shape(bottom)] => {
        let top = dedup_shape(top);
        let bottom = dedup_shape(bottom);
        bottom.borrow_mut().add_image_op(ImageOp::Overlay(top, *x as i64, *y as i64));
        Value::Shape(bottom)
    }
});

builtin_function!(replace => {
    [Value::Integer(x), Value::Integer(y), Value::Shape(top), Value::Shape(bottom)] => {
        let top = dedup_shape(top);
        let bottom = dedup_shape(bottom);
        bottom.borrow_mut().add_image_op(ImageOp::Replace(top, *x as i64, *y as i64));
        Value::Shape(bottom)
    }
});

builtin_function!(resize => {
    [Value::Integer(width), Value::Integer(height), Value::FilterType(filter), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Resize(*width as u32, *height as u32, *filter));
        Value::Shape(image)
    }
});

builtin_function!(rotate90 => {
    [Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Rotate90);
        Value::Shape(image)
    }
});

builtin_function!(rotate180 => {
    [Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Rotate180);
        Value::Shape(image)
    }
});

builtin_function!(rotate270 => {
    [Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Rotate270);
        Value::Shape(image)
    }
});

builtin_function!(thumbnail => {
    [Value::Integer(width), Value::Integer(height), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Thumbnail(*width as u32, *height as u32));
        Value::Shape(image)
    }
});

builtin_function!(tile => {
    [Value::Shape(top), Value::Shape(bottom)] => {
        let top = dedup_shape(top);
        let bottom = dedup_shape(bottom);
        bottom.borrow_mut().add_image_op(ImageOp::Tile(top));
        Value::Shape(bottom)
    }
});

builtin_function!(unsharpen => {
    [sigma, Value::Integer(threshold), Value::Shape(image)] => {
        let sigma = match sigma {
            Value::Integer(sigma) => *sigma as f32,
            Value::Float(sigma)   => *sigma,
            _ => return Err(Error::InvalidArgument("unsharpen".into())),
        };

        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Unsharpen(sigma, *threshold));
        Value::Shape(image)
    }
});

builtin_function!(pixel_sort => {
    [Value::SortMode(mode), Value::SortDirection(direction), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::PixelSort(mode.clone(), direction.clone()));
        Value::Shape(image)
    }
});
