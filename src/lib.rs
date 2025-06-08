#![cfg_attr(feature = "no-std", no_std)]

#[cfg(feature = "no-std")]
extern crate alloc;

#[cfg(all(feature = "std", feature = "no-std"))]
compile_error!("Enable either `std` or `no-std`, but not both!");

#[cfg(not(any(feature = "std", feature = "no-std")))]
compile_error!("Either `std` or `no-std` must be enabled!");

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
