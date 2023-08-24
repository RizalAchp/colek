use std::{io::Write, sync::mpsc};

use walkdir::DirEntry;

#[derive(Debug)]
pub struct AppDefault {
    writer: std::io::Stdout,
    tx: mpsc::Sender<Option<DirEntry>>,
    rx: mpsc::Receiver<Option<DirEntry>>,
    count: usize,
    size: f64,
}

impl AppDefault {
    pub fn new() -> anyhow::Result<Self> {
        let writer = std::io::stdout();
        let (tx, rx) = mpsc::channel();
        Ok(Self {
            tx,
            rx,
            writer,
            count: 0,
            size: 0.0,
        })
    }
}

impl super::App for AppDefault {
    fn on_file_scan(&mut self, file: DirEntry) {
        let _ = writeln!(self.writer, "path: {}", file.path().display()).ok();
        self.size += file.metadata().map(|x| x.len()).unwrap_or(0) as f64;
        self.count += 1;
    }

    fn finish(&mut self) -> anyhow::Result<()> {
        writeln!(self.writer, "============= Finish Scanning =============")?;
        writeln!(self.writer, "size scanned            : {}", self.count)?;
        writeln!(
            self.writer,
            "all size bytes(KBytes)  : {} KB",
            self.size * 1e-3
        )?;
        writeln!(
            self.writer,
            "all size bytes(MBytes)  : {} MB",
            self.size * 1e-6
        )?;
        writeln!(
            self.writer,
            "all size bytes(GBytes)  : {} GB",
            self.size * 1e-9
        )?;
        Ok(())
    }

    #[inline]
    fn tx(&self) -> mpsc::Sender<Option<DirEntry>> {
        self.tx.clone()
    }
    #[inline]
    fn rx(&self) -> &mpsc::Receiver<Option<DirEntry>> {
        &self.rx
    }

    fn name() -> &'static str {
        "Stdout"
    }
}
