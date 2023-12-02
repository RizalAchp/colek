mod app_zip;
mod copy;
mod default;
mod hasher;

use std::{
    fs::File,
    io::Read,
    sync::mpsc::{channel, Receiver, Sender},
};

pub use app_zip::AppZip;
pub use copy::AppCopy;
pub use default::AppDefault;
pub use hasher::{AppHasher, HasherEventDuplicate};
use ignore::{DirEntry, WalkState};

use crate::{
    filters::{contains_magic_bytes, Filter, Filters, MAGIC_BYTE_MAX_LEN},
    system::DiskPartition,
};

fn pararel_scan(
    filter: Filters,
    tx: Sender<DirEntry>,
) -> Box<dyn FnMut(Result<DirEntry, ignore::Error>) -> WalkState + Send> {
    Box::new(move |p| {
        let Ok(p) = p.map_err(|err| log::error!("walking directory: (Reason: {err})")) else {
            return WalkState::Continue;
        };
        let ext = p
            .path()
            .extension()
            .map(|x| x.to_str().unwrap_or("").to_lowercase())
            .unwrap_or_default();

        if filter.matches(&ext) {
            tx.send(p).ok();
        } else if filter.contains(Filter::Image) {
            if let Ok(mut file) = File::open(p.path()) {
                let mut buf = [0u8; MAGIC_BYTE_MAX_LEN];
                file.read(&mut buf[..]).ok();
                if contains_magic_bytes(buf) {
                    log::debug!("Found magic bytes for: '{}'", p.path().display());
                    tx.send(p).ok();
                }
            }
        }

        WalkState::Continue
    })
}

fn scans_directory(drives: Vec<DiskPartition>, tx: Sender<DirEntry>, filter: Filters) {
    log::debug!("Start Scanning directory");
    if drives.is_empty() {
        return;
    }
    rayon::spawn(move || {
        let mut drive_iter = drives.iter();
        let p = drive_iter.next().expect("should never fail");
        let mut walkbuilder = ignore::WalkBuilder::new(&p.path);
        for drive in drive_iter {
            walkbuilder.add(&drive.path);
        }
        walkbuilder.standard_filters(true).threads(4);
        walkbuilder.build_parallel().run(|| {
            let tx = tx.clone();
            pararel_scan(filter, tx)
        });
    });
    log::debug!("End Scanning directory");
}

pub trait App {
    type Item;

    fn name() -> &'static str;
    fn file_scan(&mut self, tx: Sender<Self::Item>, rx: Receiver<DirEntry>) -> crate::Result<()>;
    fn on_blocking(&mut self, _rx: Receiver<Self::Item>) -> crate::Result<()> {
        Ok(())
    }
    fn on_finish(&mut self) -> crate::Result<()> {
        log::debug!("{}: Finish", Self::name());
        Ok(())
    }

    fn run(&mut self, drives: Vec<DiskPartition>, filter: Filters) -> crate::Result<()> {
        log::info!("Running an App: {}", Self::name());

        let (tx_walkdir, rx_walkdir) = channel();
        scans_directory(drives, tx_walkdir, filter);

        let (tx_scanned, rx_scanned) = channel();
        self.file_scan(tx_scanned, rx_walkdir)?;
        self.on_blocking(rx_scanned)?;

        let r = self.on_finish();
        log::info!("Finish running App: {}", Self::name());
        r
    }
}
