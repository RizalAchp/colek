mod copy;
mod default;
mod zip;

pub use copy::AppCopy;
pub use default::AppDefault;
use futures::{
    channel::mpsc::{UnboundedReceiver, UnboundedSender},
    StreamExt,
};
use tokio::fs::DirEntry;
pub use zip::AppZip;

use async_trait::async_trait;

#[async_trait]
pub trait App: Sized {
    fn tx(&self) -> UnboundedSender<Option<DirEntry>>;
    fn rx(&mut self) -> &mut UnboundedReceiver<Option<DirEntry>>;

    async fn new(sys: &mut crate::system::SystemDiskInfo) -> anyhow::Result<Self>;
    async fn on_file_scan(&mut self, data: DirEntry);

    async fn run(&mut self, sys: &mut crate::system::SystemDiskInfo) -> anyhow::Result<()> {
        let tx = self.tx();
        log::info!("Receveing in app");
        let (ret, _) = tokio::join!(sys.file_scan(tx), async {
            while let Some(Some(data)) = self.rx().next().await {
                self.on_file_scan(data).await;
            }
        });
        ret
    }
    async fn finish(mut self) -> anyhow::Result<()> {
        Ok(())
    }
}
