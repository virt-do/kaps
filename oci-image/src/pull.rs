use crate::state::LayerMetadata;
use crate::{Error, MetadataManager, Result, State};
use flate2::read::GzDecoder;
use oci_distribution::manifest::OciManifest;
use oci_distribution::secrets::RegistryAuth;
use oci_distribution::{Client, Reference};
use sha2::Digest;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tar::Archive;
use tokio::sync::Mutex;

/// `Puller` is responsible about pull OCI images from registries.
///
/// It provides methods to simply pull an image on your machine.
pub(crate) struct Puller {
    /// The OCI client which will be used to interact with registries.
    client: Client,

    /// The OCI registry auth information.
    auth: RegistryAuth,

    /// The OCI image reference
    reference: Reference,

    /// The directory where the image will be stored
    image_dir: PathBuf,

    /// The accepted media types which will be required to pull images.
    /// Example :
    accepted_media_types: Vec<String>,
}

impl Puller {
    /// Create a new instance of `Puller`.
    ///
    /// Configure client, authentication and image reference
    /// in order to properly pull the image.
    pub fn new(image: &str, image_dir: &Path) -> Result<Self> {
        let reference = Reference::try_from(image)
            .map_err(|_| Error::InvalidOCIReference(image.to_string()))?;
        let auth = RegistryAuth::Anonymous;

        Ok(Self {
            auth,
            reference,
            client: Client::default(),
            image_dir: image_dir.to_path_buf(),
            accepted_media_types: vec![],
        })
    }

    /// Pull the image manifest and the configuration from the registry.
    pub async fn pull_manifest(&mut self) -> Result<(OciManifest, String, String)> {
        let (manifest, digest, config) = self
            .client
            .pull_manifest_and_config(&self.reference, &self.auth)
            .await
            .map_err(|e| Error::PullManifest(e.to_string()))?;

        // Build a list of the accepted media types
        self.accepted_media_types = manifest
            .layers
            .iter()
            .map(|x| x.media_type.clone())
            .collect::<Vec<String>>();

        // Remove duplicates
        self.accepted_media_types.dedup();

        Ok((manifest, digest, config))
    }

    /// Pull the image from the registry. The method will return a vector containing the layers metadata for the image.
    ///
    /// The image will be stored into the `image_dir` argument provided in `Puller` constructor.
    /// This will produce a fully compliant OCI image with a `blobs` directory and a `manifest.json` file.
    pub async fn pull_layers(
        &mut self,
        state: Arc<Mutex<State>>,
        diffs: &[String],
    ) -> Result<Vec<LayerMetadata>> {
        // Get the image data
        let data = self
            .client
            .pull(
                &self.reference,
                &self.auth,
                Vec::from_iter(self.accepted_media_types.iter().map(String::as_str)),
            )
            .await
            .map_err(|e| Error::PullImage(e.to_string()))?;

        let layers_dir = self.image_dir.join("layers");
        // Create the directories to store layers if not exists
        create_dir_all(&layers_dir).map_err(|e| Error::LayerDirectoryCreation(e.to_string()))?;

        let layers = data.layers.into_iter().enumerate().map(|(i, layer)| {
            let state = state.clone();
            let layer_data = layer.data.clone();
            let layer_sha256_digest = layer.sha256_digest();
            let layer_path = layers_dir.join(&layer_sha256_digest.replace(':', "_"));
            // This block defines the behavior for one layer.
            async move {
                // If the layer already exists on the disk, return directly
                if let Some(layer_meta) = state.lock().await.layer(&layer_sha256_digest) {
                    log::info!("Layer {} : {} (cached)", &i, &layer_sha256_digest);
                    return Ok::<_, crate::Error>(layer_meta.clone());
                }

                log::info!("Layer {} : {}", &i, &layer_sha256_digest);

                // Decompress the layer
                let mut out: Vec<u8> = Vec::new();
                let mut decoder = GzDecoder::new(layer_data.as_slice());
                std::io::copy(&mut decoder, &mut out).unwrap();

                let uncompressed_digest = format!("{}:{:x}", "sha256", sha2::Sha256::digest(&out));
                if uncompressed_digest != diffs[i] {
                    return Err(Error::UncompressedLayerInvalid(format!("uncompressed digest is different than the digest defined in the image configuration. digest={:?}, image_digest={:?}", &uncompressed_digest, &diffs[i])));
                }

                // Finally, unpack the layer
                let mut archive = Archive::new(out.as_slice());
                archive.unpack(PathBuf::from(&layer_path)).map_err(|e| Error::UnpackLayer(format!("failed to unpack layer {}. details = {:?}", &layer_path.display(), e)))?;

                let layer_meta = LayerMetadata {
                    id: layer_sha256_digest.clone(),
                    compressed_digest: layer_sha256_digest.clone(),
                    store_path: layer_path.display().to_string(),
                    uncompressed_digest,
                };

                state.lock().await.add_layer(&layer_meta.clone());

                Ok::<_, crate::Error>(layer_meta)
            }
        });

        futures_util::future::try_join_all(layers).await
    }
}

#[cfg(test)]
mod tests {
    use crate::pull::Puller;
    use crate::State;
    use oci_spec::image::ImageConfiguration;
    use std::env::temp_dir;
    use std::path::PathBuf;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    fn test_dir() -> PathBuf {
        temp_dir().join("kaps_tests").join("images")
    }

    #[test]
    fn it_create_a_puller_instance() {
        assert!(Puller::new("docker.io/library/busybox", test_dir().as_path()).is_ok());
    }

    #[test]
    fn it_throw_an_error_if_invalid_image_reference() {
        assert!(Puller::new("$", test_dir().as_path()).is_err());
    }

    #[tokio::test]
    async fn it_pull_a_manifest() {
        let mut client = Puller::new("docker.io/amd64/ubuntu", test_dir().as_path()).unwrap();
        assert!(client.pull_manifest().await.is_ok())
    }

    #[tokio::test]
    async fn it_pull_an_image_layers() {
        let state = Arc::new(Mutex::new(State::default()));
        let mut client = Puller::new("docker.io/amd64/ubuntu", test_dir().as_path()).unwrap();
        let manifest_result = client.pull_manifest().await;
        assert!(manifest_result.is_ok());

        let (_, _, config) = manifest_result.unwrap();

        let cfg = ImageConfiguration::from_reader(config.as_bytes()).unwrap();
        let pull_result = client.pull_layers(state, cfg.rootfs().diff_ids()).await;
        assert!(pull_result.is_ok());
    }
}
