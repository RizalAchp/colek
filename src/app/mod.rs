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
    filters::{is_images_impl, is_music_impl, is_videos_impl, Filter},
    system::DiskPartition,
};
macro_rules! filter_and {
    ($f:ident & $filter:ident => $fns:expr) => {
        if ($f & (Filter::$filter as u32)) != 0 {
            $fns
        } else {
            false
        }
    };
}

fn scans_directory(drives: Vec<DiskPartition>, tx: Sender<DirEntry>, filter: u32) {
    let filter = |d: &DirEntry| -> bool {
        match d
            .path()
            .extension()
            .map(|ext| ext.to_str().map(|s| s.to_ascii_lowercase()))
        {
            Some(Some(ext)) => {
                let img = filter_and!(filter & Image => is_images_impl(&ext));
                let video = filter_and!(filter & Video => is_videos_impl(&ext));
                let music = filter_and!(filter & Music => is_music_impl(&ext));
                img || video || music
            }
            _ => false,
        }
    };
    drives.into_par_iter().for_each(move |drive| {
        log::info!("Process search file in drive: {}", drive.name);
        walkdir::WalkDir::new(&drive.path)
            .into_iter()
            .for_each(|x| match x {
                Ok(k) if filter(&k) => {
                    let _ = tx.send(k).ok();
                }
                Err(err) => {
                    log::error!("Failed when walkdir - {err}");
                }
                _ => (),
            })
    });
}

pub trait App: Sized + Clone + Send {
    type Item: Send;

    fn name() -> &'static str;
    fn file_scan(&mut self, tx: Sender<Self::Item>, rx: Receiver<DirEntry>);
    fn on_blocking(&mut self, _rx: Receiver<Self::Item>) {}
    fn on_finish(&mut self) -> crate::Result<()>;

    fn run(&mut self, sys: &mut crate::system::SystemDiskInfo, filter: u32) -> crate::Result<()> {
        log::info!("Running an App: {}", Self::name());
        let Some(drives) = sys.generic_drive() else {
            return Err(crate::ColekError::NoGenericDrive);
        };
        let (tx_walkdir, rx_walkdir) = channel();

        rayon::spawn(move || {
            scans_directory(drives, tx_walkdir, filter);
        });

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
