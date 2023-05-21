pub mod comp;
pub mod watching;

use std::path::PathBuf;

use clap::Parser;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

/// A tool to watch and compile minecraft resourcepacks
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
enum Args {
    /// Watches the given directory for changes and recompile resourcepack
    #[clap(name = "watch")]
    Watch(WatchCmd),
    /// Compiles the given directory to resourcepack with additional settings
    #[clap(name = "compile")]
    Compile(CompileCmd),
}

#[derive(Parser)]
pub struct WatchCmd {
    /// Root directory to watch
    root_dir: PathBuf,
    /// Specifies the output file
    #[arg(long = "out", short, default_value = "pack.zip")]
    out_file: PathBuf,
    // /// Whether to use PackSquash for compilation and validation, or simple zipping
    // #[arg(long, short)]
    // packsquash: bool,
}

#[derive(Parser)]
pub struct CompileCmd {
    /// Root directory to build
    root_dir: PathBuf,
    /// Specifies the output file
    #[arg(long = "out", short, default_value = "pack.zip")]
    out_file: PathBuf,
    // /// Whether to use PackSquash for compilation and validation, or simple zipping
    // #[arg(long, short)]
    // packsquash: bool,
}

#[tracing::instrument]
fn main() -> anyhow::Result<()> {
    // Logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().compact().with_ansi(true))
        .init();

    // Args
    let args = Args::parse();

    match args {
        Args::Watch(watch_cmd) => {
            watching::watch(watch_cmd)?;
        }
        Args::Compile(compile_cmd) => {
            if let Err(error) = comp::compile_pack(compile_cmd.root_dir, compile_cmd.out_file) {
                tracing::error!("Failed to compile resource pack: {error}")
            }
        }
    }
    Ok(())
}
