use crate::{Handler, Result};
use async_trait::async_trait;
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
    name: String,

    /// The bundle used by the container.
    #[clap(short, long)]
    bundle: String,
}

#[async_trait]
impl Handler for RunCommand {
    async fn handler(&self, _: &mut env_logger::Builder) -> Result<()> {
        // Create a container by passing the bundle provided in arguments to it's constructor.
        let mut container = Container::new(&self.bundle, &self.name)?;

        // Run the container
        // At the moment, we don't have a detached mode for the container,
        // So the method call is blocking.
        container.run()?;

        Ok(())
    }
}
