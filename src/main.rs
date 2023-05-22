pub mod comp;
pub mod validate;
pub mod watching;

use std::path::PathBuf;

use chrono::Local;
use clap::Parser;

use tracing_subscriber::{
    fmt::{time::FormatTime, SubscriberBuilder},
    prelude::__tracing_subscriber_SubscriberExt,
    util::SubscriberInitExt,
};

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
    #[clap(default_value = ".")]
    root_dir: PathBuf,
    /// Specifies the output file
    #[arg(long = "out", short, default_value = "pack.zip")]
    out_file: PathBuf,
    /// Whether to validate JSON and strip JSONC comments
    #[arg(long, short, default_value_t = false)]
    validate: bool,
    // /// Whether to use PackSquash for compilation and validation, or simple zipping
    // #[arg(long, short)]
    // packsquash: bool,
}

#[derive(Parser)]
pub struct CompileCmd {
    /// Root directory to build
    #[clap(default_value = ".")]
    root_dir: PathBuf,
    /// Specifies the output file
    #[arg(long = "out", short, default_value = "pack.zip")]
    out_file: PathBuf,
    /// Whether to validate JSON and strip JSONC comments
    #[arg(long, short, default_value_t = false)]
    validate: bool,
    // /// Whether to use PackSquash for compilation and validation, or simple zipping
    // #[arg(long, short)]
    // packsquash: bool,
}

#[derive(Debug, Clone)]
struct TimeFmt;

impl FormatTime for TimeFmt {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        w.write_fmt(format_args!("{}", Local::now().format("%H:%M:%S")))
    }
}

#[tracing::instrument]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Logging
    let filter = tracing_subscriber::filter::filter_fn(|meta| {
        let is_rpd = meta.module_path().unwrap_or_default().contains("rpd")
            && *meta.level() <= tracing::Level::TRACE;
        is_rpd || *meta.level() <= tracing::Level::WARN
    });
    let subscriber = SubscriberBuilder::default()
        .compact()
        .with_ansi(true)
        .with_timer(TimeFmt)
        .finish();
    subscriber.with(filter).init();

    // Args
    let args = Args::parse();
    match args {
        Args::Watch(watch_cmd) => {
            if let Err(error) = watching::watch(watch_cmd).await {
                tracing::error!("Failed to watch directory: {error}")
            }
        }
        Args::Compile(compile_cmd) => {
            if let Err(error) = comp::compile_pack(
                compile_cmd.root_dir,
                compile_cmd.out_file,
                compile_cmd.validate,
            )
            .await
            {
                tracing::error!("Failed to compile resource pack: {error}")
            }
        }
    }
    Ok(())
}
