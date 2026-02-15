#[cfg(feature = "std")]
use {
    clap::{Parser, Subcommand},
    sha2::{Digest, Sha256},
    std::path::PathBuf,
    std::time::SystemTime,
    xylo_lang::{format_file, generate_file, minify_file, Config, Result},
};

#[cfg(feature = "std")]
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[cfg(feature = "std")]
#[derive(Subcommand)]
enum Commands {
    #[cfg(feature = "window-std")]
    Run {
        source: PathBuf,
        #[arg(long)]
        width: Option<u32>,
        #[arg(long)]
        height: Option<u32>,
        #[arg(long)]
        max_depth: Option<usize>,
        #[arg(short, long)]
        seed: Option<String>,
    },
    #[cfg(feature = "image-std")]
    Generate {
        source: PathBuf,
        dest: Option<PathBuf>,
        #[arg(long)]
        width: Option<u32>,
        #[arg(long)]
        height: Option<u32>,
        #[arg(long)]
        max_depth: Option<usize>,
        #[arg(short, long)]
        count: Option<u32>,
        #[arg(short, long)]
        frames: Option<usize>,
        #[arg(short, long)]
        seed: Option<String>,
    },
    Minify {
        source: PathBuf,
        dest: Option<PathBuf>,
    },
    Format {
        source: PathBuf,
        dest: Option<PathBuf>,
    },
}

#[cfg(feature = "std")]
fn main() {
    match run_cli() {
        Ok(()) => (),
        Err(e) => eprintln!("{}", e.to_string()),
    }
}

#[cfg(feature = "std")]
fn run_cli() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        #[cfg(feature = "window-std")]
        Some(Commands::Run {
            source: _,
            width: _,
            height: _,
            max_depth: _,
            seed: _,
        }) => {
            todo!();
        }
        #[cfg(feature = "image-std")]
        Some(Commands::Generate {
            source,
            dest,
            width,
            height,
            max_depth,
            count,
            frames,
            seed,
        }) => {
            let frames = frames.unwrap_or(1);
            let dest = match dest {
                Some(dest) => dest,
                None => {
                    let file_name = source
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .split(".")
                        .next()
                        .unwrap();
                    if frames > 1 {
                        format!("{}.gif", file_name,).into()
                    } else {
                        format!("{}.png", file_name,).into()
                    }
                }
            };

            let width = width.unwrap_or(400);
            let height = height.unwrap_or(400);
            let max_depth = max_depth.unwrap_or(1500);
            let count = count.unwrap_or(1);

            let seed = match seed {
                Some(seed) => {
                    let mut hasher = Sha256::default();
                    hasher.update(seed.as_bytes());
                    let mut seed = [0; 32];
                    hasher.finalize_into((&mut seed).into());
                    Some(seed)
                }
                None => None,
            };

            let config = Config {
                dimensions: (width, height),
                max_depth,
                seed,
            };

            for i in 0..count {
                let dest = if count == 1 {
                    dest.clone()
                } else {
                    dest.parent().unwrap().join(format!(
                        "{}_{}.{}",
                        dest.file_stem().unwrap().to_string_lossy(),
                        i,
                        dest.extension().unwrap().to_string_lossy()
                    ))
                };

                let now = SystemTime::now();
                generate_file(&source, &dest, frames, config)?;

                println!(
                    "Output to {:?} in {:?}",
                    dest,
                    SystemTime::now().duration_since(now).unwrap()
                );
            }
        }
        Some(Commands::Minify { source, dest }) => {
            let dest = dest.unwrap_or(source.clone());
            let now = SystemTime::now();
            minify_file(source, &dest)?;
            println!(
                "Output to {:?} in {:?}",
                dest,
                SystemTime::now().duration_since(now).unwrap()
            );
        }
        Some(Commands::Format { source, dest }) => {
            let dest = dest.unwrap_or(source.clone());
            let now = SystemTime::now();
            format_file(source, &dest)?;
            println!(
                "Output to {:?} in {:?}",
                dest,
                SystemTime::now().duration_since(now).unwrap()
            );
        }
        None => (),
    }

    Ok(())
}

#[cfg(feature = "alloc")]
fn main() {}
