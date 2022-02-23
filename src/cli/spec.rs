use std::path::PathBuf;

use crate::{Handler, Result};
use clap::Args;
use container::spec::{new_runtime_config, BUNDLE_CONFIG};
use oci_spec::image::ImageConfiguration;

use super::Error;

#[derive(Debug, Args)]
pub struct SpecCommand {}

impl Handler for SpecCommand {
    fn handler(&self) -> Result<()> {
        let image_configuration = ImageConfiguration::default();
        let spec = new_runtime_config(Some(&image_configuration)).map_err(Error::Spec)?;
        let bundle_config = PathBuf::from(".").join(BUNDLE_CONFIG);
        spec.save(&bundle_config).map_err(Error::Spec)?;
        Ok(())
    }
}
