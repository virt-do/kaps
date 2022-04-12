use crate::{Handler, Result};
use clap::Args;
use container::Container;

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
    /// Name of the container instance that will be start. It must me unique on your host
    #[clap(name = "container-id")]
    id: String,

    /// The bundle used by the container.
    #[clap(short, long)]
    bundle: String,
}

impl Handler for RunCommand {
    fn handler(&self) -> Result<()> {
        // Create a container by passing the bundle provided in arguments to it's constructor.
        let mut container = Container::new(&self.bundle, &self.id)?;

        // Run the container
        // At the moment, we don't have a detached mode for the container,
        // So the method call is blocking.
        container.run()?;

        Ok(())
    }
}
