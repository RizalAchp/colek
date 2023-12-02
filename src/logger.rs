use std::io::{stderr, Write};

use log::LevelFilter;

static LOGGER: Logger = Logger;

struct Logger;
impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        !["ignore::walk", "globset"]
            .into_iter()
            .any(|x| x.contains(metadata.target()))
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let _ = writeln!(
                stderr(),
                "{:<5}:{}: {}",
                record.level(),
                record.target(),
                record.args()
            )
            .ok();
        }
    }

    fn flush(&self) {
        stderr().flush().ok();
    }
}

pub fn init(level_filter: LogLevel) {
    log::set_logger(&LOGGER).unwrap_or_else(|err| {
        eprintln!("Failed to set logger - {err}");
    });
    log::set_max_level(level_filter.into_level_filter())
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
#[clap(rename_all = "UPPER")]
pub enum LogLevel {
    Off,
    Error,
    #[default]
    Warn,
    Info,
    Debug,
    Trace,
}
impl LogLevel {
    const fn into_level_filter(self) -> LevelFilter {
        match self {
            Self::Off => LevelFilter::Off,
            Self::Error => LevelFilter::Error,
            Self::Warn => LevelFilter::Warn,
            Self::Info => LevelFilter::Info,
            Self::Debug => LevelFilter::Debug,
            Self::Trace => LevelFilter::Trace,
        }
    }
}
