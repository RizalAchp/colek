use futures::channel::mpsc;
use tokio::{fs::DirEntry, io::AsyncWriteExt};

#[derive(Debug)]
pub struct AppDefault {
    writer: tokio::io::Stdout,
    tx: mpsc::UnboundedSender<Option<DirEntry>>,
    rx: mpsc::UnboundedReceiver<Option<DirEntry>>,
}

impl AppDefault {}

#[async_trait::async_trait]
impl super::App for AppDefault {
    async fn on_file_scan(&mut self, file: tokio::fs::DirEntry) {
        let path = format!("path: {}\n", file.path().display());
        self.writer.write(path.as_bytes()).await.ok();
        self.writer.flush().await.ok();
    }
    async fn new(_: &mut crate::system::SystemDiskInfo) -> anyhow::Result<Self> {
        let writer = tokio::io::stdout();
        let (tx, rx) = mpsc::unbounded();
        Ok(Self { tx, rx, writer })
    }
    async fn finish(mut self) -> anyhow::Result<()> {
        self.writer.flush().await?;
        self.writer.shutdown().await.map_err(From::from)
    }

    #[inline]
    fn tx(&self) -> mpsc::UnboundedSender<Option<DirEntry>> {
        self.tx.clone()
    }
    #[inline]
    fn rx(&mut self) -> &mut mpsc::UnboundedReceiver<Option<DirEntry>> {
        &mut self.rx
    }
}
