use crate::helper::get_image_manager_instance;
use crate::{Handler, Result};
use async_trait::async_trait;
use clap::Args;
use log::LevelFilter;

/// Arguments for our `MountCommand`.
///
/// These arguments are parsed by `clap` and an instance of `PullCommand` containing
/// arguments is provided.
///
/// Example :
///
/// `kaps pull registry.hub.docker.com/library/busybox`
///
/// The `handler` method provided below will be executed.
#[derive(Debug, Args)]
pub struct MountCommand {
    /// The image identifier.
    image_id: String,
    /// If set, the command will be executed silently.
    #[clap(long, short)]
    quiet: bool,
}

#[async_trait]
impl Handler for MountCommand {
    async fn handler(&self, logger: &mut env_logger::Builder) -> Result<()> {
        // Change logger behavior and init it
        // If the logger was not initialized, nothing will be displayed into the console.
        if self.quiet {
            logger.filter_level(LevelFilter::Off);
        }
        logger.init();

        let mut im = get_image_manager_instance()?;

        // Create the bundle
        let bundle = im.mount(&self.image_id).await?;

        println!("{}", &bundle.display());
        Ok(())
    }
}
