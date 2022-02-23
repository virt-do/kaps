use std::collections::HashMap;
use std::path::PathBuf;

use oci_spec::image::ImageConfiguration;
use oci_spec::runtime::{Process, Spec, SpecBuilder};
use oci_spec::OciSpecError;

use oci_spec::image::ANNOTATION_CREATED;

pub const BUNDLE_CONFIG: &str = "config.json";

pub type Result<T> = std::result::Result<T, OciSpecError>;

/// Generate a runtime config and return his path
pub fn new_runtime_config(image_config: Option<&ImageConfiguration>) -> Result<Spec> {
    if let Some(image_config) = image_config {
        let annotations = build_annotations(image_config);
        let process = build_process(image_config);

        Ok(SpecBuilder::default()
            .version(String::from("1.0"))
            .process(process)
            .annotations(annotations)
            .build()?)
    } else {
        Ok(Spec::default())
    }
}

/// Build process from an image configuration and return it
fn build_process(image_config: &ImageConfiguration) -> Process {
    let mut args: Vec<String> = vec![];
    let mut process = Process::default();

    if let Some(config) = image_config.config() {
        if let Some(entrypoint) = config.entrypoint() {
            args.extend(entrypoint.clone());
        }
        if let Some(cmd) = config.cmd() {
            args.extend(cmd.clone());
        }
        if let Some(env) = config.env() {
            process.set_env(Some(env.to_vec()));
        }
        if let Some(working_dir) = config.working_dir() {
            process.set_cwd(PathBuf::from(working_dir));
        }
        if !args.is_empty() {
            process.set_args(Some(args));
        }
    }
    process
}

/// Build annotations from an image configuration and return it
fn build_annotations(image_config: &ImageConfiguration) -> HashMap<String, String> {
    let mut annotations: HashMap<String, String> = HashMap::new();

    if let Some(created) = image_config.created() {
        annotations.insert(ANNOTATION_CREATED.to_string(), created.to_string());
    }

    if let Some(config) = image_config.config() {
        if let Some(labels) = config.labels() {
            annotations.extend(labels.clone());
        }
    }
    annotations
}

#[cfg(test)]
mod tests {
    use super::*;
    use oci_spec::image::{ConfigBuilder, ImageConfigurationBuilder};

    #[test]
    fn test_process_config() -> Result<()> {
        let image_config = ImageConfigurationBuilder::default()
            .config(
                ConfigBuilder::default()
                    .cmd(vec![String::from("-c"), String::from("ls")])
                    .entrypoint(vec![String::from("bash")])
                    .env(vec![String::from("PATH=/usr/local/sbin")])
                    .working_dir(String::from("/home"))
                    .build()?,
            )
            .build()?;

        let spec = new_runtime_config(Some(&image_config));

        assert!(spec.is_ok());

        let spec = spec?;

        assert!(spec.process().is_some());
        if let Some(process) = spec.process() {
            assert!(process.args().is_some());
            if let Some(args) = process.args() {
                assert_eq!(*args, ["bash", "-c", "ls"]);
            }
            assert!(process.env().is_some());
            if let Some(env) = process.env() {
                assert_eq!(*env, ["PATH=/usr/local/sbin"]);
            }
            assert!(process.cwd().to_str().is_some());
            assert_eq!(process.cwd().to_str().unwrap(), "/home");
        }
        Ok(())
    }

    #[test]
    fn test_annotations_config() -> Result<()> {
        let image_config = ImageConfigurationBuilder::default()
            .author("jhon")
            .os("linux")
            .architecture("amd64")
            .created("01-12")
            .config(
                ConfigBuilder::default()
                    .stop_signal("SIGKILL")
                    .exposed_ports(vec![String::from("21/tcp")])
                    .build()?,
            )
            .build()?;

        let spec = new_runtime_config(Some(&image_config));

        assert!(spec.is_ok());

        let spec = spec?;

        assert!(spec.annotations().is_some());
        if let Some(annotations) = spec.annotations() {
            let created = annotations.get_key_value(&ANNOTATION_CREATED.to_string());
            assert!(created.is_some());
            if let Some(created) = created {
                assert_eq!(
                    created,
                    (&ANNOTATION_CREATED.to_string(), &String::from("01-12"))
                );
            }
        }

        Ok(())
    }
}
