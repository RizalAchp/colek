mod app_zip;
mod copy;
mod default;
mod hasher;

use std::{
    fs::File,
    io::Read,
    sync::mpsc::{channel, Receiver, Sender},
    time::Instant,
};

pub use app_zip::AppZip;
pub use copy::AppCopy;
pub use default::AppDefault;
pub use hasher::{AppHasher, HasherEventDuplicate};
use ignore::{DirEntry, ParallelVisitor, ParallelVisitorBuilder, WalkState};

use crate::{
    filters::{contains_magic_bytes, Filter, Filters, MAGIC_BYTE_MAX_LEN},
    system::DiskPartition,
};

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
        walkbuilder
            .standard_filters(true)
            .threads(4)
            .build_parallel()
            .visit(&mut ParallelScanBuilder(&filter, &tx));
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
        let start = Instant::now();

        let (tx_walkdir, rx_walkdir) = channel();
        scans_directory(drives, tx_walkdir, filter);

        let (tx_scanned, rx_scanned) = channel();
        self.file_scan(tx_scanned, rx_walkdir)?;
        self.on_blocking(rx_scanned)?;

        let r = self.on_finish();
        let elapsed = start.elapsed();
        log::info!("Finish running App: {}", Self::name());
        log::info!("    in {:.3}s", elapsed.as_secs_f32());
        r
    }
}

struct ParallelScan {
    filters: Filters,
    tx: Sender<DirEntry>,
}

impl ParallelScan {
    fn visit_parallel(&mut self, entry: DirEntry) -> WalkState {
        match entry.path().extension().and_then(|x| {
            x.to_str()
                .and_then(|ext| Filter::from_extension(ext.to_lowercase()))
        }) {
            Some(filter_type) if self.filters.contains(filter_type) => {
                self.tx.send(entry).ok();
            }
            Some(_) => {}
            None => {
                let Ok(mut file) = File::open(entry.path()) else {
                    return WalkState::Continue;
                };
                log::debug!("try matching magic bytes: {}", entry.path().display());
                let mut buf = [0u8; MAGIC_BYTE_MAX_LEN];
                file.read(&mut buf[..]).ok();
                if contains_magic_bytes(buf) {
                    log::debug!("Found magic bytes for: '{}'", entry.path().display());
                    self.tx.send(entry).ok();
                }
            }
        }
        WalkState::Continue
    }
}

impl ParallelVisitor for ParallelScan {
    #[inline]
    fn visit(&mut self, entry: Result<DirEntry, ignore::Error>) -> WalkState {
        let Ok(entry) = entry.map_err(|err| log::error!("walking directory: (Reason: {err})"))
        else {
            return WalkState::Continue;
        };
        if !entry.file_type().is_some_and(|x| x.is_file()) {
            return WalkState::Continue;
        }
        self.visit_parallel(entry)
    }
}

struct ParallelScanBuilder<'f, 's>(&'f Filters, &'s Sender<DirEntry>);
impl<'f, 's, 'p> ParallelVisitorBuilder<'p> for ParallelScanBuilder<'f, 's> {
    fn build(&mut self) -> Box<dyn ParallelVisitor + 'p> {
        Box::new(ParallelScan {
            filters: *self.0,
            tx: self.1.clone(),
        }) as Box<dyn ParallelVisitor + 'p>
    }
}
