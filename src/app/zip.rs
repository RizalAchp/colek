use async_zip::tokio::write::ZipFileWriter;
use futures::channel::mpsc;
use futures::AsyncWriteExt;
use tokio::fs::{DirEntry, File};

pub struct AppZip {
    writer: ZipFileWriter<File>,
    tx: mpsc::UnboundedSender<Option<DirEntry>>,
    rx: mpsc::UnboundedReceiver<Option<DirEntry>>,
    counter: usize,
}

#[async_trait::async_trait]
impl super::App for AppZip {
    #[inline]
    fn tx(&self) -> mpsc::UnboundedSender<Option<DirEntry>> {
        self.tx.clone()
    }
    #[inline]
    fn rx(&mut self) -> &mut mpsc::UnboundedReceiver<Option<DirEntry>> {
        &mut self.rx
    }

    async fn on_file_scan(&mut self, file: DirEntry) {
        let ins = std::time::Instant::now();

        let path = file.path();
        let Some(filename) = path.file_name().map(|x| x.to_string_lossy().to_string()) else { return; };
        let data = match tokio::fs::read(&path).await {
            Ok(ok) => ok,
            Err(err) => {
                log::error!("Faile to read data from `{}` - {err}", path.display());
                return;
            }
        };
        println!("size: {} kb", data.len() / 1000);

        let entry =
            async_zip::ZipEntryBuilder::new(filename.into(), async_zip::Compression::Zstd).build();
        match self.writer.write_entry_whole(entry, &data).await {
            Ok(()) => self.counter += 1,
            Err(err) => log::error!("Faile to read data from `{}` - {err}", path.display()),
        }

        log::info!(
            "Zipping file: `{}` - {} s",
            path.display(),
            ins.elapsed().as_secs_f32()
        )
    }

    async fn new(sys: &mut crate::system::SystemDiskInfo) -> anyhow::Result<Self> {
        let dest = sys.dest();
        let file = File::create(dest).await?;
        let writer = ZipFileWriter::with_tokio(file);
        let (tx, rx) = mpsc::unbounded();
        Ok(Self {
            writer,
            tx,
            rx,
            counter: 0,
        })
    }

    async fn finish(mut self) -> anyhow::Result<()> {
        let mut writer = self.writer.close().await?;
        writer.flush().await?;
        writer.close().await?;
        Ok(())
    }
}
