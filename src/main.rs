mod app;
/// TODO(browser_stealer): implement browser information stealer
// mod browser;
mod filters;
mod logger;
mod system;
mod error;

use error::{ColekError, Result};
use std::path::PathBuf;

use clap::Parser;
use filters::Filter;

const APP_NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Debug, Clone, PartialEq, clap::Parser)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Log Level [possible values: TRACE, DEBUG, INFO, WARN, ERROR]
    #[arg(long, short, default_value_t = false)]
    verbose: bool,

    #[clap(long, short, value_delimiter=',', action=clap::ArgAction::Append, default_value="image")]
    filter: Vec<Filter>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Clone, PartialEq, clap::Subcommand)]
enum Commands {
    /// Output the scaned file to Stdout ( the path name )
    Stdout,

    /// Just Copy in the target Directories
    Copy {
        /// target directories to copy the files scanned
        #[arg(long, short, required = false)]
        target: Option<PathBuf>,
    },

    /// Output to Zip Files
    Zip {
        /// output files
        #[arg(long, short, required = false)]
        output: Option<PathBuf>,
    },

    /// Hash the file scanned using sha256
    Hash {
        /// on duplicate event
        #[arg(short, long, default_value = "print")]
        duplicate: app::HasherEventDuplicate,
    },
}

fn main() -> Result<()> {
    let args = CliArgs::parse();
    logger::init(args.verbose);
    log::info!("Starting Program");

    let mut sys = system::SystemDiskInfo::new();
    let filter = get_filter(args.filter);
    match args.command {
        Commands::Stdout => {
            let mut application = app::AppDefault::new()?;
            app::App::run(&mut application, &mut sys, filter)
        }
        Commands::Copy { target } => {
            let mut application = app::AppCopy::new(sys.dest(target))?;
            app::App::run(&mut application, &mut sys, filter)
        }
        Commands::Zip { output } => {
            let mut application = app::AppZip::new(sys.dest(output))?;
            app::App::run(&mut application, &mut sys, filter)
        }
        Commands::Hash { duplicate } => {
            let mut application = app::AppHasher::new(duplicate);
            app::App::run(&mut application, &mut sys, filter)
        }
    }
}

#[inline]
fn get_filter(filters: impl AsRef<[Filter]>) -> u32 {
    let mut out = 0;
    for filter in filters.as_ref() {
        out |= *filter as u32;
    }
    out
}
