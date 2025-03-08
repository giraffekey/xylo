#[cfg(feature = "std")]
use {std::time::SystemTime, xylo_lang::generate_file};

#[cfg(feature = "std")]
fn main() {
    let now = SystemTime::now();
    generate_file("example.xylo", "example.png", 800, 800).unwrap();
    println!("{:?}", SystemTime::now().duration_since(now).unwrap());
}

#[cfg(not(feature = "std"))]
fn main() {}
