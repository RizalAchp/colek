use std::{
    path::Path,
    sync::{
        mpsc::{Receiver, Sender},
        Arc,
    },
};

use ignore::DirEntry;

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
    type Item = usize;

    #[inline]
    fn name() -> &'static str {
        "Copy"
    }

    fn on_blocking(&mut self, rx: Receiver<Self::Item>) -> crate::Result<()> {
        let mut counter = 0;
        while let Ok(c) = rx.recv() {
            counter = c;
        }

        log::info!("Coping {counter} file(s)");
        Ok(())
    }

    fn file_scan(&mut self, tx: Sender<Self::Item>, rx: Receiver<DirEntry>) -> crate::Result<()> {
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

                match std::fs::copy(path, &dest) {
                    Ok(k) => {
                        log::info!(
                            "Success copying file from {path} into {dest} with size: {k} bytes",
                            path = path.display(),
                            dest = dest.display()
                        );
                        counter += 1;
                        tx.send(counter).ok();
                    }
                    Err(err) => log::error!(
                        "Failed to copy file `{path}` into `{dest}` - {err}",
                        path = path.display(),
                        dest = dest.display()
                    ),
                }
            }
            drop(tx)
        });

        Ok(())
    }

    fn on_finish(&mut self) -> crate::Result<()> {
        Ok(())
    }
}
