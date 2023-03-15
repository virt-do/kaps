use clap::Parser;
use log::{log_enabled, Level, LevelFilter};
use std::io::Write;

use crate::cli::{Cli, Handler, Result};

mod cli;
mod helper;

pub const KAPS_DATA_DIR: &str = "/var/lib/kaps";

#[tokio::main]
async fn main() -> Result<()> {
    let cli: Cli = Cli::parse();

    // Configure the logger
    let mut builder = env_logger::Builder::new();
    let logger = builder
        .filter_level(match cli.verbose {
            1 => LevelFilter::Debug,
            2 => LevelFilter::Trace,
            _ => LevelFilter::Info,
        })
        .format(|buf, record| {
            if record.level() != Level::Info
                || log_enabled!(Level::Trace)
                || log_enabled!(Level::Debug)
            {
                return writeln!(
                    buf,
                    "{}: {}",
                    record.level().to_string().to_lowercase(),
                    record.args()
                );
            }
            writeln!(buf, "{}", record.args())
        });

    // We have to pass the logger to our downstream command
    // so that way the logger behavior can be changed by the command.
    // The downstream command must INIT the logger before doing any task.
    cli.command().handler(logger).await?;

    Ok(())
}
