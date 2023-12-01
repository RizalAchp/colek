mod app_zip;
mod copy;
mod default;
mod hasher;

use std::{
    path::PathBuf,
    sync::mpsc::{channel, Receiver, Sender},
};

pub use app_zip::AppZip;
pub use copy::AppCopy;
pub use default::AppDefault;
pub use hasher::{AppHasher, HasherEventDuplicate};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use walkdir::DirEntry;

use crate::{
    filters::{is_images_impl, is_music_impl, is_videos_impl, Filter, Filters},
    system::DiskPartition,
};

#[inline]
fn filter_check(d: &DirEntry, filter: Filters) -> bool {
    macro_rules! is_filter {
        ($f:ident & $filter:ident => $fns:expr) => {
            $f.contains(Filter::$filter) && $fns
        };
    }

    match d
        .path()
        .extension()
        .and_then(|ext| ext.to_str().map(|s| s.to_ascii_lowercase()))
    {
        Some(ext) => {
            is_filter!(filter & Image => is_images_impl(&ext))
                || is_filter!(filter & Video => is_videos_impl(&ext))
                || is_filter!(filter & Music => is_music_impl(&ext))
        }
        _ => false,
    }
}

fn scans_directory(drives: Vec<DiskPartition>, tx: Sender<DirEntry>, filter: Filters) {
    rayon::spawn(move || {
        drives.into_par_iter().for_each(move |drive| {
            log::info!("Process search file in drive: {}", drive.name);
            for d in walkdir::WalkDir::new(&drive.path)
                .into_iter()
                .filter_map(move |d| match d {
                    Ok(o) if filter_check(&o, filter) => Some(o),
                    _ => None,
                })
            {
                let _ = tx.send(d).ok();
            }
        });
    })
}

pub trait App {
    type Item;

    fn name() -> &'static str;
    fn file_scan(&mut self, tx: Sender<Self::Item>, rx: Receiver<DirEntry>);
    fn on_blocking(&mut self, _rx: Receiver<Self::Item>) {}
    fn on_finish(&mut self) -> crate::Result<()>;

    fn run(&mut self, drives: Vec<DiskPartition>, filter: Filters) -> crate::Result<()> {
        log::info!("Running an App: {}", Self::name());

        let (tx_walkdir, rx_walkdir) = channel();
        scans_directory(drives, tx_walkdir, filter);

        let (tx_scanned, rx_scanned) = channel();
        self.file_scan(tx_scanned, rx_walkdir);
        self.on_blocking(rx_scanned);

        self.on_finish()
    }
}

pub struct CopyItem {
    pub dest: PathBuf,
    pub source: PathBuf,
}
