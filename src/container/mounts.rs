use crate::container::Error;
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
    vec: Vec<Mount>,
}

impl Mounts {
    /// Apply some mounts.
    /// This method should be called before the container process execution in order to prepare
    /// & mount every mounts defined for it.
    pub fn apply(mounts: &Mounts) -> Result<(), std::io::Error> {
        for mount in &mounts.vec {
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
    pub fn cleanup(&self, rootfs: PathBuf) -> Result<(), crate::container::Error> {
        for mount in &self.vec {
            let mut path = rootfs.clone();
            path.push(&mount.source);

            if let Some(code) = Command::new("umount")
                .args([path])
                .status()
                .map_err(Error::Unmount)?
                .code()
            {
                if code != 0 {
                    return Err(crate::container::Error::Unmount(
                        std::io::Error::from_raw_os_error(code),
                    ));
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
            vec: vec![
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
