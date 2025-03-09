use crate::interpreter::Value;

use anyhow::{anyhow, Result};
use rand_chacha::ChaCha8Rng;

mod compare;
mod func;
mod list;
mod math;
mod rand;
mod shape;

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
        pub fn handle_builtin(name: &str, rng: &mut ChaCha8Rng, args: &[Value]) -> Result<Value> {
            match name {
                $(
                    $name => $func(rng, args),
                )*
                _ => Err(anyhow!("Unknown function: {}", name)),
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
    ":" => {shape::compose, 2},
    "collect" => {shape::collect, 1},
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
    "complex" => {math::complex, 2},
    "rand" => {rand::rand, 0},
    "randi" => {rand::randi, 0},
    "rand_range" => {rand::rand_range, 2},
    "randi_range" => {rand::randi_range, 2},
    "rand_rangei" => {rand::rand_rangei, 2},
    "randi_rangei" => {rand::randi_rangei, 2},
    "shuffle" => {rand::shuffle, 1},
    "choose" => {rand::choose, 1},
    "compose" => {shape::compose, 2},
    "t" => {shape::translate, 3},
    "translate" => {shape::translate, 3},
    "tx" => {shape::translatex, 2},
    "translatex" => {shape::translatex, 2},
    "ty" => {shape::translatey, 2},
    "translatey" => {shape::translatey, 2},
    "tt" => {shape::translateb, 2},
    "translateb" => {shape::translateb, 2},
    "r" => {shape::rotate, 2},
    "rotate" => {shape::rotate, 2},
    "ra" => {shape::rotate_at, 4},
    "rotate_at" => {shape::rotate_at, 4},
    "s" => {shape::scale, 3},
    "scale" => {shape::scale, 3},
    "sx" => {shape::scalex, 2},
    "scalex" => {shape::scalex, 2},
    "sy" => {shape::scaley, 2},
    "scaley" => {shape::scaley, 2},
    "ss" => {shape::scaleb, 2},
    "scaleb" => {shape::scaleb, 2},
    "k" => {shape::skew, 3},
    "skew" => {shape::skew, 3},
    "kx" => {shape::skewx, 2},
    "skewx" => {shape::skewx, 2},
    "ky" => {shape::skewy, 2},
    "skewy" => {shape::skewy, 2},
    "kk" => {shape::skewb, 2},
    "skewb" => {shape::skewb, 2},
    "f" => {shape::flip, 2},
    "flip" => {shape::flip, 2},
    "fh" => {shape::fliph, 1},
    "fliph" => {shape::fliph, 1},
    "fv" => {shape::flipv, 1},
    "flipv" => {shape::flipv, 1},
    "fd" => {shape::flipd, 1},
    "flipd" => {shape::flipd, 1},
    "z" => {shape::zindex, 2},
    "zindex" => {shape::zindex, 2},
    "hsl" => {shape::hsl, 4},
    "hsla" => {shape::hsla, 5},
    "h" => {shape::hue, 2},
    "hue" => {shape::hue, 2},
    "sat" => {shape::saturation, 2},
    "saturation" => {shape::saturation, 2},
    "l" => {shape::lightness, 2},
    "lightness" => {shape::lightness, 2},
    "a" => {shape::alpha, 2},
    "alpha" => {shape::alpha, 2},
    "hshift" => {shape::hshift, 2},
    "sshift" => {shape::sshift, 2},
    "lshift" => {shape::lshift, 2},
    "ashift" => {shape::ashift, 2},
    "hex" => {shape::hex, 2},
    // "blend" => {::blend, 2},
    "move_to" => {shape::move_to, 2},
    "line_to" => {shape::line_to, 2},
    "quad_to" => {shape::quad_to, 4},
    "cubic_to" => {shape::cubic_to, 6},
    "close" => {shape::close, 0},
}
