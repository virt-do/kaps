use crate::pull::Puller;
use crate::snapshots::Snapshotter;
use crate::state::{ImageMetadata, MetadataManager, State};
use crate::utils::to_uid;
use oci_spec::image::ImageConfiguration;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

mod pull;
pub mod snapshots;
mod state;
mod utils;

pub const OCI_CONFIG: &str = "config.json";
pub const OCI_ROOTFS: &str = "rootfs";

/// The path of the file where the Kaps state will be stored.
pub(crate) const STATE_FILE: &str = "state.json";
pub(crate) const IMAGES_DIR: &str = "images";
pub(crate) const BUNDLES_DIR: &str = "bundles";

#[derive(Debug)]
pub enum Error {
    /// The OCI reference parsed with the image is invalid
    InvalidOCIReference(String),
    /// An error occurred when pulling the manifest
    PullManifest(String),
    /// An error occurred when pulling the image
    PullImage(String),
    /// An error occurred during the creation of the directory where the layer should be stored
    LayerDirectoryCreation(String),
    /// An error occurred during the creation of the layer file
    LayerFileCreation(String),
    /// An error occurred when the layer file should be written
    LayerFileWrite(String),
    /// An error occurred when the manager tried to create the data_dir
    ManagerDataDirectoryCreation(String),
    /// An error occurred when trying to create the image manifest file
    ImageManifestFileCreation(String),
    /// An error occurred when trying to write the image manifest file
    ImageManifestFileWrite(String),
    /// An error occurred when trying to open the state file
    OpenStateFile(String),
    /// An error occurred when trying to parse the state file
    ParseStateFile(String),
    /// An error occurred when trying to serialize the state
    SerializeState(String),
    /// An error occurred when trying to write the state into the file
    WriteStateFile(String),
    /// An error occurred when trying to parse the image configuration
    ParseImageConfiguration(String),
    /// The pulled number of layers is invalid
    InvalidPulledLayers(String),
    /// The layer uncompressed digest is different than the uncompressed layer digest defined in the image configuration.
    UncompressedLayerInvalid(String),
    /// The image is not found on the disk
    ImageNotFound(String),
    /// An error occurred when trying to mount OverlayFS layers
    OverlayFSMount(String),
    /// An error occurred when trying to umount an existing OverlayFS
    OverlayFSUmount(nix::Error),
    /// An error occurred when trying to create an OverlayFS related directory
    OverlayFSCreateDirectory(String),
    /// An error occurred when trying to generate the OCI config for a bundle
    GenerateOCIConfig(oci_spec::OciSpecError),
    /// An error occured when trying to unpack a layer
    UnpackLayer(String),
}

/// A common result type for our crate.
pub type Result<T> = std::result::Result<T, Error>;

/// The `ImageManager` should be responsible of the OCI images management.
///
/// It provides method to pull and unpack OCI images from OCI registries. It maintains a consistent state
/// in order to avoid pulling existing layers, to improve performance.
pub struct ImageManager {
    /// The directory where images and bundles will be stored on the host.
    data_dir: PathBuf,
    /// The image manager state
    state: Arc<Mutex<State>>,
    /// The snapshot to use
    snapshot: Box<dyn Snapshotter>,
}

impl ImageManager {
    pub fn new(data_dir: &Path, snapshot: Box<dyn Snapshotter>) -> Result<Self> {
        // If data_dir not is not existing
        // we must create it
        if !data_dir.exists() {
            create_dir_all(data_dir)
                .map_err(|e| Error::ManagerDataDirectoryCreation(e.to_string()))?;
        }

        let state = State::try_from(&data_dir.join(STATE_FILE))?;

        Ok(Self {
            data_dir: data_dir.to_path_buf(),
            state: Arc::new(Mutex::new(state)),
            snapshot,
        })
    }

    /// Unpack an OCI image.
    ///
    /// Returns an error if the image is not found.
    pub async fn mount(&mut self, image_id: &str) -> Result<PathBuf> {
        // Get the image, and if not found, throw an error
        let image = self
            .state
            .lock()
            .await
            .image(image_id)
            .ok_or_else(|| Error::ImageNotFound(format!("No image found with id={}", image_id)))?
            .clone();

        let snapshot_index = self.state.lock().await.snapshot_index();
        let mount_path = self.bundles_dir().join(&image.id);

        log::debug!("creating new OCI bundle into {}", mount_path.display());

        self.snapshot.mount(
            image
                .layers
                .into_iter()
                .map(|layer| layer.store_path)
                .collect(),
            mount_path.join(OCI_ROOTFS).as_path(),
            &snapshot_index,
            false,
        )?;

        log::debug!("generating oci runtime configuration based on image");
        // Generate a new runtime configuration based on image configuration
        container::spec::new_runtime_config(Some(&image.config))
            .map_err(Error::GenerateOCIConfig)?
            .save(mount_path.join(OCI_CONFIG))
            .map_err(Error::GenerateOCIConfig)?;

        self.state.lock().await.save(&self.state_file())?;

        Ok(mount_path)
    }

    /// Pull an image.
    ///
    /// This method pull an OCI image and generate a directory containing the image layers and the
    /// image manifest in the file `index.json`.
    pub async fn pull(
        &mut self,
        image: &str,
        remove_existing: &bool,
        id: &Option<String>,
    ) -> Result<String> {
        let mut puller = Puller::new(image, &self.images_dir())?;

        log::info!("Getting {} manifest...", &image);

        // Pull the image manifest and configuration
        let (image_manifest, image_digest, image_config_raw) = puller.pull_manifest().await?;

        let image_id = match id {
            None => to_uid(&image_digest),
            Some(id) => id.clone(),
        };

        // We don't want to pull images each time if the image is already present to improve performance.
        // If the argument `--rm` is provided, `remove_existing` will be true so we have to check
        // if the pull must be forced or not.
        if self.state.lock().await.has_image(&image_id) && !*remove_existing {
            log::info!(
                "Image {} already present on disk. To force image pulling, please specify `--rm`.",
                &image
            );
            return Ok(image_id);
        }

        // Parse the image configuration
        let config = ImageConfiguration::from_reader(image_config_raw.as_bytes())
            .map_err(|e| Error::ParseImageConfiguration(e.to_string()))?;

        // Check if the number of layers in manifest are equals to the number of layers parsed in the image configuration.
        if config.rootfs().diff_ids().len() != image_manifest.layers.len() {
            return Err(Error::InvalidPulledLayers(
                "Pulled number of layers is not the same defined in image configuration."
                    .to_string(),
            ));
        }

        log::info!("Pulling image {}...", &image);

        // Pull the image layers
        let layers = puller
            .pull_layers(self.state.clone(), config.rootfs().diff_ids())
            .await?;

        self.state
            .lock()
            .await
            .add_image(&ImageMetadata {
                id: image_id.clone(),
                digest: image_digest.clone(),
                reference: image.to_string(),
                layers,
                config,
            })
            .save(&self.state_file())?;

        Ok(image_id)
    }
    /// Get the state file path
    fn state_file(&self) -> PathBuf {
        self.data_dir.join(STATE_FILE)
    }

    /// Get the default images directory path.
    fn images_dir(&self) -> PathBuf {
        self.data_dir.join(IMAGES_DIR)
    }

    /// Get the default bundles directory path.
    fn bundles_dir(&self) -> PathBuf {
        self.data_dir.join(BUNDLES_DIR)
    }
}

#[cfg(test)]
mod tests {
    use crate::snapshots::overlay::OverlayFS;
    use crate::ImageManager;
    use std::env::temp_dir;
    use std::fs::remove_dir_all;

    #[test]
    fn test_it_create_a_manager_instance() {
        let data_dir = temp_dir().join("kaps");
        let im = ImageManager::new(
            &data_dir,
            Box::new(OverlayFS {
                data_dir: data_dir.join("snapshots"),
            }),
        );
        assert!(im.is_ok())
    }

    #[tokio::test]
    async fn test_it_pull_images() {
        let fixtures = vec!["docker.io/amd64/alpine", "mcr.microsoft.com/hello-world"];

        let data_dir = temp_dir().join("kaps_tests");
        let mut im = ImageManager::new(
            &data_dir,
            Box::new(OverlayFS {
                data_dir: data_dir.join("snapshots"),
            }),
        )
        .unwrap();

        for image in fixtures {
            assert!(im.pull(image, &true, &None).await.is_ok());
        }

        remove_dir_all(data_dir).expect("failed to clean up")
    }
}
