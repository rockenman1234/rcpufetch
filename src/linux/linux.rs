use std::fs;
use std::collections::HashMap;
use std::process::Command;
use crate::art::logos::get_logo_lines_for_vendor;

/// Struct representing parsed Linux CPU information.
pub struct LinuxCpuInfo {
    /// CPU model name (e.g., "AMD Ryzen 5 9600X 6-Core Processor")
    model: String,
    /// CPU vendor ID (e.g., "AuthenticAMD", "GenuineIntel")
    vendor: String,
    /// CPU architecture (e.g., "x86_64")
    architecture: String,
    /// CPU byte order (e.g., "Little Endian")
    byte_order: String,
    /// CPU flags (e.g., "sse4_2 avx2")
    flags: String,
    /// Number of physical CPU cores
    physical_cores: u32,
    /// Number of logical CPU cores (threads)
    logical_cores: u32,
    /// Base CPU frequency in GHz (if available)
    base_mhz: Option<f32>,
    /// L1 data cache size (per core, total) in KB
    l1d_size: Option<(u32, u32)>, // (per core, total)
    /// L1 instruction cache size (per core, total) in KB
    l1i_size: Option<(u32, u32)>,
    /// L2 cache size (per core, total) in KB
    l2_size: Option<(u32, u32)>,
    /// L3 cache size (largest, total) in KB
    l3_size: Option<(u32, u32)>,
}

impl LinuxCpuInfo {
    /// Parse and return Linux CPU information from /proc/cpuinfo and sysfs.
    ///
    /// This function attempts to extract model name, vendor, core/thread counts, cache sizes, and max frequency.
    /// It prefers sysfs for cache and frequency info, falling back to /proc/cpuinfo if needed.
    pub fn new() -> Result<Self, String> {
        // Get CPU information using lscpu
        let output = Command::new("lscpu")
            .output()
            .map_err(|e| e.to_string())?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // Get architecture using uname
        let uname_output = Command::new("uname")
            .args(["-m"])
            .output()
            .map_err(|e| e.to_string())?;
        let architecture = String::from_utf8_lossy(&uname_output.stdout).trim().to_string();
        
        // Parse the output
        let mut model = String::new();
        let mut vendor = String::new();
        let mut byte_order = String::new();
        let mut flags = String::new();
        let mut physical_cores = 0;
        let mut logical_cores = 0;
        let mut base_mhz = None;
        let mut l1d_size = None;
        let mut l1i_size = None;
        let mut l2_size = None;
        let mut l3_size = None;

        let mut collecting_flags = false;
        for line in output_str.lines() {
            if line.starts_with("Model name:") {
                model = line.split("Model name:").nth(1).unwrap_or("").trim().to_string();
            } else if line.starts_with("Vendor ID:") {
                vendor = line.split("Vendor ID:").nth(1).unwrap_or("").trim().to_string();
            } else if line.starts_with("Byte Order:") {
                byte_order = line.split("Byte Order:").nth(1).unwrap_or("").trim().to_string();
            } else if line.starts_with("Flags:") {
                collecting_flags = true;
                flags = line.split("Flags:").nth(1).unwrap_or("").trim().to_string();
            } else if collecting_flags && line.starts_with(" ") {
                flags.push_str(" ");
                flags.push_str(line.trim());
            } else {
                collecting_flags = false;
            }
            if line.starts_with("CPU(s):") {
                logical_cores = line.split("CPU(s):").nth(1).unwrap_or("0").trim().parse().unwrap_or(0);
            } else if line.starts_with("Core(s) per socket:") {
                let cores_per_socket: u32 = line.split("Core(s) per socket:").nth(1).unwrap_or("0").trim().parse().unwrap_or(0);
                let sockets: u32 = output_str.lines()
                    .find(|l| l.starts_with("Socket(s):"))
                    .and_then(|l| l.split("Socket(s):").nth(1))
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(1);
                physical_cores = cores_per_socket * sockets;
            } else if line.starts_with("CPU max MHz:") {
                let mhz = line.split("CPU max MHz:").nth(1).unwrap_or("0").trim().parse::<f32>().unwrap_or(0.0);
                base_mhz = Some(mhz / 1000.0); // Convert MHz to GHz
            } else if line.starts_with("L1d cache:") {
                let parts: Vec<&str> = line.split("L1d cache:").nth(1).unwrap_or("").trim().split_whitespace().collect();
                if parts.len() >= 2 {
                    let size = parts[0].parse::<u32>().unwrap_or(0);
                    let instances = parts[1].trim_matches(|c| c == '(' || c == ')' || c == 'i' || c == 'n' || c == 's' || c == 't' || c == 'a' || c == 'c' || c == 'e' || c == 's').parse::<u32>().unwrap_or(0);
                    l1d_size = Some((size, size * instances));
                }
            } else if line.starts_with("L1i cache:") {
                let parts: Vec<&str> = line.split("L1i cache:").nth(1).unwrap_or("").trim().split_whitespace().collect();
                if parts.len() >= 2 {
                    let size = parts[0].parse::<u32>().unwrap_or(0);
                    let instances = parts[1].trim_matches(|c| c == '(' || c == ')' || c == 'i' || c == 'n' || c == 's' || c == 't' || c == 'a' || c == 'c' || c == 'e' || c == 's').parse::<u32>().unwrap_or(0);
                    l1i_size = Some((size, size * instances));
                }
            } else if line.starts_with("L2 cache:") {
                let parts: Vec<&str> = line.split("L2 cache:").nth(1).unwrap_or("").trim().split_whitespace().collect();
                if parts.len() >= 2 {
                    let size = parts[0].parse::<u32>().unwrap_or(0);
                    let instances = parts[1].trim_matches(|c| c == '(' || c == ')' || c == 'i' || c == 'n' || c == 's' || c == 't' || c == 'a' || c == 'c' || c == 'e' || c == 's').parse::<u32>().unwrap_or(0);
                    l2_size = Some((size, size * instances));
                }
            } else if line.starts_with("L3 cache:") {
                let parts: Vec<&str> = line.split("L3 cache:").nth(1).unwrap_or("").trim().split_whitespace().collect();
                if parts.len() >= 2 {
                    let size = parts[0].parse::<u32>().unwrap_or(0);
                    let instances = parts[1].trim_matches(|c| c == '(' || c == ')' || c == 'i' || c == 'n' || c == 's' || c == 't' || c == 'a' || c == 'c' || c == 'e' || c == 's').parse::<u32>().unwrap_or(0);
                    l3_size = Some((size, size * instances));
                }
            }
        }

        Ok(LinuxCpuInfo {
            model,
            vendor,
            architecture,
            byte_order,
            flags,
            physical_cores,
            logical_cores,
            base_mhz,
            l1d_size,
            l1i_size,
            l2_size,
            l3_size,
        })
    }

    /// Print the CPU information in a horizontally aligned format with the vendor logo.
    pub fn display_info(&self) {
        let logo_lines = get_logo_lines_for_vendor(&self.vendor).unwrap_or_else(|| vec![]);
        let mut info_lines = vec![
            format!("Name: {:<30}", self.model),
            format!("Architecture: {:<30}", self.architecture),
            format!("Byte Order: {:<30}", self.byte_order),
            format!("Vendor: {:<30}", self.vendor),
            format!("Max Frequency: {:>7}", match self.base_mhz { Some(ghz) => format!("{:.3} GHz", ghz), None => "Unknown".to_string() }),
            format!("Cores: {:>2} cores ({} threads)", self.physical_cores, self.logical_cores),
            format!("L1i Size: {}", match self.l1i_size { Some((per, _)) => format!("{}KB", per), None => "Unknown".to_string() }),
            format!("L1d Size: {}", match self.l1d_size { Some((per, _)) => format!("{}KB", per), None => "Unknown".to_string() }),
            format!("L2 Size: {}", match self.l2_size { Some((per, _)) => format!("{}KB", per), None => "Unknown".to_string() }),
            format!("L3 Size: {}", match self.l3_size { Some((per, _)) => format!("{}KB", per), None => "Unknown".to_string() }),
        ];

        // Wrap flags at 80 characters per line, indenting wrapped lines
        let wrap_width = 80;
        let flag_label = "Flags: ";
        let mut flag_lines = Vec::new();
        let mut current_line = String::from(flag_label);
        for word in self.flags.split_whitespace() {
            if current_line.len() + word.len() + 1 > wrap_width {
                flag_lines.push(current_line);
                current_line = format!("{:width$}{}", "", word, width=flag_label.len());
            } else {
                if current_line.trim_end().ends_with(":") {
                    current_line.push_str(word);
                } else {
                    current_line.push(' ');
                    current_line.push_str(word);
                }
            }
        }
        if !current_line.trim().is_empty() {
            flag_lines.push(current_line);
        }

        // Insert the flag lines right after the L3 cache line
        for (i, line) in flag_lines.into_iter().enumerate() {
            info_lines.push(line);
        }

        // Pad info_lines to at least as many as logo_lines for alignment
        let mut padded_info_lines = info_lines.clone();
        while padded_info_lines.len() < logo_lines.len() {
            padded_info_lines.push(String::new());
        }

        let logo_width = logo_lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        
        // Print logo and info side by side
        for (logo, info) in logo_lines.iter().zip(padded_info_lines.iter()) {
            println!("{:<width$}   {}", logo, info, width=logo_width);
        }

        // If info_lines is longer than logo_lines, print the rest
        if info_lines.len() > logo_lines.len() {
            for info in &info_lines[logo_lines.len()..] {
                println!("{:<width$}   {}", "", info, width=logo_width);
            }
        }
    }
}