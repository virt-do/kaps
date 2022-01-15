use crate::container::Error;
use oci_spec::runtime::{LinuxNamespace, LinuxNamespaceType};
use unshare::Namespace;

#[derive(Default)]
pub struct Namespaces {
    vec: Vec<Namespace>,
}

impl Namespaces {
    /// Get the namespaces
    pub fn get(&self) -> &Vec<Namespace> {
        &self.vec
    }

    /// Convert an `oci_spec::runtime::LinuxNamespaceType` to an `unshare::Namespace`
    /// It returns an error if the namespace is invalid, or if it does not match any pattern.
    #[allow(unreachable_patterns)]
    fn from_oci_namespace(namespace: LinuxNamespaceType) -> crate::container::Result<Namespace> {
        match namespace {
            LinuxNamespaceType::Cgroup => Ok(Namespace::Cgroup),
            LinuxNamespaceType::Ipc => Ok(Namespace::Ipc),
            LinuxNamespaceType::Mount => Ok(Namespace::Mount),
            LinuxNamespaceType::Network => Ok(Namespace::Net),
            LinuxNamespaceType::Pid => Ok(Namespace::Pid),
            LinuxNamespaceType::Uts => Ok(Namespace::Uts),
            LinuxNamespaceType::User => Ok(Namespace::User),
            _ => Err(Error::OCIInvalidNamespace(namespace)),
        }
    }
}

impl From<&Option<Vec<LinuxNamespace>>> for Namespaces {
    fn from(namespaces: &Option<Vec<LinuxNamespace>>) -> Self {
        let vec = namespaces
            .as_ref()
            .unwrap_or(&Vec::<LinuxNamespace>::new())
            .iter()
            .map(|n| Self::from_oci_namespace(n.typ()).unwrap())
            // This is temporary, awaiting GUID and UID to be merged
            .filter(|n| n != &Namespace::User)
            .collect::<Vec<Namespace>>();

        Self { vec }
    }
}
