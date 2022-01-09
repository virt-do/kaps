#[macro_use]
extern crate lazy_static;

use clap::Parser;
use oci_spec::runtime::{LinuxNamespaceType, Spec};
use unshare::Namespace;

use std::path::PathBuf;
use std::process::Command;

const OCI_RUNTIME_SPEC_FILE: &str = "config.json";
const OCI_RUNTIME_SPEC_ROOTFS: &str = "rootfs";

#[derive(Parser)]
#[clap(version = "0.1", author = "Polytech Montpellier - DevOps")]
struct RuntimeOpts {
    /// Container bundle
    #[clap(short, long)]
    bundle: String,
}

#[derive(Debug)]
pub enum Error {
    CmdSpawn(unshare::Error),

    ChildWait(std::io::Error),

    ChildExitError(i32),

    OciLoad(oci_spec::OciSpecError),

    OciSpecNsType(LinuxNamespaceType),

    RootfsCleanup(std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

struct Mount {
    typ: String,
    source: String,
    destination: String,
}

lazy_static! {
    static ref DEFAULT_MOUNTS: Vec<Mount> = vec![
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
    ];
}

struct Runtime {
    rootfs: PathBuf,
    spec: Spec,
}

impl Runtime {
    pub fn new(bundle: &str) -> Result<Self> {
        let spec_file: PathBuf = [bundle, OCI_RUNTIME_SPEC_FILE].iter().collect();
        let spec = Spec::load(&spec_file).map_err(Error::OciLoad)?;
        let rootfs: PathBuf = spec
            .root()
            .as_ref()
            .map_or([bundle, OCI_RUNTIME_SPEC_ROOTFS].iter().collect(), |r| {
                [bundle, &r.path().to_string_lossy()].iter().collect()
            });

        Ok(Runtime { rootfs, spec })
    }

    #[allow(unreachable_patterns)]
    fn from_oci_namespace(ns_type: LinuxNamespaceType) -> Result<Namespace> {
        match ns_type {
            LinuxNamespaceType::Cgroup => Ok(Namespace::Cgroup),
            LinuxNamespaceType::Ipc => Ok(Namespace::Ipc),
            LinuxNamespaceType::Mount => Ok(Namespace::Mount),
            LinuxNamespaceType::Network => Ok(Namespace::Net),
            LinuxNamespaceType::Pid => Ok(Namespace::Pid),
            LinuxNamespaceType::Uts => Ok(Namespace::Uts),
            LinuxNamespaceType::User => Ok(Namespace::User),
            _ => Err(Error::OciSpecNsType(ns_type)),
        }
    }

    pub fn namespaces(&self) -> Result<Vec<Namespace>> {
        let mut namespaces = Vec::<Namespace>::new();

        if let Some(linux) = self.spec.linux() {
            if let Some(ns) = linux.namespaces() {
                for namespace in ns {
                    let ns_type = Self::from_oci_namespace(namespace.typ())?;
                    if ns_type != Namespace::User {
                        namespaces.push(ns_type);
                    }
                }
            }
        }

        Ok(namespaces)
    }

    fn prepare_rootfs() -> std::result::Result<(), std::io::Error> {
        for mount in &*DEFAULT_MOUNTS {
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

    fn cleanup_rootfs(&self) -> Result<()> {
        for mount in &*DEFAULT_MOUNTS {
            let mut path = self.rootfs.clone();
            path.push(&mount.source);

            if let Some(code) = Command::new("umount")
                .args([path])
                .status()
                .map_err(Error::RootfsCleanup)?
                .code()
            {
                if code != 0 {
                    return Err(Error::RootfsCleanup(std::io::Error::from_raw_os_error(
                        code,
                    )));
                }
            }
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let opts: RuntimeOpts = RuntimeOpts::parse();
    let runtime = Runtime::new(&opts.bundle)?;
    let namespaces = runtime.namespaces()?;

    let code = unshare::Command::new("/bin/sh")
        .chroot_dir(&runtime.rootfs)
        .unshare(&namespaces)
        .spawn()
        .map_err(Error::CmdSpawn)?
        .wait()
        .map_err(Error::ChildWait)?
        .code();

    if let Some(code) = code {
        if code != 0 {
            return Err(Error::ChildExitError(code));
        }
    }

    Ok(())
}
