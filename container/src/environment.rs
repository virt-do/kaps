use oci_spec::runtime::Process;

/// Implementation of the container environment.
#[derive(Default)]
pub struct Environment {
    vars: Vec<(String, String)>,
}

impl Environment {
    /// Get the environment variables.
    /// It converts the `Vec<(String, String)>` into a `Vec<(&str, &str)>` for `unshare` API compatibility.
    pub fn get(&self) -> Vec<(&str, &str)> {
        self.vars
            .iter()
            .map(|(key, value)| (key.as_ref(), value.as_ref()))
            .collect()
    }
}

impl From<&Option<Process>> for Environment {
    fn from(process: &Option<Process>) -> Self {
        let mut vars = Vec::<(String, String)>::new();
        if let Some(process) = process {
            if let Some(env) = process.env() {
                for var in env {
                    let key_value = var.split('=').collect::<Vec<&str>>();
                    vars.push((key_value[0].to_string(), key_value[1].to_string()));
                }
            }
        }

        Environment { vars }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Environment, Error};
    use oci_spec::runtime::Process;

    #[test]
    fn test_environment_from_process() -> Result<(), Error> {
        let mut test_process = Process::default();
        test_process.set_env(Some(vec![
            "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_string(),
            "TERM=xterm".to_string(),
        ]));
        let test_environment = Environment::from(&Some(test_process));

        assert_eq!(
            test_environment.vars[0],
            (
                "PATH".to_string(),
                "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_string()
            )
        );
        assert_eq!(
            test_environment.vars[1],
            (("TERM".to_string(), "xterm".to_string()))
        );

        Ok(())
    }
}
