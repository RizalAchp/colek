use std::io::{stderr, Write};

use log::LevelFilter;

static LOGGER: Logger = Logger;

struct Logger;
impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let _ = writeln!(
            stderr(),
            "{:<5}:{}: {}",
            record.level(),
            record.target(),
            record.args()
        )
        .ok();
    }

    fn flush(&self) {
        stderr().flush().ok();
    }
}

pub fn init(verbose: bool) {
    log::set_logger(&LOGGER).unwrap_or_else(|err| {
        eprintln!("Failed to set logger - {err}");
    });
    log::set_max_level(if verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Warn
    })
}
