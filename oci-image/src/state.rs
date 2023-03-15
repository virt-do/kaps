use crate::Error;
use oci_spec::image::ImageConfiguration;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// The `MetadataManager` defines the contract that must be implemented by
/// our state implementation.
pub(crate) trait MetadataManager {
    /// Add an `Image` to the state.
    fn add_image(&mut self, image: &ImageMetadata) -> &mut Self;
    /// Add a `Layer` to the state.
    fn add_layer(&mut self, layer: &LayerMetadata) -> &mut Self;
    /// Check if the state contains an `Image` by it's identifier
    fn has_image(&self, image_id: &str) -> bool;
    /// Check if the state contains a `Layer` by it's identifier
    fn has_layer(&self, layer_id: &str) -> bool;
    /// Get an image from the state by it's identifier
    fn image(&self, image_id: &str) -> Option<&ImageMetadata>;
    /// Get a layer from the state by it's identifier
    fn layer(&self, layer_id: &str) -> Option<&LayerMetadata>;
    /// Get a new snapshot index
    fn snapshot_index(&mut self) -> usize;
}

/// `LayerMetadata` struct holds information's about a layer in the state.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct LayerMetadata {
    /// The id of the layer
    pub id: String,
    /// The compressed layer digest
    pub compressed_digest: String,
    /// The decompressed layer digest,
    pub uncompressed_digest: String,
    /// The path where the layer is stored
    pub store_path: String,
}

/// `ImageMetadata` struct holds information's about an image in the state.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct ImageMetadata {
    /// The image id
    pub id: String,
    /// The image reference, such as `docker.io/amd64/ubuntu`
    pub reference: String,
    /// The image digest
    pub digest: String,
    /// The image layers metas
    pub layers: Vec<LayerMetadata>,
    /// The image configuration
    pub config: ImageConfiguration,
}

/// `State` is responsible about storing information's about images and layers.
#[derive(Clone, Default, Deserialize, Debug, Serialize, PartialEq)]
pub(crate) struct State {
    /// An hashmap that holds every images pulled
    images: HashMap<String, ImageMetadata>,
    /// An hashmap that holds every layers pulled
    layers: HashMap<String, LayerMetadata>,
    /// An index to track the last snapshot identifier
    index: usize,
}

impl State {
    /// Save the state into the file.
    pub fn save(&self, path: &Path) -> crate::Result<()> {
        let serialized = serde_json::to_string_pretty(&self)
            .map_err(|e| Error::SerializeState(e.to_string()))?;

        OpenOptions::new()
            .write(true)
            .open(path)
            .map_err(|e| Error::OpenStateFile(e.to_string()))?
            .write_all(serialized.as_bytes())
            .map_err(|e| Error::WriteStateFile(e.to_string()))
    }
}

impl MetadataManager for State {
    fn add_image(&mut self, image: &ImageMetadata) -> &mut State {
        self.images.insert(image.id.clone(), image.clone());
        self
    }

    fn add_layer(&mut self, layer: &LayerMetadata) -> &mut State {
        self.layers
            .insert(layer.compressed_digest.clone(), layer.clone());
        self
    }

    fn has_image(&self, image_id: &str) -> bool {
        self.images.contains_key(image_id)
    }

    fn has_layer(&self, layer_id: &str) -> bool {
        self.layers.contains_key(layer_id)
    }

    fn image(&self, image_id: &str) -> Option<&ImageMetadata> {
        self.images.get(image_id)
    }

    fn layer(&self, layer_id: &str) -> Option<&LayerMetadata> {
        self.layers.get(layer_id)
    }

    fn snapshot_index(&mut self) -> usize {
        let current_index = AtomicUsize::new(self.index);
        current_index.fetch_add(1, Ordering::SeqCst);
        // Load the new index from the current
        let new_index = current_index.load(Ordering::SeqCst);
        self.index = new_index;
        new_index
    }
}

impl TryFrom<&PathBuf> for State {
    type Error = crate::Error;

    fn try_from(state_file: &PathBuf) -> Result<Self, Self::Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(state_file)
            .map_err(|e| Error::OpenStateFile(e.to_string()))?;

        Ok(serde_json::from_reader::<File, State>(file).map_or_else(|_| State::default(), |s| s))
    }
}

#[cfg(test)]
mod tests {
    use crate::state::LayerMetadata;
    use crate::{ImageMetadata, MetadataManager, State};
    use std::env::temp_dir;
    use std::fs::create_dir_all;

    #[test]
    fn test_save_state_in_file() {
        let state_file = temp_dir().join("kaps_tests");

        create_dir_all(&state_file).expect("Failed to create directories for state file");

        let state = State::try_from(&state_file.join("test_state.json")).unwrap();

        assert!(state.save(&state_file.join("test_state.json")).is_ok());
    }

    #[test]
    fn test_add_image() {
        let state_file = temp_dir().join("kaps_tests");
        create_dir_all(&state_file).expect("Failed to create directories for state file");
        let mut state = State::try_from(&state_file.join("test_add_image.json")).unwrap();

        state.add_image(&ImageMetadata::default());

        assert_eq!(state.images.len(), 1);
    }

    #[test]
    fn test_add_layer() {
        let state_file = temp_dir().join("kaps_tests");
        create_dir_all(&state_file).expect("Failed to create directories for state file");
        let mut state = State::try_from(&state_file.join("test_add_layer.json")).unwrap();

        state.add_layer(&LayerMetadata::default());

        assert_eq!(state.layers.len(), 1);
    }

    #[test]
    fn test_load_state_from_file() {
        let fixture_layer = LayerMetadata::default();
        let fixture_image = ImageMetadata::default();

        let state_file = temp_dir().join("kaps_tests");

        create_dir_all(&state_file).expect("Failed to create directories for state file");

        let mut state = State::try_from(&state_file.join("test_state_2.json")).unwrap();

        state.add_image(&fixture_image).add_layer(&fixture_layer);

        assert!(state.save(&state_file.join("test_state_2.json")).is_ok());

        let new_state = State::try_from(&state_file.join("test_state_2.json")).unwrap();

        assert_eq!(new_state, state);
        assert_eq!(new_state.images.len(), 1);
        assert_eq!(new_state.layers.len(), 1);
    }
}
