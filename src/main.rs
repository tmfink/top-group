//! Prints information about memory usage of running processes

use std::ffi::OsStr;

use size_format;
use top_group::*;

fn main() {
    let procs_grouped = GroupedProcess::new().expect("Failed to get processes");
    println!("{:#?}", procs_grouped);

    let mut proc_group_usage: Vec<(&OsStr, u64)> = procs_grouped
        .name_to_group()
        .iter()
        .map(|(name, group)| (name.as_os_str(), group.usage_totals().memory))
        .collect();
    proc_group_usage.sort_unstable_by_key(|(name, usage)| -> (u64, &OsStr) { (*usage, *name) });
    for (name, usage) in proc_group_usage {
        let usage_bytes = usage * 1000;
        let usage = format!("{}B", size_format::SizeFormatterSI::new(usage_bytes));
        println!("{:30} {}", name.to_string_lossy(), usage);
    }
}
