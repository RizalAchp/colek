use std::path::PathBuf;

use futures::channel::mpsc;
use tokio::fs::DirEntry;

pub struct AppCopy {
    tx: mpsc::UnboundedSender<Option<DirEntry>>,
    rx: mpsc::UnboundedReceiver<Option<DirEntry>>,
    dest: PathBuf,
}

#[async_trait::async_trait]
impl super::App for AppCopy {
    #[inline]
    fn tx(&self) -> mpsc::UnboundedSender<Option<DirEntry>> {
        self.tx.clone()
    }
    #[inline]
    fn rx(&mut self) -> &mut mpsc::UnboundedReceiver<Option<DirEntry>> {
        &mut self.rx
    }

    async fn on_file_scan(&mut self, file: tokio::fs::DirEntry) {
        use tokio::fs;
        let path = file.path();
        let dest = self.dest.join(
            path.parent()
                .map(std::path::Path::to_path_buf)
                .unwrap_or_else(|| {
                    log::error!("Failed to get Parrent dir for {}", path.display());
                    PathBuf::new()
                }),
        );
        match fs::copy(&path, &dest).await {
            Ok(_) => (),
            Err(err) => log::error!(
                "Failed to copy file `{}` into `{}`, {err}",
                path.display(),
                dest.display()
            ),
        }
    }
    async fn new(sys: &mut crate::system::SystemDiskInfo) -> anyhow::Result<Self> {
        let dest = sys.dest();
        let (tx, rx) = mpsc::unbounded();

        Ok(Self { tx, rx, dest })
    }
}
