mod app;
mod dir;
mod filters;
mod system;

use std::path::PathBuf;

use app::App;
use clap::Parser;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const SIZE_BUF_WRITE: usize = (8 << 10) * 1000;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, clap::Subcommand)]
pub enum Filter {
    Image,
    Video,
    Music,
    Other {
        /// ignorecase
        #[arg(long, short, default_value_t = true)]
        ignorecase: bool,
        /// search with name
        #[arg(long, short)]
        name: Option<String>,
        /// search with extensions
        #[arg(long, short, required = true)]
        extension: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, clap::Parser)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Log Level [possible values: TRACE, DEBUG, INFO, WARN, ERROR]
    #[arg(
        long, short,
        default_value_t = if cfg!(debug_assertions) { log::LevelFilter::Debug } else { log::LevelFilter::Warn },
        value_parser = clap::value_parser!(log::LevelFilter),
    )]
    log_level: log::LevelFilter,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Clone, PartialEq, clap::Subcommand)]
enum Commands {
    /// Output the scaned file to Stdout ( the path name )
    Stdout {
        #[command(subcommand)]
        filter: Filter,
    },

    /// Just Copy in the target Directories
    Copy {
        /// target directories to copy the files scanned
        #[arg(long, short)]
        target: Option<PathBuf>,

        #[command(subcommand)]
        filter: Filter,
    },

    /// Output to Zip Files
    Zip {
        /// output files
        #[arg(long, short, required = false)]
        output: Option<PathBuf>,

        /// filter for walkdir [possible values: image, music, video, <other: extensions>]
        #[command(subcommand)]
        filter: Filter,
    },
}

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    env_logger::Builder::from_env(env_logger::Env::default())
        .filter_level(args.log_level)
        .init();

    match args.command {
        Commands::Stdout { filter } => {
            let mut sys = system::SystemDiskInfo::new(filter);
            let mut application = app::AppDefault::new(&mut sys).await?;
            application.run(&mut sys).await?;
            application.finish().await
        }
        Commands::Copy { target, filter } => {
            let mut sys = system::SystemDiskInfo::new(filter).with_output(target);
            let mut application = app::AppCopy::new(&mut sys).await?;
            application.run(&mut sys).await?;
            application.finish().await
        }
        Commands::Zip { output, filter } => {
            let mut sys = system::SystemDiskInfo::new(filter).with_output(output);
            let mut application = app::AppZip::new(&mut sys).await?;
            application.run(&mut sys).await?;
            application.finish().await
        }
    }
}
