use crate::error::{Error, Result};
use crate::interpreter::{Data, Value};

use rand_chacha::ChaCha8Rng;

mod color;
mod compare;
mod func;
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
    "intersperse" => {list::intersperse, 2},
    "copy" => {shape::copy, 1},
    "|>" => {func::pipe, 2},
    "pipe" => {func::pipe, 2},
    // "." => compose_fn 2,
    // "compose_fn" => compose_fn 2,
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
    "mask" => {shape::mask, 1},
    "unmask" => {shape::unmask, 1},
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
