//! Gets information about running processes grouped by name

use std::collections::HashMap;
use std::ffi::OsString;
use std::iter::Sum;
use std::ops::Add;

use procfs;

/// Memory usage statistics
#[derive(Debug, Clone, Copy, Default)]
pub struct MemoryUsage {
    /// (resident - shared) in kB
    pub memory: u64,

    /// Resident size in kB
    pub resident: u64,

    /// Shared size in kB
    pub shared: u64,
}

impl Add for MemoryUsage {
    type Output = MemoryUsage;

    fn add(self, other: MemoryUsage) -> MemoryUsage {
        MemoryUsage {
            memory: self.memory + other.memory,
            resident: self.resident + other.resident,
            shared: self.shared + other.shared,
        }
    }
}

impl Sum for MemoryUsage {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        iter.fold(MemoryUsage::default(), |acc, x| acc + x)
    }
}

/// Information about groups of processes with the same name
#[derive(Debug, Clone, Default)]
pub struct ProcessGroups {
    /// PID to memory usage mapping
    pid_to_usage: HashMap<i32, MemoryUsage>,

    /// Total memory usage for all PIDs
    usage_totals: MemoryUsage,
}

impl ProcessGroups {
    /// PID to memory usage mapping
    pub fn pid_to_usage(&self) -> &HashMap<i32, MemoryUsage> {
        &self.pid_to_usage
    }

    /// Total memory usage for all PIDs
    pub fn usage_totals(&self) -> MemoryUsage {
        self.usage_totals
    }

    fn add_usage(&mut self, pid: i32, usage: MemoryUsage) {
        self.pid_to_usage.insert(pid, usage);
        self.usage_totals = self.usage_totals + usage;
    }
}

/// Running processes grouped by exe name
#[derive(Debug, Clone, Default)]
pub struct GroupedProcess {
    /// Mapping from process name to usage
    name_to_group: HashMap<OsString, ProcessGroups>,
}

impl GroupedProcess {
    /// Creates a new `GroupedProcess` by querying all running processes
    pub fn new() -> Self {
        let procs = procfs::all_processes();
        let mut procs_grouped: HashMap<OsString, ProcessGroups> = HashMap::new();
        for proc in procs {
            let exe = if let Ok(exe) = proc.exe() {
                exe
            } else {
                continue;
            };
            let basename = exe.file_name().expect("Failed to get basename").to_owned();
            let status = if let Ok(status) = proc.status() {
                status
            } else {
                continue;
            };
            let resident = if let Some(resident) = status.vmrss {
                resident
            } else {
                continue;
            };
            let shared = status.rssshmem.unwrap();
            let memory = resident - shared;
            let usage = MemoryUsage {
                memory,
                resident,
                shared,
            };

            procs_grouped
                .entry(basename)
                .or_insert(Default::default())
                .add_usage(proc.pid(), usage);
        }

        GroupedProcess {
            name_to_group: procs_grouped,
        }
    }

    /// Name of process to process groups
    pub fn name_to_group(&self) -> &HashMap<OsString, ProcessGroups> {
        &self.name_to_group
    }
}
