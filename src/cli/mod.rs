mod mount;
mod pull;
mod run;
mod spec;

use crate::cli::mount::MountCommand;
use crate::cli::pull::PullCommand;
use crate::cli::run::RunCommand;
use crate::cli::spec::SpecCommand;
use async_trait::async_trait;
use clap::{Parser, Subcommand};

/// CLI related errors
#[derive(Debug)]
pub enum Error {
    Spec(oci_spec::OciSpecError),
    Runtime(container::Error),
    Image(oci_image::Error),
}

impl From<container::Error> for Error {
    fn from(error: container::Error) -> Self {
        Self::Runtime(error)
    }
}

impl From<oci_image::Error> for Error {
    fn from(error: oci_image::Error) -> Self {
        Self::Image(error)
    }
}

/// A common result type for our CLI.
pub type Result<T> = std::result::Result<T, Error>;

/// `Handler` is a trait that should be implemented for each of our commands.
///
/// It defines the contract & the input / output of a command execution.
#[async_trait]
pub trait Handler {
    /// Executes the command handler.
    ///
    /// Every command should take no argument, has it is built at runtime with these arguments.
    /// Also, a command must always return a `Result<()>`.
    async fn handler(&self, logger: &mut env_logger::Builder) -> crate::Result<()>;
}

#[derive(Parser, Debug)]
#[clap(version, author)]
pub struct Cli {
    /// The level of verbosity.
    #[clap(short, long, parse(from_occurrences))]
    pub(crate) verbose: usize,

    /// The subcommand to apply
    #[clap(subcommand)]
    pub(crate) command: Command,
}

impl Cli {
    /// Get the command used by the user.
    ///
    /// For example, if the user executes the command `run`,
    /// we dynamically return the command so the `main` can
    /// execute it.
    pub fn command(self) -> Box<dyn Handler> {
        match self.command {
            Command::Run(cmd) => Box::new(cmd),
            Command::Spec(cmd) => Box::new(cmd),
            Command::Pull(cmd) => Box::new(cmd),
            Command::Mount(cmd) => Box::new(cmd),
        }
    }
}

/// The enumeration of our commands.
///
/// Each of our commands should be listed in this enumeration with the following format :
/// CommandName(CommandHandler)
///
/// Example:
///
/// You want to add the `list` command:
///
/// List(ListCommand)
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Run a container
    Run(RunCommand),
    /// Generate container spec
    Spec(SpecCommand),
    // Pull a container image
    Pull(PullCommand),
    /// Mount an image into a rootfs to be used by a container
    Mount(MountCommand),
}
