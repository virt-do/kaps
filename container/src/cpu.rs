use anyhow::Result;
use controlgroup::{
    v1::{cpu, Cgroup, CgroupPath, SubsystemKind},
    Pid,
};
use oci_spec::runtime::LinuxCpu;
use std::path::PathBuf;

pub struct Cpu {
    cpu_cgroup: cpu::Subsystem,
}

impl Cpu {
    pub fn new() -> Self {
        // Define and create a new cgroup controlled by the CPU subsystem.
        let mut cpu_cgroup =
            cpu::Subsystem::new(CgroupPath::new(SubsystemKind::Cpu, PathBuf::from("kaps")));
        cpu_cgroup.create().unwrap();
        Cpu { cpu_cgroup }
    }

    pub fn apply(&mut self, cpu: &LinuxCpu) -> Result<()> {
        if let Some(cpu_shares) = cpu.shares() {
            if cpu_shares != 0 {
                let _ = &self.cpu_cgroup.set_shares(cpu_shares);
            }
        }

        if let Some(cpu_period) = cpu.period() {
            if cpu_period != 0 {
                let _ = &self.cpu_cgroup.set_cfs_period_us(cpu_period);
            }
        }

        if let Some(cpu_quota) = cpu.quota() {
            if cpu_quota != 0 {
                let _ = &self.cpu_cgroup.set_cfs_quota_us(cpu_quota);
            }
        }

        if let Some(rt_runtime) = cpu.realtime_runtime() {
            if rt_runtime != 0 {
                let _ = &self.cpu_cgroup.set_rt_runtime_us(rt_runtime);
            }
        }

        if let Some(rt_period) = cpu.realtime_period() {
            if rt_period != 0 {
                let _ = &self.cpu_cgroup.set_rt_period_us(rt_period);
            }
        }

        // Attach the self process to the cgroup.
        let pid = Pid::from(std::process::id());
        self.cpu_cgroup.add_task(pid).unwrap();

        Ok(())
    }

    pub fn delete(&mut self) -> Result<()> {
        // Removing self process from the cgroup
        let pid = Pid::from(std::process::id());
        self.cpu_cgroup.remove_task(pid)?;
        // and deleting the cgroup.
        self.cpu_cgroup.delete()?;
        Ok(())
    }
}
