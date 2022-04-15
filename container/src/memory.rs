use anyhow::Result;
use controlgroup::{
    v1::{memory, Cgroup, CgroupPath, SubsystemKind},
    Pid,
};
use oci_spec::runtime::LinuxMemory;
use std::path::PathBuf;

pub struct Memory {
    memory_cgroup: memory::Subsystem,
}

impl Memory {
    pub fn new() -> Self {
        // Define and create a new cgroup controlled by the Memory subsystem.
        let mut memory_cgroup = memory::Subsystem::new(CgroupPath::new(
            SubsystemKind::Memory,
            PathBuf::from("kaps"),
        ));
        memory_cgroup.create().unwrap();
        Memory { memory_cgroup }
    }

    pub fn apply(&mut self, memory: &LinuxMemory) -> Result<()> {
        if let Some(limit) = memory.limit() {
            if limit != 0 {
                let _ = &self.memory_cgroup.set_limit_in_bytes(limit);
            }
        }

        if let Some(swappiness) = memory.swappiness() {
            if swappiness != 0 {
                let _ = &self.memory_cgroup.set_swappiness(swappiness);
            }
        }

        if let Some(kernel) = memory.kernel() {
            if kernel != 0 {
                let _ = &self.memory_cgroup.set_kmem_limit_in_bytes(kernel);
            }
        }

        if let Some(kernel_tcp) = memory.kernel_tcp() {
            if kernel_tcp != 0 {
                let _ = &self.memory_cgroup.set_kmem_tcp_limit_in_bytes(kernel_tcp);
            }
        }

        if let Some(reservation) = memory.reservation() {
            if reservation != 0 {
                let _ = &self.memory_cgroup.set_soft_limit_in_bytes(reservation);
            }
        }

        if let Some(disable_oom_killer) = memory.disable_oom_killer() {
            let _ = &self.memory_cgroup.disable_oom_killer(disable_oom_killer);
        }

        // Attach the self process to the cgroup.
        let pid = Pid::from(std::process::id());
        self.memory_cgroup.add_task(pid).unwrap();

        Ok(())
    }

    pub fn delete(&mut self) -> Result<()> {
        // Removing self process from the cgroup
        let pid = Pid::from(std::process::id());
        self.memory_cgroup.remove_task(pid)?;
        // and deleting the cgroup.
        self.memory_cgroup.delete()?;
        Ok(())
    }
}
