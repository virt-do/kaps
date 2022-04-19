use crate::KAPS_DATA_DIR;
use oci_image::snapshots::overlay::OverlayFS;
use oci_image::ImageManager;
use std::path::Path;

/// Create a new image manager instance, with OverlayFS as the snapshotter
pub fn get_image_manager_instance() -> oci_image::Result<ImageManager> {
    let data_dir = Path::new(KAPS_DATA_DIR);
    let snapshots_dir = data_dir.join("snapshots");
    ImageManager::new(
        data_dir,
        Box::new(OverlayFS {
            data_dir: snapshots_dir,
        }),
    )
}
