#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{borrow::ToOwned, format, string::String, vec::Vec};

#[cfg(feature = "std")]
use std::{fs, path::Path};

use anyhow::{anyhow, Result};
use base64::prelude::*;
use png::{ColorType, Encoder};
use tiny_skia::Pixmap;

mod compiler;
mod format;
mod functions;
mod minify;
mod parser;
mod renderer;

pub fn generate_pixmap(code: &str, width: u32, height: u32) -> Result<Pixmap> {
    let (_remaining, tree) = parser::parse(code.to_owned().leak()).map_err(|e| anyhow!(e))?;
    let shape = compiler::compile(tree)?;
    Ok(renderer::render(shape, width, height))
}

pub fn generate_png_data(code: &str, width: u32, height: u32) -> Result<Vec<u8>> {
    let pixmap = generate_pixmap(code, width, height)?;

    let mut buf = Vec::new();
    {
        let mut encoder = Encoder::new(&mut buf, width, height);
        encoder.set_color(ColorType::Rgba);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(pixmap.data())?;
    }
    Ok(buf)
}

pub fn generate_data_uri(code: &str, width: u32, height: u32) -> Result<String> {
    let data = generate_png_data(code, width, height)?;
    let uri = format!("data:image/png;base64,{}", BASE64_STANDARD.encode(data));
    Ok(uri)
}

#[cfg(feature = "std")]
pub fn generate_pixmap_from_file<P: AsRef<Path>>(
    input_path: P,
    width: u32,
    height: u32,
) -> Result<Pixmap> {
    let code = fs::read_to_string(input_path)?;
    generate_pixmap(&code, width, height)
}

#[cfg(feature = "std")]
pub fn generate_file<P: AsRef<Path>>(
    input_path: P,
    output_path: P,
    width: u32,
    height: u32,
) -> Result<()> {
    let pixmap = generate_pixmap_from_file(input_path, width, height)?;
    pixmap.save_png(output_path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        generate_file("example.xylo", "example.png", 400, 400).unwrap();
        let uri = generate_data_uri(
            "
root =
    let shape = SQUARE
        shape
            ",
            400,
            400,
        )
        .unwrap();
        assert_eq!(uri, "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAZAAAAGQCAYAAACAvzbMAAAQfUlEQVR4Ae3AA6AkWZbG8f937o3IzKdyS2Oubdu2bdu2bdu2bWmMnpZKr54yMyLu+Xa3anqmhztr1U9cddVVV1111b8e4qqrrrrqqqv+9RBXXXXVVVdd9a+HuOqqq6666qp/PcRVV1111VVX/eshrrrqqquuuupfD3HVVVddddVV/3qIq6666qqrrvrXQ1x11VVXXXXVvx7iqquuuuqqq/71EFddddVVV131r4e46qqrrrrqqn89xFVXXXXVVVf96yGuuuqqq6666l8PcdVVV1111VX/eoirrrrqqquu+tdDXHXVVVddddW/HuKqq6666qqr/vUQV1111VVXXfWvh7jqqquuuuqqfz3EVVddddVVV/3rIa666qqrrrrqXw9x1VVXXXXVVf96iKuuuuqqq67610NcddVVV1111b8e4qqrrrrqqqv+9RBXXXXVVVdd9a+HuOqqq6666qp/PcRVV1111VVX/eshrrrqqquuuupfD3HVVVddddVV/3qIq6666qqrrvrXQ1x11VVXXXXVvx7iqquuuuqqq/71EFddddVVV131r4e46qqrrrrqqn89xFVXXXXVVVf96yGuuuqqq6666l8PcdVVV1111VX/eoirrrrqqquu+tdDXHXVVVddddW/HuKqq6666qqr/vUQV1111VVXXfWvh7jqqquuuuqqfz3EVVddddVVV/3rIa666qqrrrrqXw9x1VVXXXXVVf96iKuuuuqqq67610NcddVVV1111b8e4qqrrrrqqqv+9RBXXXXVVVdd9a+HuOqqq6666qp/PcRVV1111VVX/eshrrrqqquuuupfD3HVVVddddVV/3qIq6666qqrrvrXQ1x11VVXXXXVvx7iqquuuuqqq/71EFddddVVV131r4e46qqrrrrqqn89xFVXXXXVVVf96yGuuuqqq6666l8PcdVVV1111VX/eoirrrrqqquu+tdDXHXVVVddddW/HuKqq6666qqr/vUQV1111VVXXfWvh7jqqquuuuqqfz3EVVddddVVV/3rIa666qqrrrrqXw9x1VVXXXXVVf96iKuuuuqqq67610NcddVVV1111b8e4qqrrrrqqqv+9RBXXXXVVVdd9a+HuOqqq6666qp/PcRVV1111VVX/eshrrrqqquuuupfD3HVVVddddVV/3qIq6666qqrrvrXQ1x11VVXXXXVvx7iqquuuuqqq/71EFddddVVV131r4e46qqrrrrqqn89xFVXXXXVVVf96yGuuuqqq6666l8PcdVVV1111VX/eoirrrrqqquu+tdDXHXVVVddddW/HuKqq6666qqr/vUQV1111VVXXfWvh7jqqquuuuqqfz3EVVddddVVV/3rIa666qqrrrrqXw9x1VVXXXXVVf96iKuuuuqqq67610NcddVVV1111b8e4qqrrrrqqqv+9RBXXXXVVVdd9a+HuOqqq6666qp/PcRVV1111VVX/eshrrrqqquuuupfD3HVVVddddVV/3qIq6666qqrrvrXQ1x11VVXXXXVvx7iqquuuuqqq/71EFddddVVV131r4e46qqrrrrqqn89xFVXXXXVVVf96yGuuuqqq6666l8PcdVVV1111VX/eoirrrrqqquu+tdDXHXVVVddddW/HuKqq6666qqr/vUQV1111VVXXfWvh7jqqquuuuqqfz3EVVddddVVV/3rIa666qqrrrrqXw9x1VVXXXXVVf96iKuuuuqqq67610NcddVVV1111b8e4qqrrrrqqqv+9RBXXXXVVVdd9a+HuOqqq6666qp/PcRVV1111VVX/eshrrrqqquuuupfD3HVVVddddVV/3qIq6666qqrrvrXQ1x11VVXXXXVvx7iqquuuuqqq/71EFddddVVV131r4e46qqrrrrqqn89xFVXXXXVVVf96yGuuuqqq6666l8PcdVVV1111VX/eoirrrrqqquu+tdDXHXVVVddddW/HuKqq6666qqr/vUQV1111VVXXfWvh7jqqquuuuqqfz3EVVddddVVV/3rIa666qqrrrrqXw9x1VVXXXXVVf96iKuuuuqqq67610NcddVVV1111b8e4qqrrrrqqqv+9RBXXXXVVVdd9a+HuOqqq6666qp/PcRVV1111VVX/eshrrrqqquuuupfD3HVVVddddVV/3qIq6666qqrrvrXQ1x11VVXXXXVvx7iqquuuuqqq/71EFddddVVV131r4e46qqrrrrqqn89xFVXXXXVVVf96yGuuuqqq6666l8PcdVVV1111VX/eoirrrrqqquu+tdDXHXVVVddddW/HuKqq6666qqr/vUQV1111VVXXfWvh7jqqquuuuqqfz3EVVddddVVV/3rIa666qqrrrrqXw9x1VVXXXXVVf96iKuuuuqqq67610NcddVVV1111b8e4qqrrrrqqqv+9RBXXXXVVVdd9a+HuOqqq6666qp/PcRVV1111VVX/eshrrrqqquuuupfD3HVVVddddVV/3qIq6666qqrrvrXQ1x11VVXXXXVvx7iqquuuuqqq/71EFddddVVV131r4e46qqrrrrqqn89xFVXXXXVVVf96yGuuuqqq6666l8PcdVVV1111VX/eoirrrrqqquu+tdDXHXVVVddddW/HuKqq6666qqr/vUQV1111VVXXfWvh7jqqquuuuqqfz3EVVddddVVV/3rIa666qqrrrrqXw9x1VVXXXXVVf96iKuuuuqqq67610NcddVVV1111b8e4qqrrrrqqqv+9RBXXXXVVVdd9a+HuOqqq6666qp/PcRVV1111VVX/eshrrrqqquuuupfD3HVVVddddVV/3qIq6666vmybQBJ4qqrrnpuiKuuuur5sm0ASeKqq656boirrrrqqquu+tdDXHXVVVddddW/HuKqq6666qqr/vUQV1111VVXXfWvh7jqqquuuuqqfz3EVVddddVVV/3rIa666qqrrrrqXw9x1VVXXXXVVf96iKuuuuqqq67610NcddVVV1111b8e4qqrrrrqqqv+9RBXXXXVVVdd9a+HuOqqq6666qp/PcRVV1111VVX/eshrrrqqquuuupfD3HVVVddddVV/3qIq6666qqrrvrXQ1x11VVXXXXVvx7iqquuuuqqq/71EFddddVVV131r4e46qqrrrrqqn89xFVXXXXVVVf96yGuuuqqq6666l8PcdVVV1111VX/eoirrrrqqquu+tdDXHXVVVddddW/HuKqq6666qqr/vUQV1111VVXXfWvh7jqqquuuuqqfz3EVVddddVVV/3rIa666qqrrrrqXw9x1VVXXXXVVf96iKuuuuqqq67610NcddVVV1111b8e4qqrrrrqqqv+9RBXXXXVVVdd9a+HuOqqq6666qp/PcRVV1111VVX/eshrrrqqquuuupfD3HVVVddddVV/3qIq6666qqrrvrXQ1x11VVXXXXVvx7iqquuuuqqq/71EFddddVVV131r4e46qqrrrrqqn89xFVXXXXVVVf96yGuuuqqq6666l8PcdVVV1111VX/eoirrrrqqquu+tdDXHXVVVddddW/HuKqq6666qqr/vUQV1111VVXXfWvh7jqqquuuuqqfz3EVVddddVVV/3rIa666qqrrrrqXw9x1VVXXXXVVf96iKuuuuqqq67610NcddVVV1111b8e4qqrrrrqqqv+9RBXXXXVVVdd9a+HuOqqq6666qp/PcRVV1111VVX/eshrrrqqquuuupfD3HVVVddddVV/3qIq6666qqrrvrXQ1x11VVXXXXVvx7iqquuuuqqq/71EFddddVVV131r4e46qqrrrrqqn89xFVXXXXVVVf96yGuuuqqq6666l8PcdVVV1111VX/eoirrrrqqquu+tdDXHXVVVddddW/HuKqq6666qqr/vUQV1111VVXXfWvh7jqqquuuuqqfz3EVVddddVVV/3rIa666qqrrrrqXw9x1VVXXXXVVf96iKuuuuqqq67610NcddVVV1111b8e4qqrrrrqqqv+9RBXXXXVVVdd9a+HuOqqq6666qp/PcRVV1111VVX/eshrrrqqquuuupfD3HVVVddddVV/3qIq6666qqrrvrXQ1x11VVXXXXVvx7iqquuuuqqq/71EFddddVVV131r4e46qqrrrrqqn89xFVXXXXVVVf96yGuuuqqq6666l8PcdVVV1111VX/eoirrrrqqquu+tdDXHXVVVddddW/HuKqq6666qqr/vUQV1111VVXXfWvh7jqqquuuuqqfz3EVVddddVVV/3rIa666qqrrrrqXw9x1VVXXXXVVf96iKuuuuqqq67610NcddVVV1111b8e4qqrrrrqqqv+9RBXXXXVVVdd9a+HuOqqq6666qp/PcRVV1111VVX/eshrrrqqquuuupfD3HVVVddddVV/3qIq6666qqrrvrXQ1x11VVXXXXVvx7iqquuuuqqq/71EFddddVVV131r4e46qqrrrrqqn89xFVXXXXVVVf96yGuuuqqq6666l8PcdVVV1111VX/eoirrrrqqquu+tdDXHXVVVddddW/HuKqq6666qqr/vUQV1111VVXXfWvh7jqqquuuuqqfz3EVVddddVVV/3rIa666qqrrrrqXw9x1VVXXXXVVf96iKuuuuqqq67610NcddVVV1111b8e4qqrrrrqqqv+9RBXXXXVVVdd9a+HuOqqq6666qp/PcRVV1111VVX/eshrrrqqquuuupfD3HVVVddddVV/3qIq6666qqrrvrXQ1x11VVXXXXVvx7iqquuuuqqq/71EFddddVVV131r4e46qqrrrrqqn89xFVXXXXVVVf96yGuuuqqq6666l8PcdVVV1111VX/eoirrrrqqquu+tdDXHXVVVddddW/HuKqq6666qqr/vUQV1111VVXXfWvh7jqqquuuuqqfz3EVVddddVVV/3rIa666qqrrrrqXw9x1VVXXXXVVf96iKuuuuqqq67610NcddVVV1111b8e4qqrrrrqqqv+9RBXXXXVVVdd9a+HuOqqq6666qp/PcRVV1111VVX/eshrrrqqquuuupfD3HVVVddddVV/3qIq6666qqrrvrXQ1x11VVXXXXVvx7iqquuuuqqq/71EFddddVVV131r4e46qqrrrrqqn89xFVXXXXVVVf96yGuuuqqq6666l8PcdVVV1111VX/eoirrrrqqquu+tdDXHXVVVddddW/HuKqq6666qqr/vUQV1111VVXXfWvh7jqqquuuuqqfz3EVVddddVVV/3rIa666qqrrrrqXw9x1VVXXXXVVf96iKuuuuqqq67610NcddVVV1111b8e4qqrrrrqqqv+9RBXXXXVVVdd9a+HuOqqq6666qp/PcRVV1111VVX/eshrrrqqquuuupfD3HVVVddddVV/3qIq6666qqrrvrXQ1x11VVXXXXVvx7iqquuuuqqq/71EFddddVVV131r4e46qqrrrrqqn89xFVXXXXVVVf96yGuuuqqq6666l8PcdVVV1111VX/eoirrrrqqquu+tdDXHXVVVddddW/HuKqq6666qqr/vUQV1111VVXXfWvh7jqqquuuuqqfz3EVVddddVVV/3rIa666qqrrrrqX49/BA5uCZF/KhQeAAAAAElFTkSuQmCC");
    }
}
