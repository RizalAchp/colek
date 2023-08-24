use std::{path::PathBuf, sync::mpsc};
use sysinfo::{DiskExt, SystemExt};

use crate::filters::FilterFn;

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

pub struct SystemDiskInfo {
    pub name: Option<String>,
    pub kernel_version: Option<String>,
    pub os_version: Option<String>,
    pub host_name: Option<String>,
    pub drives: Vec<DiskPartition>,
}

impl SystemDiskInfo {
    pub fn new() -> Self {
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
        }
    }

    pub fn dest(&mut self, out: Option<PathBuf>) -> PathBuf {
        let dest = self
            .removable_drive()
            .map(|x| x.path.clone())
            .unwrap_or_else(|| {
                log::error!("No Removeable Drive, Defaulting in current location");
                PathBuf::from("./")
            });

        let out = if let Some(ref out) = out {
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
}

pub fn file_scan(
    drives: Vec<DiskPartition>,
    filter: Box<FilterFn>,
    tx: mpsc::Sender<Option<walkdir::DirEntry>>,
) -> anyhow::Result<()> {
    for drive in drives {
        log::info!("Process search file in drive: {}", drive.name);
        for file in walkdir::WalkDir::new(&drive.path)
            .into_iter()
            .filter_map(|res| match res {
                Ok(ok) if filter(&ok) => Some(ok),
                Err(err) => {
                    log::error!("Error on scanning dir: {err}");
                    None
                }
                _ => None,
            })
        {
            tx.send(Some(file)).ok();
        }
    }
    let _ = tx.send(None).ok();
    Ok(())
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
