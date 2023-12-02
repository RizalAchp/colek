use std::{
    io::{stdout, BufWriter, Stdout, Write},
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::{Receiver, Sender},
        Arc,
    },
};

use ignore::DirEntry;

#[derive(Debug)]
pub struct AppDefault {
    stdout: BufWriter<Stdout>,
    count: Arc<AtomicUsize>,
    size: Arc<AtomicUsize>,
}

impl AppDefault {
    pub fn new() -> crate::Result<Self> {
        Ok(Self {
            stdout: BufWriter::with_capacity(50 << 10, stdout()),
            count: Arc::new(AtomicUsize::new(0)),
            size: Arc::new(AtomicUsize::new(0)),
        })
    }
}

impl super::App for AppDefault {
    type Item = PathBuf;
    fn file_scan(&mut self, tx: Sender<Self::Item>, rx: Receiver<DirEntry>) -> crate::Result<()> {
        log::debug!("{}: on FileScan", Self::name());
        let size = self.size.clone();
        let count = self.count.clone();
        rayon::spawn(move || {
            while let Ok(direntry) = rx.recv() {
                let s = direntry.metadata().map(|x| x.len()).unwrap_or(0) as usize;
                size.fetch_add(s, Ordering::Relaxed);
                count.fetch_add(1, Ordering::Relaxed);
                tx.send(direntry.path().to_path_buf()).ok();
            }
        });
        Ok(())
    }

    fn on_blocking(&mut self, rx: Receiver<Self::Item>) -> crate::Result<()> {
        log::debug!("{}: on Blocking", Self::name());
        while let Ok(direntry) = rx.recv() {
            writeln!(self.stdout, "path: '{}'", direntry.display()).ok();
        }
        Ok(())
    }

    fn on_finish(&mut self) -> crate::Result<()> {
        self.stdout.flush().ok();
        println!("============= Finish Scanning =============");
        println!(
            "size scanned            : {}",
            self.count.load(Ordering::Relaxed)
        );
        let file_size_kb = self.size.load(Ordering::Relaxed) as f32 / 1024.0;
        let file_size_mb = file_size_kb / 1024.0;
        let file_size_gb = file_size_mb / 1024.0;
        println!("all size bytes(KBytes)  : {:.2} KB", file_size_kb);
        println!("all size bytes(MBytes)  : {:.2} MB", file_size_mb);
        println!("all size bytes(GBytes)  : {:.2} GB", file_size_gb);
        Ok(())
    }

    fn name() -> &'static str {
        "Stdout"
    }
}
