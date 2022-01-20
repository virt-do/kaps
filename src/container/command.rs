use oci_spec::runtime::Process;

const DEFAULT_OCI_ARG: &str = "sh";

/// Implementation of the container arguments.
pub struct Command {
    arg0: String,
    args: Vec<String>,
}

impl Default for Command {
    fn default() -> Self {
        Command {
            arg0: DEFAULT_OCI_ARG.to_string(),
            args: vec![],
        }
    }
}

impl From<&Command> for unshare::Command {
    fn from(origin: &Command) -> Self {
        let mut command = unshare::Command::new(&origin.arg0);
        command.args(&origin.args);

        command
    }
}

impl From<&Option<Process>> for Command {
    fn from(process: &Option<Process>) -> Self {
        let mut command: Command = Command::default();
        if let Some(process) = process {
            if let Some(arguments) = process.args() {
                command.args = arguments.to_vec();
            }
        }

        if !command.args.is_empty() {
            command.arg0 = command.args.remove(0);
        }

        command
    }
}
