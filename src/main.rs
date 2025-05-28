mod linux; // Declares the linux module (src/linux/mod.rs)
mod art; // Add this to declare the art module

// Use the LinuxCpuInfo struct from the nested linux module
use crate::linux::linux::LinuxCpuInfo;

fn main() {

    // TODO: Check for current OS. then display the logo for the current OS.
    // For now, we will just use the LinuxCpuInfo struct to display CPU information.
    // This reads from /proc/cpuinfo and displays the CPU model, vendor, and logical processor count.
    match LinuxCpuInfo::new() {
        Ok(cpu_info) => {
            cpu_info.display_info();
        }
        Err(e) => {
            eprintln!("Error fetching CPU info: {}", e);
        }
    }
}
