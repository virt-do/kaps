use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

const KAPS_ROOT_PATH: &str = "/var/run/kaps/containers";
const OCI_VERSION: &str = "0.2.0";
const STATE_FILE: &str = "state.json";

/// Container runtime status
#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
pub enum Status {
    #[serde(rename = "creating")]
    Creating,
    #[serde(rename = "created")]
    Created,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "stopped")]
    Stopped,
}

impl Default for Status {
    fn default() -> Self {
        Status::Creating
    }
}

/// Represent the state of the running container.
#[derive(Serialize, Deserialize, Debug)]
pub struct ContainerState {
    id: String,
    /// OCI version.
    oci_version: String,
    /// Runtime state of the container.
    pub status: Arc<RwLock<Status>>,
    /// ID of the container process.
    pub pid: i32,
    /// Path to the bundle.
    bundle: PathBuf,
}

impl Default for ContainerState {
    fn default() -> Self {
        ContainerState {
            oci_version: OCI_VERSION.to_string(),
            id: String::default(),
            status: Arc::new(RwLock::new(Status::default())),
            pid: 0,
            bundle: PathBuf::default(),
        }
    }
}

impl ContainerState {
    pub fn new(id: &str, bundle_path: &str) -> Result<Self> {
        ContainerState::_new(id, bundle_path, KAPS_ROOT_PATH)
    }

    fn _new(id: &str, bundle_path: &str, container_dir: &str) -> Result<Self> {
        let bundle = PathBuf::from(bundle_path);
        let container_path = PathBuf::from(container_dir).join(id);

        if container_path.as_path().exists() {
            return Err(Error::ContainerExists(format!(
                "A container with the id '{}' already exists",
                id
            )));
        }

        // create the container directory
        fs::create_dir_all(&container_path).map_err(Error::CreateStateFile)?;

        // create the `state.json` file of the container
        File::create(container_path.join(STATE_FILE)).map_err(Error::CreateStateFile)?;

        let container_state = ContainerState {
            id: id.to_string(),
            bundle,
            ..Default::default()
        };

        container_state.save(container_dir)?;

        Ok(container_state)
    }

    /// Get the current runtime status of the container.
    ///
    /// As The container status is a RwLock,
    /// calling this function results in acquiring a read lock on the status.
    fn _status(&self) -> Result<Status> {
        let container_status = Arc::clone(&self.status);
        let container_status = container_status
            .read()
            .map_err(|e| Error::StatusLockPoisoned(e.to_string()))?;

        Ok(*container_status)
    }

    /// Save the container state.
    ///
    /// The container state file must already have been created.
    fn save(&self, container_dir: &str) -> Result<()> {
        let container_path = PathBuf::from(container_dir).join(&self.id);

        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(container_path.join(STATE_FILE))
            .map_err(Error::OpenStateFile)?;

        serde_json::to_writer_pretty(file, &self).map_err(Error::WriteStateFile)
    }

    /// Update runtime status of container.
    pub fn set_status(&mut self, status: Status) -> Result<()> {
        self._set_status(status, KAPS_ROOT_PATH)
    }

    fn _set_status(&mut self, status: Status, container_dir: &str) -> Result<()> {
        let container_status = Arc::clone(&self.status);

        let mut container_status = container_status
            .write()
            .map_err(|e| Error::StatusLockPoisoned(e.to_string()))?;

        *container_status = status;
        drop(container_status);

        self.save(container_dir)
    }

    /// Remove the container state file.
    pub fn remove(&self) -> Result<()> {
        self._remove(KAPS_ROOT_PATH)
    }

    fn _remove(&self, container_dir: &str) -> Result<()> {
        let container_path = PathBuf::from(container_dir).join(&self.id);

        fs::remove_dir_all(container_path).map_err(Error::RemoveStateFile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Error, Result};
    use std::path::Path;

    const KAPS_TEST_ROOT_PATH: &str = "/tmp/kaps";

    #[test]
    fn should_create_state_file() -> Result<()> {
        let container_id = "test1";

        let state = ContainerState::_new(container_id, "fake/path/to/bundle", KAPS_TEST_ROOT_PATH)?;

        assert!(Path::new(KAPS_TEST_ROOT_PATH).join(container_id).exists());

        let _ = state._remove(KAPS_TEST_ROOT_PATH)?;

        Ok(())
    }

    #[test]
    fn should_remove_state_file() -> Result<()> {
        let container_id = "test2";

        let state = ContainerState::_new(container_id, "fake/path/to/bundle", KAPS_TEST_ROOT_PATH)?;

        let _ = state._remove(KAPS_TEST_ROOT_PATH)?;

        assert!(!Path::new(KAPS_TEST_ROOT_PATH).join(container_id).exists());

        Ok(())
    }

    #[test]
    fn should_update_runtime_status() -> Result<()> {
        let container_id = "test3";
        let container_path = PathBuf::from(KAPS_TEST_ROOT_PATH).join(container_id);

        let mut state =
            ContainerState::_new(container_id, "fake/path/to/bundle", KAPS_TEST_ROOT_PATH)?;

        state._set_status(Status::Stopped, KAPS_TEST_ROOT_PATH)?;

        let file_state =
            fs::read_to_string(container_path.join(STATE_FILE)).map_err(Error::ReadStateFile)?;

        let file_state: ContainerState =
            serde_json::from_str(&file_state).map_err(Error::SerializeError)?;

        let file_status = file_state._status()?;

        let status = state._status()?;

        assert_eq!(status, file_status);

        let _ = state._remove(KAPS_TEST_ROOT_PATH)?;

        Ok(())
    }
}
