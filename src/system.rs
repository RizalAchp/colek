use futures::channel::mpsc;
use std::{path::PathBuf, sync::Arc};
use sysinfo::{DiskExt, SystemExt};
use tokio::fs::DirEntry;

use crate::{dir, filters::FilterFn, Filter};

#[cfg(windows)]
const ROOT_DIR: &str = "C:";
#[cfg(windows)]
const BOOT_DIR: &str = "C:";
#[cfg(unix)]
const ROOT_DIR: &str = "/";
#[cfg(unix)]
fn is_generic_partition(part: &str) -> bool {
    !matches!(part, "/boot" | "/efi")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DriveType {
    Root,
    Generic,
    Removable,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiskPartition {
    pub tp: DriveType,
    pub name: String,
    pub path: PathBuf,
}

impl DiskPartition {
    fn from_heim_partition(part: &sysinfo::Disk) -> Option<Self> {
        let name = part.name().to_string_lossy().to_string();
        let path = part.mount_point().to_path_buf();
        let p_str = path.to_str();
        let tp = if part.is_removable() {
            DriveType::Removable
        } else if p_str == Some(ROOT_DIR) {
            DriveType::Root
        } else if p_str.is_some_and(is_generic_partition) {
            DriveType::Generic
        } else {
            return None;
        };

        #[cfg(debug_assertions)]
        {
            let total = part.total_space() as f32 * 1e-9;
            let free = part.available_space() as f32 * 1e-9;
            let used = total - free;

            println!(
                "{name:<25} {total:<12} {used:<12} {free:<12} {mount}",
                mount = path.display(),
            );
        }

        Some(Self { tp, name, path })
    }
}

#[derive(Clone)]
struct WithFilterCall<F> {
    fp: F,
}

impl<F> WithFilterCall<F>
where
    F: Fn(&DirEntry) -> bool + Send + Sync + 'static,
{
    pub fn new(fp: F) -> Self {
        WithFilterCall { fp }
    }

    pub fn call(&self, dir: &DirEntry) -> bool {
        (self.fp)(dir)
    }
}

pub struct SystemDiskInfo {
    pub name: Option<String>,
    pub kernel_version: Option<String>,
    pub os_version: Option<String>,
    pub host_name: Option<String>,
    pub drives: Vec<DiskPartition>,

    output: Option<PathBuf>,
    filter: Arc<WithFilterCall<Box<FilterFn>>>,
}

impl SystemDiskInfo {
    pub fn new(filter: crate::Filter) -> Self {
        let sys = sysinfo::System::new_all();
        #[cfg(debug_assertions)]
        {
            println!(
                "{device:<25} {total:<12} {used:<12} {free:<12} Mount",
                device = "Device",
                total = "Total, GB",
                used = "Used, GB",
                free = "Free, GB",
            );
        }

        let name = sys.name();
        let kernel_version = sys.kernel_version();
        let os_version = sys.os_version();
        let host_name = sys.host_name();

        let drives = sys
            .disks()
            .iter()
            .filter_map(DiskPartition::from_heim_partition)
            .collect();

        Self {
            name,
            kernel_version,
            os_version,
            host_name,
            drives,
            output: None,
            filter: Arc::new(WithFilterCall::new(match filter {
                Filter::Image => Box::new(crate::filters::is_images) as Box<FilterFn>,
                Filter::Video => Box::new(crate::filters::is_videos) as Box<FilterFn>,
                Filter::Music => Box::new(crate::filters::is_images_and_videos) as Box<FilterFn>,
                Filter::Other {
                    ignorecase,
                    name,
                    extension,
                } => Box::new(move |a: &'_ tokio::fs::DirEntry| -> bool {
                    let path = a.path();
                    let has_name = if let Some(ref name) = &name {
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
            })),
        }
    }

    pub fn with_output(mut self, out: Option<PathBuf>) -> Self {
        self.output = out;
        self
    }

    pub fn dest(&mut self) -> PathBuf {
        let dest = self
            .removable_drive()
            .map(|x| x.path.clone())
            .unwrap_or_else(|| {
                log::error!("No Removeable Drive, Defaulting in current location");
                PathBuf::from("./")
            });

        let out = if let Some(ref out) = self.output {
            out.clone()
        } else {
            PathBuf::from(format!("{}_{}.zip", crate::APP_NAME, self))
        };

        if !dest.exists() {
            out
        } else {
            dest.join(out)
        }
    }

    #[inline]
    #[allow(unused)]
    pub fn root_drive(&mut self) -> Option<DiskPartition> {
        self.drives
            .iter()
            .find(|item| matches!(item.tp, DriveType::Root))
            .cloned()
    }

    #[inline]
    #[allow(unused)]
    pub fn generic_drive(&mut self) -> Option<Vec<DiskPartition>> {
        let gen_drve: Vec<_> = self
            .drives
            .iter()
            .filter_map(|item| {
                if matches!(item.tp, DriveType::Generic) {
                    Some(item.clone())
                } else {
                    None
                }
            })
            .collect();
        if gen_drve.is_empty() {
            None
        } else {
            Some(gen_drve)
        }
    }

    pub fn removable_drive(&mut self) -> Option<DiskPartition> {
        self.drives
            .iter()
            .find(|item| matches!(item.tp, DriveType::Removable))
            .cloned()
    }

    #[allow(unused)]
    pub async fn file_scan(
        &mut self,
        tx: mpsc::UnboundedSender<Option<DirEntry>>,
    ) -> anyhow::Result<()> {
        use futures::StreamExt;
        let Some(drives) = self.generic_drive() else { return Err(anyhow::anyhow!("Failed to get Generics Drive")); };

        for drive in drives {
            log::info!("Process search file in drive: {}", drive.name);
            let mut visit_stream = dir::WalkDir::new(&drive.path);
            while let Some(entry) = visit_stream.next().await {
                match entry {
                    Ok(dir) if self.filter.call(&dir) => _ = tx.unbounded_send(Some(dir)),
                    Ok(_) => continue,
                    Err(err) => log::error!("{err}"),
                }
            }
        }
        tx.unbounded_send(None);
        Ok(())
    }
}
impl std::fmt::Display for SystemDiskInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref name) = self.name {
            write!(f, "{}-", name.replace(' ', "-"))?;
        }
        if let Some(ref host_name) = self.host_name {
            write!(f, "{}", host_name.replace(' ', "-"))?;
        }
        Ok(())
    }
}

#[inline]
fn is_ignorecase(is: bool, inp: impl AsRef<str>) -> String {
    if is {
        inp.as_ref().to_ascii_lowercase()
    } else {
        inp.as_ref().to_owned()
    }
}
