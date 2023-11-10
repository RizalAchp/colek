use std::{
    path::Path,
    sync::{
        mpsc::{Receiver, Sender},
        Arc,
    },
};

use walkdir::DirEntry;

use super::CopyItem;

#[derive(Clone, Debug)]
pub struct AppCopy {
    dest: Arc<Path>,
}
impl AppCopy {
    pub fn new(dest: impl Into<Arc<Path>>) -> crate::Result<Self> {
        Ok(Self { dest: dest.into() })
    }
}

impl super::App for AppCopy {
    type Item = CopyItem;

    #[inline]
    fn name() -> &'static str {
        "Copy"
    }

    fn on_blocking(&mut self, rx: Receiver<Self::Item>) {
        while let Ok(CopyItem { dest, source }) = rx.recv() {
            match std::fs::copy(&source, &dest) {
                Ok(k) => log::info!(
                    "Success copying file from {path} into {dest} with size: {k} bytes",
                    path = source.display(),
                    dest = dest.display()
                ),
                Err(err) => log::error!(
                    "Failed to copy file `{path}` into `{dest}` - {err}",
                    path = source.display(),
                    dest = dest.display()
                ),
            }
        }
    }

    fn file_scan(&mut self, tx: Sender<Self::Item>, rx: Receiver<DirEntry>) {
        let dest = self.dest.clone();
        rayon::spawn(move || {
            let mut counter = 0;
            while let Ok(file) = rx.recv() {
                let path = file.path();
                let fname = path
                    .file_name()
                    .map(|x| x.to_string_lossy().to_string())
                    .unwrap_or_else(|| counter.to_string());
                let dest = dest.join(fname);
                let _ = tx
                    .send(CopyItem {
                        dest,
                        source: path.to_path_buf(),
                    })
                    .ok();

                counter += 1;
            }
            drop(tx)
        });
    }

    fn on_finish(&mut self) -> crate::Result<()> {
        Ok(())
    }
}
