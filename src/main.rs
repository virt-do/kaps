use std::process::Command;

#[derive(Debug)]
pub enum Error {
    CmdSpawn(std::io::Error),

    ChildWait(std::io::Error),

    ChildExitError(i32),
}

fn main() -> Result<(), Error> {
    let code = Command::new("/bin/sh")
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
