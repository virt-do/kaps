use clap::Parser;
use unshare::Namespace;

use std::path::PathBuf;

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
}

pub type Result<T> = std::result::Result<T, Error>;

struct Runtime {
    rootfs: PathBuf,
    spec_file: PathBuf,
}

impl Runtime {
    pub fn new(bundle: &str) -> Result<Self> {
        Ok(Runtime {
            rootfs: [bundle, OCI_RUNTIME_SPEC_ROOTFS].iter().collect(),
            spec_file: [bundle, OCI_RUNTIME_SPEC_FILE].iter().collect(),
        })
    }
}

fn main() -> Result<()> {
    let opts: RuntimeOpts = RuntimeOpts::parse();
    let mut namespaces = Vec::<Namespace>::new();

    namespaces.push(Namespace::Pid);

    let runtime = Runtime::new(&opts.bundle)?;

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
