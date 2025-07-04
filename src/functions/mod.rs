#[cfg(feature = "std")]
use std::rc::Rc;

#[cfg(feature = "no-std")]
use alloc::rc::Rc;

use crate::error::{Error, Result};
use crate::interpreter::{Data, Value};
use crate::shape::Shape;

use core::cell::RefCell;
use rand_chacha::ChaCha8Rng;

mod character;
mod color;
mod compare;
mod func;
mod image;
mod list;
mod math;
mod path;
mod rand;
mod shape;
mod system;
mod transform;

macro_rules! define_builtins {
    (
        $(
            $name:literal => {$func:path, $param_count:literal}
        ),* $(,)?
    ) => {
        // Generate the static BUILTIN_FUNCTIONS array
        pub static BUILTIN_FUNCTIONS: &[&str] = &[
            "map",
            $($name),*
        ];

        // Generate the handle_builtin function with match statements
        pub fn handle_builtin(name: &str, rng: &mut ChaCha8Rng, data: &Data, args: &[Value]) -> Result<Value> {
            match name {
                $(
                    $name => $func(rng, data, args),
                )*
                _ => Err(Error::UnknownFunction(name.into())),
            }
        }

        // Generate the builtin_param_count function with match statements
        pub fn builtin_param_count(name: &str) -> usize {
            match name {
                "map" => 2,
                $(
                    $name => $param_count,
                )*
                _ => unreachable!(),
            }
        }
    };
}

define_builtins! {
    "width" => {system::width, 0},
    "height" => {system::height, 0},
    "neg" => {math::neg, 1},
    "!" => {compare::not, 1},
    "not" => {compare::not, 1},
    "~" => {math::bitnot, 1},
    "bitnot" => {math::bitnot, 1},
    "+" => {math::add, 2},
    "add" => {math::add, 2},
    "-" => {math::sub, 2},
    "sub" => {math::sub, 2},
    "*" => {math::mul, 2},
    "mul" => {math::mul, 2},
    "/" => {math::div, 2},
    "div" => {math::div, 2},
    "%" => {math::modulo, 2},
    "mod" => {math::modulo, 2},
    "**" => {math::pow, 2},
    "pow" => {math::pow, 2},
    "&" => {math::bitand, 2},
    "bitand" => {math::bitand, 2},
    "|" => {math::bitor, 2},
    "bitor" => {math::bitor, 2},
    "^" => {math::bitxor, 2},
    "bitxor" => {math::bitxor, 2},
    "<<" => {math::bitleft, 2},
    "bitleft" => {math::bitleft, 2},
    ">>" => {math::bitright, 2},
    "bitright" => {math::bitright, 2},
    "pi" => {math::pi, 0},
    "π" => {math::pi, 0},
    "tau" => {math::tau, 0},
    "τ" => {math::tau, 0},
    "e" => {math::e, 0},
    "ℯ" => {math::e, 0},
    "phi" => {math::phi, 0},
    "φ" => {math::phi, 0},
    "int" => {math::int, 1},
    "float" => {math::float, 1},
    "complex" => {math::complex, 2},
    "real" => {math::real, 1},
    "imag" => {math::imag, 1},
    "deg_to_rad" => {math::deg_to_rad, 1},
    "rad_to_deg" => {math::rad_to_deg, 1},
    "sin" => {math::sin, 1},
    "cos" => {math::cos, 1},
    "tan" => {math::tan, 1},
    "asin" => {math::asin, 1},
    "acos" => {math::acos, 1},
    "atan" => {math::atan, 1},
    "atan2" => {math::atan2, 2},
    "sinh" => {math::sinh, 1},
    "cosh" => {math::cosh, 1},
    "tanh" => {math::tanh, 1},
    "asinh" => {math::asinh, 1},
    "acosh" => {math::acosh, 1},
    "atanh" => {math::atanh, 1},
    "ln" => {math::ln, 1},
    "log10" => {math::log10, 1},
    "log" => {math::log, 2},
    "floor" => {math::floor, 1},
    "ceil" => {math::ceil, 1},
    "abs" => {math::abs, 1},
    "sqrt" => {math::sqrt, 1},
    "cbrt" => {math::cbrt, 1},
    "fact" => {math::fact, 1},
    "fact2" => {math::fact2, 1},
    "min" => {math::min, 2},
    "max" => {math::max, 2},
    "==" => {compare::eq, 2},
    "eq" => {compare::eq, 2},
    "!=" => {compare::neq, 2},
    "neq" => {compare::neq, 2},
    "<" => {compare::lt, 2},
    "lt" => {compare::lt, 2},
    "<=" => {compare::lte, 2},
    "lte" => {compare::lte, 2},
    ">" => {compare::gt, 2},
    "gt" => {compare::gt, 2},
    ">=" => {compare::gte, 2},
    "gte" => {compare::gte, 2},
    "&&" => {compare::and, 2},
    "and" => {compare::and, 2},
    "||" => {compare::or, 2},
    "or" => {compare::or, 2},
    ".." => {list::range, 2},
    "range" => {list::range, 2},
    "..=" => {list::rangei, 2},
    "rangei" => {list::rangei, 2},
    "++" => {list::concat, 2},
    "concat" => {list::concat, 2},
    "+>" => {list::prepend, 2},
    "prepend" => {list::prepend, 2},
    "<+" => {list::append, 2},
    "append" => {list::append, 2},
    "nth" => {list::nth, 2},
    "set" => {list::set, 3},
    "length" => {list::length, 1},
    "is_empty" => {list::is_empty, 1},
    "head" => {list::head, 1},
    "tail" => {list::tail, 1},
    "init" => {list::init, 1},
    "last" => {list::last, 1},
    "contains" => {list::contains, 2},
    "take" => {list::take, 2},
    "drop" => {list::drop, 2},
    "index_of" => {list::index_of, 2},
    "reverse" => {list::reverse, 1},
    "slice" => {list::slice, 3},
    "split" => {list::split, 2},
    "unique" => {list::unique, 1},
    "min_of" => {list::min_of, 1},
    "max_of" => {list::max_of, 1},
    "sum" => {list::sum, 1},
    "product" => {list::product, 1},
    "sort" => {list::sort, 1},
    "flatten" => {list::flatten, 1},
    "join" => {list::join, 2},
    "intercalate" => {list::intercalate, 2},
    "intersperse" => {list::intersperse, 2},
    "is_control" => {character::is_control, 1},
    "is_uppercase" => {character::is_uppercase, 1},
    "is_lowercase" => {character::is_lowercase, 1},
    "is_alpha" => {character::is_alpha, 1},
    "is_alphanumeric" => {character::is_alphanumeric, 1},
    "is_digit" => {character::is_digit, 1},
    "is_hex_digit" => {character::is_hex_digit, 1},
    "is_numeric" => {character::is_numeric, 1},
    "is_punctuation" => {character::is_punctuation, 1},
    "is_whitespace" => {character::is_whitespace, 1},
    "is_ascii" => {character::is_ascii, 1},
    "to_uppercase" => {character::to_uppercase, 1},
    "to_lowercase" => {character::to_lowercase, 1},
    "digit_to_int" => {character::digit_to_int, 1},
    "int_to_digit" => {character::int_to_digit, 1},
    "words" => {character::words, 1},
    "lines" => {character::lines, 1},
    "string" => {character::string, 1},
    "|>" => {func::pipe, 2},
    "pipe" => {func::pipe, 2},
    // "." => compose_fn 2,
    // "compose_fn" => compose_fn 2,
    "rand" => {rand::rand, 0},
    "randi" => {rand::randi, 0},
    "rand_range" => {rand::rand_range, 2},
    "randi_range" => {rand::randi_range, 2},
    "rand_rangei" => {rand::rand_rangei, 2},
    "randi_rangei" => {rand::randi_rangei, 2},
    "shuffle" => {rand::shuffle, 1},
    "choose" => {rand::choose, 1},
    "noise1" => {rand::noise1, 1},
    "noise2" => {rand::noise2, 2},
    "noise3" => {rand::noise3, 3},
    "noise4" => {rand::noise4, 4},
    ":" => {shape::compose, 2},
    "compose" => {shape::compose, 2},
    "collect" => {shape::collect, 1},
    "blend" => {shape::blend, 2},
    "anti_alias" => {shape::anti_alias, 2},
    "fill" => {shape::fill, 1},
    "winding" => {shape::winding, 1},
    "even_odd" => {shape::even_odd, 1},
    "stroke" => {shape::stroke, 2},
    "miter_limit" => {shape::miter_limit, 2},
    "line_cap" => {shape::line_cap, 2},
    "line_join" => {shape::line_join, 2},
    "dash" => {shape::dash, 3},
    "no_dash" => {shape::no_dash, 1},
    "mask" => {shape::mask, 2},
    "pattern" => {shape::pattern, 3},
    "voronoi" => {shape::voronoi, 2},
    "t" => {transform::translate, 3},
    "translate" => {transform::translate, 3},
    "tx" => {transform::translatex, 2},
    "translatex" => {transform::translatex, 2},
    "ty" => {transform::translatey, 2},
    "translatey" => {transform::translatey, 2},
    "tt" => {transform::translateb, 2},
    "translateb" => {transform::translateb, 2},
    "r" => {transform::rotate, 2},
    "rotate" => {transform::rotate, 2},
    "ra" => {transform::rotate_at, 4},
    "rotate_at" => {transform::rotate_at, 4},
    "s" => {transform::scale, 3},
    "scale" => {transform::scale, 3},
    "sx" => {transform::scalex, 2},
    "scalex" => {transform::scalex, 2},
    "sy" => {transform::scaley, 2},
    "scaley" => {transform::scaley, 2},
    "ss" => {transform::scaleb, 2},
    "scaleb" => {transform::scaleb, 2},
    "k" => {transform::skew, 3},
    "skew" => {transform::skew, 3},
    "kx" => {transform::skewx, 2},
    "skewx" => {transform::skewx, 2},
    "ky" => {transform::skewy, 2},
    "skewy" => {transform::skewy, 2},
    "kk" => {transform::skewb, 2},
    "skewb" => {transform::skewb, 2},
    "f" => {transform::flip, 2},
    "flip" => {transform::flip, 2},
    "fh" => {transform::fliph, 1},
    "fliph" => {transform::fliph, 1},
    "fv" => {transform::flipv, 1},
    "flipv" => {transform::flipv, 1},
    "fd" => {transform::flipd, 1},
    "flipd" => {transform::flipd, 1},
    "z" => {transform::zindex, 2},
    "zindex" => {transform::zindex, 2},
    "zshift" => {transform::zshift, 2},
    "hsl" => {color::hsl, 4},
    "hsla" => {color::hsla, 5},
    "h" => {color::hue, 2},
    "hue" => {color::hue, 2},
    "sat" => {color::saturation, 2},
    "saturation" => {color::saturation, 2},
    "l" => {color::lightness, 2},
    "lightness" => {color::lightness, 2},
    "a" => {color::alpha, 2},
    "alpha" => {color::alpha, 2},
    "hshift" => {color::hshift, 2},
    "satshift" => {color::satshift, 2},
    "lshift" => {color::lshift, 2},
    "ashift" => {color::ashift, 2},
    "hex" => {color::hex, 2},
    "solid" => {color::solid, 1},
    "g" => {color::gradient, 2},
    "gradient" => {color::gradient, 2},
    "linear_grad" => {color::linear_grad, 4},
    "radial_grad" => {color::radial_grad, 5},
    "grad_start" => {color::grad_start, 3},
    "grad_end" => {color::grad_end, 3},
    "to_linear_grad" => {color::to_linear_grad, 1},
    "grad_radius" => {color::grad_radius, 2},
    "grad_stop_hsl" => {color::grad_stop_hsl, 5},
    "grad_stop_hsla" => {color::grad_stop_hsla, 6},
    "grad_stop_hex" => {color::grad_stop_hex, 3},
    "grad_spread_mode" => {color::grad_spread_mode, 2},
    "move_to" => {path::move_to, 2},
    "line_to" => {path::line_to, 2},
    "quad_to" => {path::quad_to, 4},
    "cubic_to" => {path::cubic_to, 6},
    "close" => {path::close, 0},
    "import_image" => {image::import_image, 1},
    "text" => {image::text, 3},
    "image_quality" => {image::image_quality, 2},
    "brighten" => {image::brighten, 2},
    "contrast" => {image::contrast, 2},
    "grayscale" => {image::grayscale, 1},
    "grayscale_alpha" => {image::grayscale_alpha, 1},
    "huerotate" => {image::huerotate, 2},
    "invert" => {image::invert, 1},
    "blur" => {image::blur, 2},
    "fast_blur" => {image::fast_blur, 2},
    "crop" => {image::crop, 5},
    "filter3x3" => {image::filter3x3, 2},
    "fliph_image" => {image::fliph_image, 1},
    "flipv_image" => {image::flipv_image, 1},
    "flipd_image" => {image::flipd_image, 1},
    "gradienth" => {image::gradienth, 5},
    "gradientv" => {image::gradientv, 5},
    "overlay" => {image::overlay, 4},
    "replace" => {image::replace, 4},
    "resize" => {image::resize, 4},
    "rotate90" => {image::rotate90, 1},
    "rotate180" => {image::rotate180, 1},
    "rotate270" => {image::rotate270, 1},
    "thumbnail" => {image::thumbnail, 3},
    "tile" => {image::tile, 2},
    "unsharpen" => {image::unsharpen, 3},
    "adaptive_threshold" => {image::adaptive_threshold, 2},
    "equalize_histogram" => {image::equalize_histogram, 1},
    "match_histogram" => {image::match_histogram, 2},
    "stretch_contrast" => {image::stretch_contrast, 5},
    "threshold" => {image::threshold, 3},
    "distance_transform" => {image::distance_transform, 2},
    "euclidean_squared_distance_transform" => {image::euclidean_squared_distance_transform, 1},
    "canny" => {image::canny, 3},
    "bilateral_filter" => {image::bilateral_filter, 4},
    "box_filter" => {image::box_filter, 3},
    "gaussian_blur" => {image::gaussian_blur, 2},
    "sharpen_gaussian" => {image::sharpen_gaussian, 3},
    "horizontal_filter" => {image::horizontal_filter, 2},
    "vertical_filter" => {image::vertical_filter, 2},
    "laplacian_filter" => {image::laplacian_filter, 1},
    "median_filter" => {image::median_filter, 3},
    "separable_filter" => {image::separable_filter, 3},
    "separable_filter_equal" => {image::separable_filter_equal, 2},
    "sharpen3x3" => {image::sharpen3x3, 1},
    "translate_image" => {image::translate_image, 3},
    "rotate_image" => {image::rotate_image, 5},
    "rotate_image_about_center" => {image::rotate_image_about_center, 3},
    "warp_image" => {image::warp_image, 3},
    "horizontal_prewitt" => {image::horizontal_prewitt, 1},
    "horizontal_scharr" => {image::horizontal_scharr, 1},
    "horizontal_sobel" => {image::horizontal_sobel, 1},
    "vertical_prewitt" => {image::vertical_prewitt, 1},
    "vertical_scharr" => {image::vertical_scharr, 1},
    "vertical_sobel" => {image::vertical_sobel, 1},
    "prewitt_gradients" => {image::prewitt_gradients, 1},
    "sobel_gradients" => {image::sobel_gradients, 1},
    "integral_image" => {image::integral_image, 1},
    "integral_squared_image" => {image::integral_squared_image, 1},
    "red_channel" => {image::red_channel, 1},
    "green_channel" => {image::green_channel, 1},
    "blue_channel" => {image::blue_channel, 1},
    "image_close" => {image::image_close, 3},
    "image_dilate" => {image::image_dilate, 3},
    "image_erode" => {image::image_erode, 3},
    "image_open" => {image::image_open, 3},
    "gaussian_noise" => {image::gaussian_noise, 4},
    "salt_and_pepper_noise" => {image::salt_and_pepper_noise, 3},
    "suppress_non_maximum" => {image::suppress_non_maximum, 2},
    "pixel_sort" => {image::pixel_sort, 3},
}

#[macro_export]
macro_rules! builtin_function {
    ($name:ident => {
        $(
            $pattern:pat => $body:expr
        ),* $(,)?
    }) => {
        pub fn $name(_rng: &mut ChaCha8Rng, _data: &Data, args: &[Value]) -> Result<Value> {
            match args {
                $(
                    $pattern => Ok($body),
                )*
                _ => Err(Error::InvalidArgument(stringify!($name).into())),
            }
        }
    };

    ($name:ident rng => {
        $(
            $pattern:pat => $body:expr
        ),* $(,)?
    }) => {
        pub fn $name(rng: &mut ChaCha8Rng, _data: &Data, args: &[Value]) -> Result<Value> {
            match args {
                $(
                    $pattern => $body(rng),
                )*
                _ => Err(Error::InvalidArgument(
                    stringify!($name).into(),
                )),
            }
        }
    };

    ($name:ident data => {
        $(
            $pattern:pat => $body:expr
        ),* $(,)?
    }) => {
        pub fn $name(_rng: &mut ChaCha8Rng, data: &Data, args: &[Value]) -> Result<Value> {
            match args {
                $(
                    $pattern => $body(data),
                )*
                _ => Err(Error::InvalidArgument(
                    stringify!($name).into(),
                )),
            }
        }
    };
}

fn dedup_shape(shape: &Rc<RefCell<Shape>>) -> Rc<RefCell<Shape>> {
    if Rc::strong_count(shape) > 2 {
        // If there's duplicates, clone the underlying data of the shape and create a new reference
        Rc::new(RefCell::new(shape.borrow().clone()))
    } else {
        // Otherwise, clone the reference to the shape
        shape.clone()
    }
}
