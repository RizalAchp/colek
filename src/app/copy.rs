use std::{path::PathBuf, sync::mpsc};

use walkdir::DirEntry;

pub struct AppCopy {
    tx: mpsc::Sender<Option<DirEntry>>,
    rx: mpsc::Receiver<Option<DirEntry>>,
    dest: PathBuf,
}
impl AppCopy {
    pub fn new(dest: PathBuf) -> anyhow::Result<Self> {
        let (tx, rx) = mpsc::channel();
        Ok(Self { tx, rx, dest })
    }
}

impl super::App for AppCopy {
    #[inline]
    fn name() -> &'static str {
        "Copy"
    }

    #[inline]
    fn tx(&self) -> mpsc::Sender<Option<DirEntry>> {
        self.tx.clone()
    }
    #[inline]
    fn rx(&self) -> &mpsc::Receiver<Option<DirEntry>> {
        &self.rx
    }

    fn on_file_scan(&mut self, file: DirEntry) {
        let path = file.path();
        let Some(parent) = path.parent().map(std::path::Path::to_path_buf) else {
            return;
        };
        let dest = self.dest.join(parent);
        match std::fs::copy(path, &dest) {
            Ok(k) => log::info!(
                "Success copying file from {path} into {dest} with size: {k} bytes",
                path = path.display(),
                dest = dest.display()
            ),
            Err(err) => log::error!(
                "Failed to copy file `{path}` into `{dest}` - {err}",
                path = path.display(),
                dest = dest.display()
            ),
        }
    }

    fn finish(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}
