use std::{
    path::PathBuf,
    sync::{
        mpsc::{Receiver, Sender},
        Arc,
    },
};

use rayon::prelude::{ParallelBridge, ParallelIterator};
use walkdir::DirEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum HasherEventDuplicate {
    Remove,
    Rename,
    Print,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hashes {
    hash: u128,
    size: usize,
}

#[derive(Debug, Clone)]
pub struct AppHasher {
    event_duplicate: Arc<HasherEventDuplicate>,
    hashes: Vec<(PathBuf, Hashes)>,
}
impl AppHasher {
    pub fn new(event_duplicate: HasherEventDuplicate) -> Self {
        Self {
            event_duplicate: Arc::new(event_duplicate),
            hashes: Vec::new(),
        }
    }
}

impl super::App for AppHasher {
    type Item = (PathBuf, Hashes);

    fn name() -> &'static str {
        "Hasher App"
    }
    fn on_blocking(&mut self, recver: Receiver<Self::Item>) {
        log::debug!("on_blocking");
        while let Ok(item) = recver.recv() {
            let (path, hash) = &item;
            for other in self.hashes.iter() {
                let (other_path, other_hash) = other;
                if other_hash == hash {
                    match *self.event_duplicate {
                        HasherEventDuplicate::Remove => {
                            std::fs::remove_file(path).unwrap_or_else(|err| {
                                log::error!("Failed to remove file - (Reason: {err})")
                            })
                        }
                        HasherEventDuplicate::Rename => {
                            let pmv = format!("{}-{}", path.to_str().unwrap_or(""), hash.hash);
                            std::fs::rename(path, pmv).unwrap_or_else(|err| {
                                log::error!("Failed to rename file - (Reason: {err})")
                            })
                        }
                        HasherEventDuplicate::Print => {
                            println!(
                                "==============================================================="
                            );
                            println!("duplicate detected:",);
                            println!("=> {} ({})", other_path.display(), other_hash.hash);
                            println!("=> {} ({})", path.display(), hash.hash);
                        }
                    }
                    break;
                }
            }
            self.hashes.push(item);
        }
    }

    fn file_scan(&mut self, tx: Sender<Self::Item>, rx: Receiver<DirEntry>) {
        log::debug!("file_scan");
        rayon::spawn(move || {
            rx.into_iter().par_bridge().for_each(|entry| {
                let path = entry.path();
                match std::fs::read(path) {
                    Ok(ok) => {
                        let hash = xxhash_rust::xxh3::xxh3_128(&ok);
                        tx.send((
                            path.to_path_buf(),
                            Hashes {
                                hash,
                                size: ok.len(),
                            },
                        ))
                        .ok();
                    }
                    Err(err) => {
                        log::error!(
                            "Failed to read the contents of file: '{}' - {err}",
                            path.display()
                        );
                    }
                }
            });
            drop(tx);
        });
    }

    fn on_finish(&mut self) -> crate::Result<()> {
        log::debug!("finish");
        Ok(())
    }
}
