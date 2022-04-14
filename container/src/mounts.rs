use crate::Error;
use std::path::PathBuf;
use std::process::Command;

/// Implementation of the OCI `Mount`.
#[derive(Clone)]
struct Mount {
    typ: String,
    source: String,
    destination: String,
}

#[derive(Clone)]
pub struct Mounts {
    mounts: Vec<Mount>,
}

impl Mounts {
    /// Apply some mounts.
    /// This method should be called before the container process execution in order to prepare
    /// & mount every mounts defined for it.
    pub fn apply(&self) -> Result<(), std::io::Error> {
        for mount in &self.mounts {
            if let Some(code) = Command::new("mount")
                .args(["-t", &mount.typ, &mount.source, &mount.destination])
                .status()?
                .code()
            {
                if code != 0 {
                    return Err(std::io::Error::from_raw_os_error(code));
                }
            }
        }
        Ok(())
    }

    /// Cleanup the mounts of a rootfs.
    /// This method should be called when a container has ended, to clean up the FS.
    pub fn cleanup(&self, rootfs: PathBuf) -> Result<(), crate::Error> {
        for mount in &self.mounts {
            let mut path = rootfs.clone();
            path.push(&mount.source);

            if let Some(code) = Command::new("umount")
                .args([path])
                .status()
                .map_err(Error::Unmount)?
                .code()
            {
                if code != 0 {
                    return Err(crate::Error::Unmount(std::io::Error::from_raw_os_error(
                        code,
                    )));
                }
            }
        }

        Ok(())
    }
}

impl Default for Mounts {
    /// Returns the default mounts for a container.
    /// Based on the OCI Specification
    fn default() -> Self {
        Mounts {
            mounts: vec![
                Mount {
                    typ: String::from("devtmpfs"),
                    source: String::from("dev"),
                    destination: String::from("/dev"),
                },
                Mount {
                    typ: String::from("proc"),
                    source: String::from("proc"),
                    destination: String::from("/proc"),
                },
                Mount {
                    typ: String::from("sysfs"),
                    source: String::from("sys"),
                    destination: String::from("/sys"),
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::mounts::Mount;
    use crate::Mounts;
    use proc_mounts::MountList;
    use std::path::PathBuf;

    #[test]
    fn test_mount_apply_and_cleanup() -> Result<(), std::io::Error> {
        let test_folders = vec![
            "/tmp/kaps-test_a",
            "/tmp/kaps-test_b",
            "/tmp/kaps-test_c",
            "/tmp/kaps-test_d",
        ];
        for folder in &test_folders {
            std::fs::create_dir_all(folder)?;
        }

        let host_mounts_before_apply = MountList::new().unwrap();
        let mounts = Mounts {
            mounts: vec![
                Mount {
                    typ: String::from("devtmpfs"),
                    source: String::from("tmp/kaps-test_a"),
                    destination: String::from("/tmp/kaps-test_b"),
                },
                Mount {
                    typ: String::from("devtmpfs"),
                    source: String::from("tmp/kaps-test_c"),
                    destination: String::from("/tmp/kaps-test_d"),
                },
            ],
        };
        mounts.apply()?;

        let host_mounts_after_apply = MountList::new().unwrap();
        assert_ne!(host_mounts_before_apply, host_mounts_after_apply);

        mounts.cleanup(PathBuf::from("")).unwrap();
        let host_mounts_after_cleanup = MountList::new().unwrap();
        assert_eq!(host_mounts_before_apply, host_mounts_after_cleanup);

        for folder in &test_folders {
            std::fs::remove_dir_all(folder)?;
        }
        Ok(())
    }
}
