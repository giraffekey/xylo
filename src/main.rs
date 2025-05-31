use clap::{Parser, Subcommand};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::time::SystemTime;
use xylo_lang::{format_file, generate_file, minify_file, Config, Result};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Generate {
        source: PathBuf,
        dest: Option<PathBuf>,
        #[arg(long)]
        width: Option<u32>,
        #[arg(long)]
        height: Option<u32>,
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

fn main() {
    match run_cli() {
        Ok(()) => (),
        Err(e) => eprintln!("{}", e.to_string()),
    }
}

fn run_cli() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Generate {
            source,
            dest,
            width,
            height,
            seed,
        }) => {
            let dest = match dest {
                Some(dest) => dest,
                None => format!(
                    "{}.png",
                    source
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .split(".")
                        .next()
                        .unwrap()
                )
                .into(),
            };

            let width = width.unwrap_or(400);
            let height = height.unwrap_or(400);

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
                seed,
            };

            let now = SystemTime::now();
            generate_file(source, &dest, config)?;

            println!(
                "Output to {:?} in {:?}",
                dest,
                SystemTime::now().duration_since(now).unwrap()
            );
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
