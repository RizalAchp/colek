mod app_zip;
mod copy;
mod default;

use std::sync::mpsc::{Receiver, Sender};

pub use app_zip::AppZip;
pub use copy::AppCopy;
pub use default::AppDefault;

use crate::filters::FilterFn;

pub trait App: Sized {
    fn name() -> &'static str;
    fn tx(&self) -> Sender<Option<walkdir::DirEntry>>;
    fn rx(&self) -> &Receiver<Option<walkdir::DirEntry>>;
    fn on_file_scan(&mut self, data: walkdir::DirEntry);
    fn finish(&mut self) -> anyhow::Result<()>;

    fn run(
        &mut self,
        sys: &mut crate::system::SystemDiskInfo,
        filter: crate::Filter,
    ) -> anyhow::Result<()> {
        log::info!("Running an App: {}", Self::name());
        let tx = self.tx();
        let Some(drives) = sys.generic_drive() else {
            anyhow::bail!("No Generic Drive detected in this computer!");
        };

        let handle = std::thread::spawn(|| {
            let filter = match filter {
                crate::Filter::Image => Box::new(crate::filters::is_images) as Box<FilterFn>,
                crate::Filter::Video => Box::new(crate::filters::is_videos) as Box<FilterFn>,
                crate::Filter::Music => Box::new(crate::filters::is_music) as Box<FilterFn>,
                crate::Filter::Other {
                    ignorecase,
                    name,
                    extension,
                } => Box::new(move |a: &'_ walkdir::DirEntry| -> bool {
                    let path = a.path();
                    let has_name = if let Some(ref name) = name {
                        let name = is_ignorecase(ignorecase, name);
                        path.file_name()
                            .map(|x| is_ignorecase(ignorecase, x.to_string_lossy()))
                            .is_some_and(|x| x.contains(&name))
                    } else {
                        true
                    };
                    let has_extension = if let Some(ref ext) = extension {
                        path.extension()
                            .map(|x| is_ignorecase(ignorecase, x.to_string_lossy()))
                            .is_some_and(|x| x == is_ignorecase(ignorecase, ext))
                    } else {
                        false
                    };

                    has_extension && has_name
                }) as Box<FilterFn>,
            };
            crate::system::file_scan(drives, filter, tx)
        });

        while let Ok(Some(data)) = self.rx().recv() {
            self.on_file_scan(data);
        }

        handle
            .join()
            .map_err(|err| anyhow::anyhow!("Failed to join thread: {err:?}"))??;

        self.finish()
    }
}

#[inline]
fn is_ignorecase(is: bool, inp: impl AsRef<str>) -> String {
    if is {
        inp.as_ref().to_lowercase()
    } else {
        inp.as_ref().to_owned()
    }
}
