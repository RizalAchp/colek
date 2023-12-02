use std::{
    fs::File,
    io::{self, BufReader, BufWriter},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::{Receiver, Sender},
        Arc,
    },
};

use ignore::DirEntry;

#[derive(Clone)]
pub struct AppZip {
    zipfilepath: Arc<Path>,
    counter: Arc<AtomicUsize>,
}
impl AppZip {
    pub fn new(zipfilepath: PathBuf) -> crate::Result<Self> {
        let Some(dest) = zipfilepath.parent() else {
            return Err(crate::error::ColekError::Err(format!(
                "failed to get parrent path: '{}' - path terminates in root",
                zipfilepath.display()
            )));
        };
        std::fs::create_dir_all(dest)?;
        Ok(Self {
            zipfilepath: zipfilepath.into(),
            counter: Arc::new(AtomicUsize::new(0)),
        })
    }
}

impl super::App for AppZip {
    type Item = u64;

    fn name() -> &'static str {
        "Zip"
    }

    fn on_blocking(&mut self, rx: Receiver<Self::Item>) -> crate::Result<()> {
        while let Ok(copied) = rx.recv() {
            log::info!("Copied file into Zip Archive: {copied} bytes")
        }
        Ok(())
    }

    fn file_scan(&mut self, tx: Sender<Self::Item>, rx: Receiver<DirEntry>) -> crate::Result<()> {
        let counter = self.counter.clone();
        let mut writer = zip::ZipWriter::new(BufWriter::new(File::create(&self.zipfilepath)?));
        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);

        rayon::spawn(move || {
            while let Ok(file) = rx.recv() {
                let path = file.path();
                let c = counter.fetch_add(1, Ordering::Relaxed);
                let fname = path
                    .file_name()
                    .map(|x| x.to_string_lossy().to_string())
                    .unwrap_or_else(|| c.to_string());
                let dest = PathBuf::from(fname);
                let source = path;

                let Ok(_) = writer.start_file(dest.to_string_lossy(), options) else {
                    continue;
                };
                if let Ok(file) = File::open(source) {
                    let mut file = BufReader::new(file);
                    match io::copy(&mut file, &mut writer) {
                        Ok(ok) => {
                            tx.send(ok).ok();
                        }
                        Err(err) => {
                            log::error!(
                                "Failed to copy from '{}' - (Reason: {err})",
                                source.display()
                            );
                        }
                    }
                }
            }

            drop(tx);
            writer.finish().ok();
        });

        Ok(())
    }

    fn on_finish(&mut self) -> crate::Result<()> {
        Ok(())
    }
}
