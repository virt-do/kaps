use crate::container::Container;
use crate::{Handler, Result};
use clap::Args;

/// Arguments for our `RunCommand`.
///
/// These arguments are parsed by `clap` and an instance of `RunCommand` containing
/// arguments is provided.
///
/// Example :
///
/// `run0 run -b /tmp/your-bundle`
///
/// The `handler` method provided below will be executed.
#[derive(Debug, Args)]
pub struct RunCommand {
    /// The bundle used by the container.
    #[clap(short, long)]
    bundle: String,
}

impl Handler for RunCommand {
    fn handler(&self) -> Result<()> {
        // Create a container by passing the bundle provided in arguments to it's constructor.
        let container = Container::new(&self.bundle)?;

        // Run the container
        // At the moment, we don't have a detached mode for the container,
        // So the method call is blocking.
        container.run()?;

        Ok(())
    }
}
