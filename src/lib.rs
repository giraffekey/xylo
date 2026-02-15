#![cfg_attr(feature = "alloc", no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Enable either `std` or `alloc`, but not both!");

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either `std` or `alloc` must be enabled!");

mod colors;
mod error;
mod format;
mod functions;
mod interpreter;
mod minify;
mod out;
mod parser;
mod renderer;
mod shape;

pub use error::{Error, Result};
pub use format::format;
pub use minify::minify;
pub use out::*;
