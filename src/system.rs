use std::path::PathBuf;
use sysinfo::{DiskExt, SystemExt};

use crate::err_log;

#[cfg(windows)]
const ROOT_DIR: &str = "C:";
#[cfg(windows)]
const BOOT_DIR: &str = "C:";
#[cfg(windows)]
fn is_generic_partition(part: &str) -> bool {
    true
}
#[cfg(unix)]
const ROOT_DIR: &str = "/";

#[cfg(unix)]
fn is_generic_partition(part: &str) -> bool {
    !(part.contains("efi") || part.contains("boot"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DriveType {
    Root,
    Generic,
    Removable,
    Boot,
}
impl DriveType {
    fn from_sysinfo_disk(disk: &sysinfo::Disk) -> Self {
        let p_str = disk.mount_point().to_str();
        if disk.is_removable() {
            DriveType::Removable
        } else if p_str == Some(ROOT_DIR) {
            #[cfg(windows)]
            {
                path = dirs::home_dir().unwrap_or(path);
                DriveType::Generic
            }
            #[cfg(unix)]
            DriveType::Root
        } else if p_str.is_some_and(is_generic_partition) {
            DriveType::Generic
        } else {
            DriveType::Boot
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiskPartition {
    pub tp: DriveType,
    pub name: String,
    pub path: PathBuf,
}

impl DiskPartition {
    fn from_sysinfo(part: &sysinfo::Disk) -> Self {
        let tp = DriveType::from_sysinfo_disk(part);
        let name = part.name().to_str().unwrap_or("").to_owned();
        let path = part.mount_point().to_path_buf();
        if log::max_level() >= log::LevelFilter::Info {
            let total = part.total_space() as f32 * 1e-9;
            let free = part.available_space() as f32 * 1e-9;
            let used = total - free;
            eprintln!(
                "{name:<10} {total:<10} {used:<10} {free:<10} '{mount}'",
                mount = path.display(),
            );
        }
        Self { tp, name, path }
    }
}

#[derive(Debug, Clone)]
pub struct SystemDiskInfo {
    pub name: Option<String>,
    pub kernel_version: Option<String>,
    pub os_version: Option<String>,
    pub host_name: Option<String>,
    pub default_filename: String,
    pub drives: Vec<DiskPartition>,
}

impl SystemDiskInfo {
    pub fn new() -> Self {
        let sys = sysinfo::System::new_all();
        if log::max_level() >= log::LevelFilter::Info {
            eprintln!("===================================================");
            eprintln!(
                "{device:<10} {total:<10} {used:<10} {free:<10} Mount",
                device = "Device",
                total = "Total, GB",
                used = "Used, GB",
                free = "Free, GB",
            );
        }
        let drives = sys
            .disks()
            .iter()
            .map(DiskPartition::from_sysinfo)
            .collect();
        eprintln!("===================================================");

        let name = sys.name();
        let kernel_version = sys.kernel_version();
        let os_version = sys.os_version();
        let host_name = sys.host_name();

        use crate::APP_NAME;
        let default_filename = match (&name, &host_name) {
            (Some(n), _) => format!("{APP_NAME}_{n}"),
            (None, Some(hn)) => format!("{APP_NAME}_{hn}"),
            _ => APP_NAME.to_owned(),
        };

        Self {
            name,
            kernel_version,
            os_version,
            host_name,
            default_filename,
            drives,
        }
    }

    pub fn dest(&mut self, out: Option<PathBuf>) -> PathBuf {
        let dest = match out {
            Some(out) => out,
            None => self.removable_drive().map_or_else(
                || {
                    log::error!("No Removeable Drive");
                    PathBuf::from(&self.default_filename)
                },
                |x| x.path.join(&self.default_filename),
            ),
        };

        if !dest.exists() {
            err_log!(
                std::fs::create_dir_all(&dest),
                "create_dir_all: {}",
                dest.display()
            );
        }
        dest
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
