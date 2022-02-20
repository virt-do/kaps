use clap::Parser;

use crate::cli::{Cli, Handler, Result};

mod cli;
mod helper;

pub const KAPS_DATA_DIR: &str = "/tmp/kaps";

#[tokio::main]
async fn main() -> Result<()> {
    let cli: Cli = Cli::parse();

    cli.command().handler().await?;

    Ok(())
}
