use clap::Parser;

use crate::cli::{Cli, Handler, Result};

mod cli;

fn main() -> Result<()> {
    let cli: Cli = Cli::parse();

    cli.command().handler()?;

    Ok(())
}
