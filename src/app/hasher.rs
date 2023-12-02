use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        mpsc::{Receiver, Sender},
        Arc,
    },
};

use ignore::DirEntry;
use rayon::prelude::{ParallelBridge, ParallelIterator};

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum HasherEventDuplicate {
    Remove,
    Rename,
    Print,
}
impl HasherEventDuplicate {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hashes {
    hash: u128,
    size: usize,
}

#[derive(Debug, Clone)]
pub struct AppHasher {
    event_duplicate: Arc<HasherEventDuplicate>,
    hashes: HashMap<Hashes, PathBuf>,
}
impl AppHasher {
    pub fn new(event_duplicate: HasherEventDuplicate) -> Self {
        Self {
            event_duplicate: Arc::new(event_duplicate),
            hashes: HashMap::new(),
        }
    }

    pub fn on_duplicate(
        &self,
        (src_hash, src_path): &(Hashes, PathBuf),
        (dup_hash, dup_path): (&Hashes, &PathBuf),
    ) -> crate::Result<()> {
        let src_path_created = src_path.metadata()?.created()?;
        let dup_path_created = dup_path.metadata()?.created()?;
        let (path, hash) = if src_path_created < dup_path_created {
            (src_path, src_hash)
        } else {
            (dup_path, dup_hash)
        };
        use HasherEventDuplicate as EV;
        match *self.event_duplicate {
            EV::Remove => std::fs::remove_file(path).map_err(From::from),
            EV::Rename => {
                let pmv = format!("{}-{}", path.to_str().unwrap_or(""), hash.hash);
                std::fs::rename(path, pmv).map_err(From::from)
            }
            EV::Print => {
                println!("==================== DUPLICATE ======================");
                println!("=> {} ({})", src_path.display(), src_hash.hash);
                println!("=> {} ({})", dup_path.display(), dup_hash.hash);
                Ok(println!())
            }
        }
    }
}

impl super::App for AppHasher {
    type Item = (Hashes, PathBuf);

    fn name() -> &'static str {
        "Hasher App"
    }
    fn on_blocking(&mut self, recver: Receiver<Self::Item>) -> crate::Result<()> {
        log::debug!("on_blocking");
        while let Ok(item) = recver.recv() {
            if let Some(find) = self.hashes.get_key_value(&item.0) {
                self.on_duplicate(&item, find)?
            }
            self.hashes.insert(item.0, item.1);
        }
        Ok(())
    }

    fn file_scan(&mut self, tx: Sender<Self::Item>, rx: Receiver<DirEntry>) -> crate::Result<()> {
        log::debug!("file_scan");
        let spawn = move || {
            let for_each_entry = |entry: DirEntry| {
                let path = entry.path();
                match std::fs::read(path) {
                    Ok(ok) => {
                        let hash = xxhash_rust::xxh3::xxh3_128(&ok);
                        tx.send((
                            Hashes {
                                hash,
                                size: ok.len(),
                            },
                            path.to_path_buf(),
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
            };
            rx.into_iter().par_bridge().for_each(for_each_entry);
            drop(tx);
        };
        rayon::spawn(spawn);
        Ok(())
    }

    fn on_finish(&mut self) -> crate::Result<()> {
        log::debug!("finish");
        Ok(())
    }
}
