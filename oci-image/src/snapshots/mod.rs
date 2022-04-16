pub mod overlay;

use crate::Result;
use std::path::{Path, PathBuf};

/// The `Snapshotter` trait defines methods that can be implemented in order to create a container image snapshot.
pub trait Snapshotter: Send + Sync {
    fn mount(
        &mut self,
        layers: Vec<String>,
        mount_path: &Path,
        index: &usize,
        read_only: bool,
    ) -> Result<MountPoint>;
    fn umount(&self, mount_point: &MountPoint) -> Result<()>;
}

/// `MountPoint` holds information about a mount point on the host.
#[derive(Debug)]
pub struct MountPoint {
    /// The FS type for the mount point
    #[allow(dead_code)]
    pub typ: String,
    /// The mount destination path
    #[allow(dead_code)]
    pub mount_path: PathBuf,
}
