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
