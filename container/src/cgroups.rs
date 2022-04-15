
/*pub struct Cgroups {
    cgroup: Cgroup,
    resources: Resources,
}

impl Cgroups {
    pub fn new() -> Self {

    }

    pub fn set_cpus(&mut self, cpus: u64) {
        self.resources
            .cpu
            .attrs
            .insert("cgroup.procs".to_string(), cpus.to_string());
        self.cgroup.apply(&self.resources).unwrap();
    }
}*/
