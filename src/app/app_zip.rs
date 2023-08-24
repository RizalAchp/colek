use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    sync::mpsc,
};

use walkdir::DirEntry;
use zip::write::FileOptions;

pub struct AppZip {
    writer: zip::ZipWriter<File>,
    tx: mpsc::Sender<Option<DirEntry>>,
    rx: mpsc::Receiver<Option<DirEntry>>,
    buffer: Vec<u8>,
    counter: usize,
}
impl AppZip {
    pub fn new(dest: PathBuf) -> anyhow::Result<Self> {
        let file = File::create(dest)?;
        let writer = zip::ZipWriter::new(file);
        let (tx, rx) = mpsc::channel();
        Ok(Self {
            writer,
            tx,
            rx,
            buffer: Vec::with_capacity(2048),
            counter: 0,
        })
    }
}

impl super::App for AppZip {
    fn name() -> &'static str {
        "Zip"
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
        let ins = std::time::Instant::now();
        let path = file.path();
        let Some(filename) = path.file_name().map(|x| x.to_string_lossy()) else { return; };

        if let Err(err) = self
            .writer
            .start_file(filename, FileOptions::default().compression_level(Some(9)))
        {
            log::error!("Failed on creating file in zipfile: {err}");
            return;
        }

        self.buffer.clear();
        match File::open(path).and_then(|mut file| file.read_to_end(&mut self.buffer)) {
            Ok(ok) => log::info!(
                "Success reading file from {path} - {ok} bytes",
                path = path.display()
            ),
            Err(err) => {
                log::error!(
                    "Failed to read file from {path} - {err}",
                    path = path.display()
                );
                return;
            }
        };

        match self.writer.write_all(&self.buffer) {
            Ok(_) => (),
            Err(err) => {
                log::error!(
                    "Failed to werite content file from {path} into zipfile - {err}",
                    path = path.display()
                );
                return;
            }
        }
        self.counter += 1;
        log::info!(
            "{}: Zipping file: `{}` - {} s",
            self.counter,
            path.display(),
            ins.elapsed().as_secs_f32()
        );
    }

    fn finish(&mut self) -> anyhow::Result<()> {
        self.buffer.clear();
        self.writer.finish()?;
        Ok(())
    }
}
