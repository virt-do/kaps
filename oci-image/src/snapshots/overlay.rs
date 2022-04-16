use crate::snapshots::{MountPoint, Snapshotter};
use crate::{Error, Result};
use nix::mount::MsFlags;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

/// `OverlayFS` is used to easily mount and unmount
/// overlay file systems.
#[derive(Debug)]
pub struct OverlayFS {
    pub data_dir: PathBuf,
}

impl Snapshotter for OverlayFS {
    /// Create a mount point with OverlayFS.
    ///
    /// This will ensure all directories required by the mount are created, and if not it'll create them.
    /// Once ready, the method will mount all layers into the mount_path.
    fn mount(
        &mut self,
        layers: Vec<String>,
        mount_path: &Path,
        index: &usize,
        read_only: bool,
    ) -> Result<MountPoint> {
        let layers = layers.iter().map(|l| l.as_str()).collect::<Vec<&str>>();
        let workdir = self.data_dir.join(index.to_string());

        // OverlayFS configuration
        let overlay_lowerdir = layers.join(":");
        let overlay_upperdir = workdir.join("upperdir");
        let overlay_workdir = workdir.join("workdir");

        if !self.data_dir.exists() {
            log::debug!(
                "creating OverlayFS data directory = {}",
                &self.data_dir.display()
            );
            create_dir_all(&self.data_dir)
                .map_err(|e| Error::OverlayFSCreateDirectory(e.to_string()))?;
        }
        create_dir_all(&overlay_upperdir)
            .map_err(|e| Error::OverlayFSCreateDirectory(e.to_string()))?;
        create_dir_all(&overlay_workdir)
            .map_err(|e| Error::OverlayFSCreateDirectory(e.to_string()))?;

        if !mount_path.exists() {
            log::debug!("creating overlayfs mount path = {}", mount_path.display());
            create_dir_all(mount_path)
                .map_err(|e| Error::OverlayFSCreateDirectory(e.to_string()))?;
        }

        let source = Path::new("overlay");
        let flags = match read_only {
            true => MsFlags::MS_RDONLY,
            false => MsFlags::empty(),
        };
        let options = format!(
            "lowerdir={},upperdir={},workdir={}",
            &overlay_lowerdir,
            overlay_upperdir.display(),
            overlay_workdir.display()
        );

        log::debug!("mounting layers into {}", &mount_path.display());

        nix::mount::mount(
            Some(source),
            mount_path,
            Some("overlay"),
            flags,
            Some(options.as_str()),
        )
        .map_err(|e| Error::OverlayFSMount(e.to_string()))?;

        log::debug!("new overlay mountpoint at {}", &mount_path.display());

        Ok(MountPoint {
            typ: "overlay".to_string(),
            mount_path: mount_path.to_path_buf(),
        })
    }

    /// Execute a `umount` sys call on the `mount_point`.
    #[allow(dead_code)]
    fn umount(&self, mount_point: &MountPoint) -> Result<()> {
        nix::mount::umount(mount_point.mount_path.as_path()).map_err(Error::OverlayFSUmount)?;
        log::debug!(
            "successfully unmounted = {}",
            &mount_point.mount_path.display()
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // TODO uncomment this when privileges issue solved

    // use crate::snapshots::OverlayFS;
    // use std::env::temp_dir;
    // use std::fs::{create_dir_all, remove_dir_all, File};
    // use std::path::PathBuf;

    // #[test]
    // fn test_mount_overlayfs() {
    //     let tmp = temp_dir().join("kaps_mount_tests");
    //
    //     let dir_a = tmp.join("a");
    //     let dir_b = tmp.join("b");
    //
    //     let file_a = dir_a.join("a.txt");
    //     let file_b = dir_b.join("b.txt");
    //
    //     create_dir_all(&file_a.parent().unwrap()).expect("failed to create file a hierarchy");
    //     create_dir_all(&file_b.parent().unwrap()).expect("failed to create file b hierarchy");
    //
    //     File::create(&file_a).expect("failed to create file a");
    //     File::create(&file_b).expect("failed to create file b");
    //
    //     assert!(&file_a.exists());
    //     assert!(&file_b.exists());
    //
    //     let mut layers = Vec::<PathBuf>::new();
    //     layers.push(dir_a.clone());
    //     layers.push(dir_b.clone());
    //
    //     let mount_path = tmp.join("rootfs");
    //     let mount = OverlayFS::mount(layers, &mount_path, &tmp.join("overlayfs"));
    //
    //     assert!(mount.is_ok());
    //
    //     assert!(mount_path.join("a").join("a.txt").exists());
    //     assert!(mount_path.join("b").join("b.txt").exists());
    //
    //     // Clean up
    //     remove_dir_all(&tmp).expect(format!("failed to clean up {}", tmp.display()).as_str())
    // }
}
