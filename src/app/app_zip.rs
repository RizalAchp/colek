use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::{Receiver, Sender},
        Arc,
    },
};

use walkdir::DirEntry;

use super::CopyItem;

#[derive(Clone)]
pub struct AppZip {
    dir_path: Arc<Path>,
    zipfilepath: Arc<Path>,
    counter: Arc<AtomicUsize>,
}
impl AppZip {
    pub fn new(zipfilepath: PathBuf) -> crate::Result<Self> {
        let dest = zipfilepath.with_extension("");
        std::fs::create_dir_all(&dest)?;
        Ok(Self {
            dir_path: dest.into(),
            zipfilepath: zipfilepath.into(),
            counter: Arc::new(AtomicUsize::new(0)),
        })
    }
}

impl super::App for AppZip {
    type Item = CopyItem;

    fn name() -> &'static str {
        "Zip"
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
        let counter = self.counter.clone();
        let dirpath = self.dir_path.clone();
        rayon::spawn(move || {
            while let Ok(file) = rx.recv() {
                let path = file.path();
                let c = counter.fetch_add(1, Ordering::Relaxed);
                let fname = path
                    .file_name()
                    .map(|x| x.to_string_lossy().to_string())
                    .unwrap_or_else(|| c.to_string());
                let dest = dirpath.join(fname);
                let _ = tx
                    .send(CopyItem {
                        dest,
                        source: path.to_path_buf(),
                    })
                    .ok();
            }

            drop(tx);
        });
    }

    fn on_finish(&mut self) -> crate::Result<()> {
        let file = File::create(&self.zipfilepath)?;
        let mut writer = zip::ZipWriter::new(file);

        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);
        // Recursively walk through the directory and add its contents to the ZIP archive.
        for entry in walkdir::WalkDir::new(&self.dir_path) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let relative_path = entry
                    .path()
                    .strip_prefix(&self.dir_path)
                    .map_err(|x| x.to_string())?;
                writer.start_file(relative_path.to_string_lossy(), options)?;
                let mut file = File::open(entry.path())?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                writer.write_all(&buffer)?;
            }
        }
        writer.finish()?;

        todo!();
    }
}
