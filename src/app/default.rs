use std::sync::{
    atomic::{AtomicUsize, Ordering},
    mpsc::{Receiver, Sender},
    Arc,
};

use walkdir::DirEntry;

#[derive(Debug, Clone)]
pub struct AppDefault {
    count: Arc<AtomicUsize>,
    size: Arc<AtomicUsize>,
}

impl AppDefault {
    pub fn new() -> crate::Result<Self> {
        Ok(Self {
            count: Arc::new(AtomicUsize::new(0)),
            size: Arc::new(AtomicUsize::new(0)),
        })
    }
}

impl super::App for AppDefault {
    type Item = ();
    fn file_scan(&mut self, _tx: Sender<Self::Item>, rx: Receiver<DirEntry>) {
        while let Ok(direntry) = rx.recv() {
            println!("path: {}", direntry.path().display());
            let s = direntry.metadata().map(|x| x.len()).unwrap_or(0) as usize;
            self.size.fetch_add(s, Ordering::Relaxed);
            self.count.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn on_finish(&mut self) -> crate::Result<()> {
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
