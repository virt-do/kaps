use std::path::PathBuf;

use oci_spec::runtime::Spec;

use command::Command;
use environment::Environment;
use mounts::Mounts;
use namespaces::Namespaces;

mod command;
mod environment;
mod mounts;
mod namespaces;
pub mod spec;

/// Containers related errors
#[derive(Debug)]
pub enum Error {
    OCISpecificationLoad(oci_spec::OciSpecError),
    OCIInvalidNamespace(oci_spec::runtime::LinuxNamespaceType),
    ContainerSpawnCommand(unshare::Error),
    ContainerWaitCommand(std::io::Error),
    ContainerExit(i32),
    Unmount(std::io::Error),
}

/// A common result type for our container module.
pub type Result<T> = std::result::Result<T, Error>;

/// Some OCI constants useful for our container implementation.
const OCI_RUNTIME_SPEC_FILE: &str = "config.json";
const OCI_RUNTIME_SPEC_ROOTFS: &str = "rootfs";

/// The `Container` struct provides a simple way to
/// create and run a container on the host.
#[derive(Default)]
pub struct Container {
    /// The path to the rootfs used by the container
    rootfs: PathBuf,
    /// The namespaces which will be given to the container
    namespaces: Namespaces,
    /// The additional mounts mounted into the container beyond `rootfs`.
    mounts: Mounts,
    /// The container environment
    environment: Environment,
    /// The command entrypoint
    command: Command,
}

impl Container {
    /// Build a new container with the bundle provided in parameters.
    pub fn new(bundle_path: &str) -> Result<Self> {
        let bundle = PathBuf::from(bundle_path);

        // Load the specification from the file
        let spec =
            Spec::load(&bundle.join(OCI_RUNTIME_SPEC_FILE)).map_err(Error::OCISpecificationLoad)?;

        // Get the container rootfs from the OCI specification, and if not present, set to
        // the default `bundle_path/rootfs`
        let rootfs = spec
            .root()
            .as_ref()
            .map_or(bundle.join(OCI_RUNTIME_SPEC_ROOTFS), |x| {
                bundle.clone().join(x.path())
            });

        // Get the container namespaces if the linux block is defined into the specification.
        let namespaces = spec
            .linux()
            .as_ref()
            .map_or(Namespaces::default(), |linux| {
                Namespaces::from(linux.namespaces())
            });

        Ok(Container {
            environment: Environment::from(spec.process()),
            command: Command::from(spec.process()),
            namespaces,
            rootfs,
            ..Default::default()
        })
    }

    /// Run the container.
    pub fn run(&self) -> Result<()> {
        let mounts = self.mounts.clone();
        let code = unsafe {
            let mut child = match unshare::Command::from(&self.command)
                .chroot_dir(&self.rootfs)
                .unshare(&*self.namespaces.get())
                .pre_exec(move || Mounts::apply(&mounts))
                .envs(self.environment.get())
                .spawn()
            {
                Ok(child) => child,
                Err(_) => {
                    return self.mounts.cleanup(self.rootfs.clone());
                }
            };
            child.wait().map_err(Error::ContainerWaitCommand)?.code()
        };

        self.mounts.cleanup(self.rootfs.clone())?;

        if let Some(code) = code {
            if code != 0 {
                return Err(Error::ContainerExit(code));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Container, Error};
    use proc_mounts::MountList;
    use tempdir::TempDir;

    #[test]
    fn test_mount_on_empty_rootfs_should_fail_and_cleanup() -> Result<(), std::io::Error> {
        //use an empty rootfs for this test
        let dir = TempDir::new_in("../hack/fixtures", "test")?;
        let test_folder_path = dir.path().to_str().unwrap();
        std::fs::create_dir(format!("{}/rootfs", &test_folder_path))?;
        std::fs::copy(
            "../hack/fixtures/config.json",
            format!("{}/config.json", &test_folder_path),
        )?;

        let host_mounts_before_run_fail = MountList::new().unwrap();
        let container = Container::new(test_folder_path).unwrap();
        assert!(container.run().is_err());

        let host_mounts_after_run_fail = MountList::new().unwrap();
        assert_eq!(host_mounts_before_run_fail, host_mounts_after_run_fail);

        Ok(())
    }

    #[test]
    fn test_create_container_with_invalid_oci_runtime_spec_file() -> Result<(), Error> {
        assert!(Container::new("invalid_spec_file").is_err());
        Ok(())
    }
}
