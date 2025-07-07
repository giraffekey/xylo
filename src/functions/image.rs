#[cfg(feature = "std")]
use std::rc::Rc;

#[cfg(feature = "no-std")]
use alloc::{rc::Rc, vec::Vec};

#[cfg(feature = "io")]
use imageproc::geometric_transformations::Interpolation;

use crate::builtin_function;
use crate::error::{Error, Result};
use crate::functions::dedup_shape;
use crate::interpreter::{Data, Value};
use crate::shape::{ImageOp, ImagePath, Shape};

use core::cell::RefCell;
use rand_chacha::ChaCha8Rng;
use tiny_skia::FilterQuality;

#[cfg(not(feature = "io"))]
type Interpolation = ();

builtin_function!(import_image => {
    [Value::String(path)] => {
        Value::Shape(Rc::new(RefCell::new(Shape::image(ImagePath::File(path.clone())))))
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
    [Value::Integer(contrast), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Contrast(*contrast as f32));
        Value::Shape(image)
    },
    [Value::Float(contrast), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Contrast(*contrast));
        Value::Shape(image)
    },
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
    [Value::Integer(sigma), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Blur(*sigma as f32));
        Value::Shape(image)
    },
    [Value::Float(sigma), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Blur(*sigma));
        Value::Shape(image)
    },
});

builtin_function!(fast_blur => {
    [Value::Integer(sigma), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::FastBlur(*sigma as f32));
        Value::Shape(image)
    },
    [Value::Float(sigma), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::FastBlur(*sigma));
        Value::Shape(image)
    },
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
            Value::Integer(n) => Ok(*n as f32),
            Value::Float(n) => Ok(*n),
            _ => Err(Error::InvalidArgument("filter3x3".into())),
        }).collect::<Result<Vec<_>>>()?;

        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Filter3x3(kernel.try_into().unwrap()));
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
    [Value::Integer(sigma), Value::Integer(threshold), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Unsharpen(*sigma as f32, *threshold));
        Value::Shape(image)
    },
    [Value::Float(sigma), Value::Integer(threshold), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Unsharpen(*sigma, *threshold));
        Value::Shape(image)
    },
});

builtin_function!(adaptive_threshold => {
    [Value::Integer(block_radius), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::AdaptiveThreshold(*block_radius as u32));
        Value::Shape(image)
    }
});

builtin_function!(equalize_histogram => {
    [Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::EqualizeHistogram);
        Value::Shape(image)
    }
});

builtin_function!(match_histogram => {
    [Value::Shape(target), Value::Shape(image)] => {
        let target = dedup_shape(target);
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::MatchHistogram(target));
        Value::Shape(image)
    }
});

builtin_function!(stretch_contrast => {
    [Value::Integer(input_lower), Value::Integer(input_upper), Value::Integer(output_lower), Value::Integer(output_upper), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::StretchContrast(*input_lower as u8, *input_upper as u8, *output_lower as u8, *output_upper as u8));
        Value::Shape(image)
    }
});

builtin_function!(threshold => {
    [Value::Integer(t), Value::ThresholdType(t_type), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Threshold(*t as u8, *t_type));
        Value::Shape(image)
    }
});

builtin_function!(distance_transform => {
    [Value::Norm(norm), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::DistanceTransform(*norm));
        Value::Shape(image)
    }
});

builtin_function!(euclidean_squared_distance_transform => {
    [Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::EuclideanSquaredDistanceTransform);
        Value::Shape(image)
    }
});

builtin_function!(canny => {
    [Value::Integer(low_threshold), Value::Integer(high_threshold), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Canny(*low_threshold as f32, *high_threshold as f32));
        Value::Shape(image)
    },
    [Value::Float(low_threshold), Value::Float(high_threshold), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Canny(*low_threshold, *high_threshold));
        Value::Shape(image)
    },
    [Value::Integer(low_threshold), Value::Float(high_threshold), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Canny(*low_threshold as f32, *high_threshold));
        Value::Shape(image)
    },
    [Value::Float(low_threshold), Value::Integer(high_threshold), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Canny(*low_threshold, *high_threshold as f32));
        Value::Shape(image)
    },
});

builtin_function!(bilateral_filter => {
    [Value::Integer(window_size), Value::Integer(sigma_color), Value::Integer(sigma_spatial), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::BilateralFilter(*window_size as u32, *sigma_color as f32, *sigma_spatial as f32));
        Value::Shape(image)
    },
    [Value::Integer(window_size), Value::Float(sigma_color), Value::Float(sigma_spatial), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::BilateralFilter(*window_size as u32, *sigma_color, *sigma_spatial));
        Value::Shape(image)
    },
    [Value::Integer(window_size), Value::Integer(sigma_color), Value::Float(sigma_spatial), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::BilateralFilter(*window_size as u32, *sigma_color as f32, *sigma_spatial));
        Value::Shape(image)
    },
    [Value::Integer(window_size), Value::Float(sigma_color), Value::Integer(sigma_spatial), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::BilateralFilter(*window_size as u32, *sigma_color, *sigma_spatial as f32));
        Value::Shape(image)
    },
});

builtin_function!(box_filter => {
    [Value::Integer(x_radius), Value::Integer(y_radius), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::BoxFilter(*x_radius as u32, *y_radius as u32));
        Value::Shape(image)
    }
});

builtin_function!(gaussian_blur => {
    [Value::Integer(sigma), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::GaussianBlur(*sigma as f32));
        Value::Shape(image)
    },
    [Value::Float(sigma), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::GaussianBlur(*sigma));
        Value::Shape(image)
    },
});

builtin_function!(sharpen_gaussian => {
    [Value::Integer(sigma), Value::Integer(amount), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::SharpenGaussian(*sigma as f32, *amount as f32));
        Value::Shape(image)
    },
    [Value::Float(sigma), Value::Float(amount), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::SharpenGaussian(*sigma, *amount));
        Value::Shape(image)
    },
    [Value::Integer(sigma), Value::Float(amount), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::SharpenGaussian(*sigma as f32, *amount));
        Value::Shape(image)
    },
    [Value::Float(sigma), Value::Integer(amount), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::SharpenGaussian(*sigma, *amount as f32));
        Value::Shape(image)
    },
});

builtin_function!(horizontal_filter => {
    [Value::List(kernel), Value::Shape(image)] => {
        let kernel = kernel.iter().map(|value| match value {
            Value::Integer(n) => Ok(*n as f32),
            Value::Float(n) => Ok(*n),
            _ => Err(Error::InvalidArgument("horizontal_filter".into())),
        }).collect::<Result<Vec<_>>>()?;

        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::HorizontalFilter(kernel));
        Value::Shape(image)
    },
});

builtin_function!(vertical_filter => {
    [Value::List(kernel), Value::Shape(image)] => {
        let kernel = kernel.iter().map(|value| match value {
            Value::Integer(n) => Ok(*n as f32),
            Value::Float(n) => Ok(*n),
            _ => Err(Error::InvalidArgument("vertical_filter".into())),
        }).collect::<Result<Vec<_>>>()?;

        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::VerticalFilter(kernel));
        Value::Shape(image)
    },
});

builtin_function!(laplacian_filter => {
    [Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::LaplacianFilter);
        Value::Shape(image)
    }
});

builtin_function!(median_filter => {
    [Value::Integer(x_radius), Value::Integer(y_radius), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::MedianFilter(*x_radius as u32, *y_radius as u32));
        Value::Shape(image)
    }
});

builtin_function!(separable_filter => {
    [Value::List(h_kernel), Value::List(v_kernel), Value::Shape(image)] => {
        let h_kernel = h_kernel.iter().map(|value| match value {
            Value::Integer(n) => Ok(*n as f32),
            Value::Float(n) => Ok(*n),
            _ => Err(Error::InvalidArgument("separable_filter".into())),
        }).collect::<Result<Vec<_>>>()?;

        let v_kernel = v_kernel.iter().map(|value| match value {
            Value::Integer(n) => Ok(*n as f32),
            Value::Float(n) => Ok(*n),
            _ => Err(Error::InvalidArgument("separable_filter".into())),
        }).collect::<Result<Vec<_>>>()?;

        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::SeparableFilter(h_kernel, v_kernel));
        Value::Shape(image)
    },
});

builtin_function!(separable_filter_equal => {
    [Value::List(kernel), Value::Shape(image)] => {
        let kernel = kernel.iter().map(|value| match value {
            Value::Integer(n) => Ok(*n as f32),
            Value::Float(n) => Ok(*n),
            _ => Err(Error::InvalidArgument("separable_filter_equal".into())),
        }).collect::<Result<Vec<_>>>()?;

        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::SeparableFilterEqual(kernel));
        Value::Shape(image)
    },
});

builtin_function!(sharpen3x3 => {
    [Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Sharpen3x3);
        Value::Shape(image)
    }
});

builtin_function!(translate_image => {
    [Value::Integer(tx), Value::Integer(ty), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Translate(*tx, *ty));
        Value::Shape(image)
    }
});

builtin_function!(rotate_image => {
    [cx, cy, angle, Value::FilterQuality(quality), Value::Shape(image)] => {
        let cx = match cx {
            Value::Integer(cx) => *cx as f32,
            Value::Float(cx)   => *cx,
            _ => return Err(Error::InvalidArgument("rotate_image".into())),
        };

        let cy = match cy {
            Value::Integer(cy) => *cy as f32,
            Value::Float(cy)   => *cy,
            _ => return Err(Error::InvalidArgument("rotate_image".into())),
        };

        let angle = match angle {
            Value::Integer(angle) => *angle as f32,
            Value::Float(angle)   => *angle,
            _ => return Err(Error::InvalidArgument("rotate_image".into())),
        };

        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Rotate(cx, cy, angle, quality_to_interpolation(quality)));
        Value::Shape(image)
    }
});

builtin_function!(rotate_image_about_center => {
    [Value::Integer(angle), Value::FilterQuality(quality), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::RotateAboutCenter(*angle as f32, quality_to_interpolation(quality)));
        Value::Shape(image)
    },
    [Value::Float(angle), Value::FilterQuality(quality), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::RotateAboutCenter(*angle, quality_to_interpolation(quality)));
        Value::Shape(image)
    },
});

builtin_function!(warp_image => {
    [Value::List(transform), Value::FilterQuality(quality), Value::Shape(image)] => {
        if transform.len() != 9 {
            return Err(Error::InvalidList);
        }

        let transform = transform.iter().map(|value| match value {
            Value::Integer(n) => Ok(*n as f32),
            Value::Float(n) => Ok(*n),
            _ => Err(Error::InvalidArgument("warp".into())),
        }).collect::<Result<Vec<_>>>()?;

        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Warp(transform.try_into().unwrap(), quality_to_interpolation(quality)));
        Value::Shape(image)
    },
});

#[cfg(feature = "io")]
fn quality_to_interpolation(quality: &FilterQuality) -> Interpolation {
    match quality {
        FilterQuality::Nearest => Interpolation::Nearest,
        FilterQuality::Bilinear => Interpolation::Bilinear,
        FilterQuality::Bicubic => Interpolation::Bicubic,
    }
}

#[cfg(not(feature = "io"))]
fn quality_to_interpolation(_quality: &FilterQuality) {}

builtin_function!(horizontal_prewitt => {
    [Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::HorizontalPrewitt);
        Value::Shape(image)
    }
});

builtin_function!(horizontal_scharr => {
    [Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::HorizontalScharr);
        Value::Shape(image)
    }
});

builtin_function!(horizontal_sobel => {
    [Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::HorizontalSobel);
        Value::Shape(image)
    }
});

builtin_function!(vertical_prewitt => {
    [Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::VerticalPrewitt);
        Value::Shape(image)
    }
});

builtin_function!(vertical_scharr => {
    [Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::VerticalScharr);
        Value::Shape(image)
    }
});

builtin_function!(vertical_sobel => {
    [Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::VerticalSobel);
        Value::Shape(image)
    }
});

builtin_function!(prewitt_gradients => {
    [Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::PrewittGradients);
        Value::Shape(image)
    }
});

builtin_function!(sobel_gradients => {
    [Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::SobelGradients);
        Value::Shape(image)
    }
});

builtin_function!(integral_image => {
    [Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::IntegralImage);
        Value::Shape(image)
    }
});

builtin_function!(integral_squared_image => {
    [Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::IntegralSquaredImage);
        Value::Shape(image)
    }
});

builtin_function!(red_channel => {
    [Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::RedChannel);
        Value::Shape(image)
    }
});

builtin_function!(green_channel => {
    [Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::GreenChannel);
        Value::Shape(image)
    }
});

builtin_function!(blue_channel => {
    [Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::BlueChannel);
        Value::Shape(image)
    }
});

builtin_function!(image_close => {
    [Value::Norm(norm), Value::Integer(k), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Close(*norm, *k as u8));
        Value::Shape(image)
    }
});

builtin_function!(image_dilate => {
    [Value::Norm(norm), Value::Integer(k), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Dilate(*norm, *k as u8));
        Value::Shape(image)
    }
});

builtin_function!(image_erode => {
    [Value::Norm(norm), Value::Integer(k), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Erode(*norm, *k as u8));
        Value::Shape(image)
    }
});

builtin_function!(image_open => {
    [Value::Norm(norm), Value::Integer(k), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::Open(*norm, *k as u8));
        Value::Shape(image)
    }
});

builtin_function!(gaussian_noise => {
    [Value::Integer(mean), Value::Integer(stddev), Value::Integer(seed), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::GaussianNoise(*mean as f64, *stddev as f64, *seed as u64));
        Value::Shape(image)
    },
    [Value::Float(mean), Value::Float(stddev), Value::Integer(seed), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::GaussianNoise(*mean as f64, *stddev as f64, *seed as u64));
        Value::Shape(image)
    },
    [Value::Integer(mean), Value::Float(stddev), Value::Integer(seed), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::GaussianNoise(*mean as f64, *stddev as f64, *seed as u64));
        Value::Shape(image)
    },
    [Value::Float(mean), Value::Integer(stddev), Value::Integer(seed), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::GaussianNoise(*mean as f64, *stddev as f64, *seed as u64));
        Value::Shape(image)
    },
});

builtin_function!(salt_and_pepper_noise => {
    [Value::Integer(rate), Value::Integer(seed), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::SaltAndPepperNoise(*rate as f64, *seed as u64));
        Value::Shape(image)
    },
    [Value::Float(rate), Value::Integer(seed), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::SaltAndPepperNoise(*rate as f64, *seed as u64));
        Value::Shape(image)
    },
});

builtin_function!(suppress_non_maximum => {
    [Value::Integer(radius), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::SuppressNonMaximum(*radius as u32));
        Value::Shape(image)
    },
});

builtin_function!(pixel_sort => {
    [Value::SortMode(mode), Value::SortDirection(direction), Value::Shape(image)] => {
        let image = dedup_shape(image);
        image.borrow_mut().add_image_op(ImageOp::PixelSort(mode.clone(), direction.clone()));
        Value::Shape(image)
    }
});

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "std")]
    use std::rc::Rc;

    #[cfg(feature = "no-std")]
    use alloc::{rc::Rc, vec};

    #[cfg(feature = "io")]
    use image::imageops::FilterType;

    #[cfg(not(feature = "io"))]
    use crate::parser::FilterType;

    use crate::parser::{SortDirection, SortMode};
    use crate::shape::ImagePath;
    use core::cell::RefCell;
    use rand::SeedableRng;
    use tiny_skia::FilterQuality;

    #[test]
    fn test_image_import_and_text() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();

        // Test image import
        let image_result =
            import_image(&mut rng, &data, &[Value::String("test.png".into())]).unwrap();

        if let Value::Shape(shape) = image_result {
            let shape = shape.borrow();
            if let Shape::Image { path, .. } = &*shape {
                assert!(matches!(path, ImagePath::File(_)));
            } else {
                panic!("Expected Image shape");
            }
        } else {
            panic!("Expected Shape value");
        }

        // Test text creation
        let text_result = text(
            &mut rng,
            &data,
            &[
                Value::String("Hello".into()),
                Value::String("Arial".into()),
                Value::Integer(24),
            ],
        )
        .unwrap();
        assert!(matches!(text_result, Value::Shape(_)));
    }

    #[test]
    fn test_image_quality() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        let image = Rc::new(RefCell::new(Shape::image(ImagePath::File(
            "test.png".into(),
        ))));

        let result = image_quality(
            &mut rng,
            &data,
            &[
                Value::FilterQuality(FilterQuality::Nearest),
                Value::Shape(image.clone()),
            ],
        )
        .unwrap();
        assert!(matches!(result, Value::Shape(_)));
    }

    #[test]
    fn test_image_operations() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        let image = Rc::new(RefCell::new(Shape::image(ImagePath::File(
            "test.png".into(),
        ))));

        // Test brightness adjustment
        let bright_result = brighten(
            &mut rng,
            &data,
            &[Value::Integer(10), Value::Shape(image.clone())],
        )
        .unwrap();
        assert!(matches!(bright_result, Value::Shape(_)));

        // Test contrast adjustment
        let contrast_result = contrast(
            &mut rng,
            &data,
            &[Value::Float(1.5), Value::Shape(image.clone())],
        )
        .unwrap();
        assert!(matches!(contrast_result, Value::Shape(_)));

        // Test grayscale conversion
        let gray_result = grayscale(&mut rng, &data, &[Value::Shape(image.clone())]).unwrap();
        assert!(matches!(gray_result, Value::Shape(_)));

        // Test blur
        let blur_result = blur(
            &mut rng,
            &data,
            &[Value::Float(2.0), Value::Shape(image.clone())],
        )
        .unwrap();
        assert!(matches!(blur_result, Value::Shape(_)));

        // Test crop
        let crop_result = crop(
            &mut rng,
            &data,
            &[
                Value::Integer(10),
                Value::Integer(10),
                Value::Integer(100),
                Value::Integer(100),
                Value::Shape(image.clone()),
            ],
        )
        .unwrap();
        assert!(matches!(crop_result, Value::Shape(_)));
    }

    #[test]
    fn test_image_filters() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        let image = Rc::new(RefCell::new(Shape::image(ImagePath::File(
            "test.png".into(),
        ))));

        // Test 3x3 filter
        let kernel = vec![
            Value::Float(0.0),
            Value::Float(-1.0),
            Value::Float(0.0),
            Value::Float(-1.0),
            Value::Float(5.0),
            Value::Float(-1.0),
            Value::Float(0.0),
            Value::Float(-1.0),
            Value::Float(0.0),
        ];
        let filter_result = filter3x3(
            &mut rng,
            &data,
            &[Value::List(kernel), Value::Shape(image.clone())],
        )
        .unwrap();
        assert!(matches!(filter_result, Value::Shape(_)));

        // Test unsharp mask
        let unsharp_result = unsharpen(
            &mut rng,
            &data,
            &[
                Value::Float(1.0),
                Value::Integer(5),
                Value::Shape(image.clone()),
            ],
        )
        .unwrap();
        assert!(matches!(unsharp_result, Value::Shape(_)));
    }

    #[test]
    fn test_image_transforms() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        let image = Rc::new(RefCell::new(Shape::image(ImagePath::File(
            "test.png".into(),
        ))));

        // Test flips
        let fliph_result = fliph_image(&mut rng, &data, &[Value::Shape(image.clone())]).unwrap();
        assert!(matches!(fliph_result, Value::Shape(_)));

        let flipv_result = flipv_image(&mut rng, &data, &[Value::Shape(image.clone())]).unwrap();
        assert!(matches!(flipv_result, Value::Shape(_)));

        // Test rotations
        let rotate90_result = rotate90(&mut rng, &data, &[Value::Shape(image.clone())]).unwrap();
        assert!(matches!(rotate90_result, Value::Shape(_)));

        // Test resize
        let resize_result = resize(
            &mut rng,
            &data,
            &[
                Value::Integer(200),
                Value::Integer(200),
                Value::FilterType(FilterType::Lanczos3),
                Value::Shape(image.clone()),
            ],
        )
        .unwrap();
        assert!(matches!(resize_result, Value::Shape(_)));
    }

    #[test]
    fn test_image_compositing() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        let image1 = Rc::new(RefCell::new(Shape::image(ImagePath::File(
            "test1.png".into(),
        ))));
        let image2 = Rc::new(RefCell::new(Shape::image(ImagePath::File(
            "test2.png".into(),
        ))));

        // Test overlay
        let overlay_result = overlay(
            &mut rng,
            &data,
            &[
                Value::Integer(10),
                Value::Integer(10),
                Value::Shape(image1.clone()),
                Value::Shape(image2.clone()),
            ],
        )
        .unwrap();
        assert!(matches!(overlay_result, Value::Shape(_)));

        // Test tile
        let tile_result = tile(
            &mut rng,
            &data,
            &[Value::Shape(image1.clone()), Value::Shape(image2.clone())],
        )
        .unwrap();
        assert!(matches!(tile_result, Value::Shape(_)));
    }

    #[test]
    fn test_image_gradients() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        let image = Rc::new(RefCell::new(Shape::image(ImagePath::File(
            "test.png".into(),
        ))));

        // Test horizontal gradient
        let gradh_result = gradienth(
            &mut rng,
            &data,
            &[
                Value::Hex([255, 0, 0]),
                Value::Float(1.0),
                Value::Hex([0, 0, 255]),
                Value::Float(0.5),
                Value::Shape(image.clone()),
            ],
        )
        .unwrap();
        assert!(matches!(gradh_result, Value::Shape(_)));

        // Test vertical gradient
        let gradv_result = gradientv(
            &mut rng,
            &data,
            &[
                Value::Hex([255, 0, 0]),
                Value::Float(1.0),
                Value::Hex([0, 0, 255]),
                Value::Float(0.5),
                Value::Shape(image.clone()),
            ],
        )
        .unwrap();
        assert!(matches!(gradv_result, Value::Shape(_)));
    }

    #[test]
    fn test_pixel_sorting() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        let image = Rc::new(RefCell::new(Shape::image(ImagePath::File(
            "test.png".into(),
        ))));

        // Test pixel sort
        let sort_result = pixel_sort(
            &mut rng,
            &data,
            &[
                Value::SortMode(SortMode::Brightness(0)),
                Value::SortDirection(SortDirection::Column),
                Value::Shape(image.clone()),
            ],
        )
        .unwrap();
        assert!(matches!(sort_result, Value::Shape(_)));
    }

    #[test]
    fn test_invalid_inputs() {
        let mut rng = ChaCha8Rng::from_seed([0; 32]);
        let data = Data::default();
        let image = Rc::new(RefCell::new(Shape::image(ImagePath::File(
            "test.png".into(),
        ))));

        // Test text with invalid size
        assert!(text(
            &mut rng,
            &data,
            &[
                Value::String("Hello".into()),
                Value::String("Arial".into()),
                Value::String("24".into()) // invalid size
            ]
        )
        .is_err());

        // Test filter3x3 with wrong kernel size
        let bad_kernel = vec![Value::Float(1.0); 4]; // needs 9 elements
        assert!(filter3x3(
            &mut rng,
            &data,
            &[Value::List(bad_kernel), Value::Shape(image.clone())]
        )
        .is_err());

        // Test crop with negative values
        assert!(crop(
            &mut rng,
            &data,
            &[
                Value::Integer(-10), // invalid
                Value::Integer(10),
                Value::Integer(100),
                Value::Integer(100),
                Value::Shape(image.clone())
            ]
        )
        .is_err());
    }
}
