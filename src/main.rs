mod app;
mod error;
mod filters;
mod logger;
mod system;

use app::App;
use error::{ColekError, Result};
use logger::LogLevel;
use std::{path::PathBuf, process::ExitCode};

use clap::Parser;
use filters::{Filter, Filters};

const APP_NAME: &str = env!("CARGO_PKG_NAME");
fn main() -> ExitCode {
    let args = CliArgs::parse();
    logger::init(args.verbose);
    log::info!("{APP_NAME} - Starting Program");

    let mut sys = system::SystemDiskInfo::new();
    dbg!(&args.filter);
    let filter = Filters::from(args.filter.unwrap_or_else(|| vec![Filter::Image]));
    if let Err(err) = args.command.run(&mut sys, filter) {
        log::error!("{APP_NAME} - Failed on running command: {err}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
#[derive(Debug, Clone, PartialEq, clap::Parser)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// set filter for runner app
    #[clap(long, short, value_delimiter=',', action=clap::ArgAction::Append)]
    filter: Option<Vec<Filter>>,

    /// set max verbosity level for stdout/stderr logger
    #[arg(long, short, default_value = "warn")]
    verbose: LogLevel,

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

impl Commands {
    pub fn run(self, sys: &mut system::SystemDiskInfo, filter: Filters) -> Result<()> {
        let Some(drives) = sys.generic_drive() else {
            return Err(crate::ColekError::NoGenericDrive);
        };
        match self {
            Commands::Stdout => {
                let mut application = app::AppDefault::new()?;
                application.run(drives, filter)
            }
            Commands::Copy { target } => {
                let mut application = app::AppCopy::new(sys.dest(target))?;
                application.run(drives, filter)
            }
            Commands::Zip { output } => {
                let mut application = app::AppZip::new(sys.dest(output))?;
                application.run(drives, filter)
            }
            Commands::Hash { duplicate } => {
                let mut application = app::AppHasher::new(duplicate);
                application.run(drives, filter)
            }
        }
    }
}
