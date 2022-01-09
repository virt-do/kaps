use clap::Parser;
use unshare::Namespace;

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

fn main() -> Result<()> {
    let opts: RuntimeOpts = RuntimeOpts::parse();
    let mut namespaces = Vec::<Namespace>::new();

    namespaces.push(Namespace::Pid);

    let code = unshare::Command::new("/bin/sh")
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
