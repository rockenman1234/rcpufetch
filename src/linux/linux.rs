use std::fs;
use std::collections::HashMap;
use std::process::Command;
use crate::art::logos::get_logo_lines_for_vendor;

/// Struct representing parsed Linux CPU information.
///
/// This struct contains comprehensive CPU information parsed from /proc/cpuinfo
/// and other system files, including detailed cache information, CPU flags,
/// core counts, and frequency data.
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
    /// Maximum CPU frequency in GHz (if available)
    max_mhz: Option<f32>,
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
    /// Parse and return Linux CPU information from /proc/cpuinfo and system files.
    ///
    /// This function reads directly from /proc/cpuinfo to extract CPU model name, vendor,
    /// core/thread counts, cache sizes, frequency information, CPU flags, and byte order.
    ///
    /// The function attempts to gather comprehensive CPU information by:
    /// - Parsing /proc/cpuinfo for basic CPU details, flags, and cache information
    /// - Using uname to determine system architecture
    /// - Reading cpufreq information for maximum frequency data
    /// - Calculating physical and logical core counts from processor entries
    ///
    /// # Returns
    ///
    /// Returns `Ok(LinuxCpuInfo)` on success, or `Err(String)` with error description on failure.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - /proc/cpuinfo cannot be read
    /// - uname command fails to execute
    /// - Critical CPU information cannot be parsed
    pub fn new() -> Result<Self, String> {
        // Read /proc/cpuinfo directly
        let cpuinfo_content = fs::read_to_string("/proc/cpuinfo")
        .map_err(|e| format!("Failed to read /proc/cpuinfo: {}", e))?;

        // Get architecture using uname
        let uname_output = Command::new("uname")
        .args(["-m"])
        .output()
        .map_err(|e| format!("Failed to get architecture: {}", e))?;
        let architecture = String::from_utf8_lossy(&uname_output.stdout).trim().to_string();

        // Parse /proc/cpuinfo
        let parsed_info = Self::parse_cpuinfo(&cpuinfo_content)?;

        // Get byte order information
        let byte_order = Self::get_byte_order();

        // Get maximum frequency
        let max_mhz = Self::get_max_frequency().or(parsed_info.max_mhz);

        // Get cache information from sysfs (fallback to /proc/cpuinfo values)
        let (l1d_size, l1i_size, l2_size, l3_size) = Self::get_cache_info()
        .unwrap_or((parsed_info.l1d_size, parsed_info.l1i_size, parsed_info.l2_size, parsed_info.l3_size));

        Ok(LinuxCpuInfo {
            model: parsed_info.model,
            vendor: parsed_info.vendor,
            architecture,
            byte_order,
            flags: parsed_info.flags,
            physical_cores: parsed_info.physical_cores,
            logical_cores: parsed_info.logical_cores,
            max_mhz,
            l1d_size,
            l1i_size,
            l2_size,
            l3_size,
        })
    }

    /// Parse CPU information from /proc/cpuinfo content.
    ///
    /// This function processes the raw content of /proc/cpuinfo and extracts
    /// relevant CPU information including model name, vendor, flags, core counts,
    /// and cache sizes.
    ///
    /// # Arguments
    ///
    /// * `content` - The raw string content of /proc/cpuinfo
    ///
    /// # Returns
    ///
    /// Returns a `ParsedCpuInfo` struct containing the extracted information,
    /// or an error string if parsing fails.
    fn parse_cpuinfo(content: &str) -> Result<ParsedCpuInfo, String> {
        let mut model = String::new();
        let mut vendor = String::new();
        let mut flags = String::new();
        let mut cache_size = None;
        let mut max_mhz = None;

        // Track unique physical IDs and core IDs for accurate counting
        let mut physical_ids = std::collections::HashSet::new();
        let mut core_ids = std::collections::HashSet::new();
        let mut logical_cores = 0;

        // Parse each processor entry
        for processor_block in content.split("\n\n") {
            if processor_block.trim().is_empty() {
                continue;
            }

            logical_cores += 1;
            let mut current_physical_id = None;
            let mut current_core_id = None;

            for line in processor_block.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                if let Some((key, value)) = line.split_once(':') {
                    let key = key.trim();
                    let value = value.trim();

                    match key {
                        "model name" => {
                            if model.is_empty() {
                                model = value.to_string();
                            }
                        },
                        "vendor_id" => {
                            if vendor.is_empty() {
                                vendor = value.to_string();
                            }
                        },
                        "flags" => {
                            if flags.is_empty() {
                                flags = value.to_string();
                            }
                        },
                        "cache size" => {
                            if cache_size.is_none() {
                                // Parse cache size (e.g., "1024 KB" -> 1024)
                                if let Some(size_str) = value.split_whitespace().next() {
                                    cache_size = size_str.parse::<u32>().ok();
                                }
                            }
                        },
                        "cpu MHz" => {
                            // Track the highest frequency seen
                            if let Ok(mhz) = value.parse::<f32>() {
                                max_mhz = Some(max_mhz.map_or(mhz, |current: f32| current.max(mhz)));
                            }
                        },
                        "physical id" => {
                            if let Ok(id) = value.parse::<u32>() {
                                current_physical_id = Some(id);
                            }
                        },
                        "core id" => {
                            if let Ok(id) = value.parse::<u32>() {
                                current_core_id = Some(id);
                            }
                        },
                        _ => {}
                    }
                }
            }

            // Track unique physical and core IDs
            if let Some(phys_id) = current_physical_id {
                physical_ids.insert(phys_id);
            }
            if let (Some(phys_id), Some(core_id)) = (current_physical_id, current_core_id) {
                core_ids.insert((phys_id, core_id));
            }
        }

        // Calculate physical cores
        let physical_cores = if !core_ids.is_empty() {
            core_ids.len() as u32
        } else if !physical_ids.is_empty() {
            // Fallback: assume single core per physical ID if core IDs aren't available
            physical_ids.len() as u32
        } else {
            // Last resort: assume single physical core
            1
        };

        // Convert max MHz to GHz
        let max_mhz = max_mhz.map(|mhz| mhz / 1000.0);

        // For cache sizes, we'll use the cache size from /proc/cpuinfo as L2 cache
        // and try to infer other cache levels (this is a limitation of /proc/cpuinfo)
        let l2_size = cache_size.map(|size| (size, size * physical_cores));

        Ok(ParsedCpuInfo {
            model,
            vendor,
            flags,
            physical_cores,
            logical_cores,
            max_mhz,
            l1d_size: None, // Not typically available in /proc/cpuinfo
            l1i_size: None, // Not typically available in /proc/cpuinfo
            l2_size,
            l3_size: None, // Not typically available in /proc/cpuinfo
        })
    }

    /// Determine the system's byte order.
    ///
    /// This function determines whether the system uses little-endian or big-endian
    /// byte ordering by checking the native byte order of the current architecture.
    ///
    /// # Returns
    ///
    /// Returns a string indicating the byte order: "Little Endian" or "Big Endian".
    fn get_byte_order() -> String {
        if cfg!(target_endian = "little") {
            "Little Endian".to_string()
        } else {
            "Big Endian".to_string()
        }
    }

    /// Get maximum CPU frequency from cpufreq information.
    ///
    /// This function attempts to read the maximum CPU frequency from the Linux
    /// cpufreq subsystem by checking scaling_max_freq files for all CPU cores.
    ///
    /// # Returns
    ///
    /// Returns `Some(f32)` with the maximum frequency in GHz if available,
    /// or `None` if the information cannot be read.
    fn get_max_frequency() -> Option<f32> {
        // Try to read from cpufreq
        if let Ok(entries) = fs::read_dir("/sys/devices/system/cpu") {
            let mut max_freq = 0u64;

            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("cpu") && name[3..].chars().all(|c| c.is_ascii_digit()) {
                        let freq_path = path.join("cpufreq/scaling_max_freq");
                        if let Ok(freq_str) = fs::read_to_string(&freq_path) {
                            if let Ok(freq) = freq_str.trim().parse::<u64>() {
                                max_freq = max_freq.max(freq);
                            }
                        }
                    }
                }
            }

            if max_freq > 0 {
                // Convert from kHz to GHz
                return Some((max_freq as f32) / 1_000_000.0);
            }
        }

        None
    }

    /// Get detailed cache information from sysfs.
    ///
    /// This function reads cache information directly from the Linux sysfs filesystem
    /// at `/sys/devices/system/cpu/cpu*/cache/index*/` to get accurate cache sizes
    /// for all cache levels. It reads cache information from cpu0 only to avoid
    /// double-counting, then calculates totals based on sharing characteristics:
    /// - L1 and L2 caches are typically per-core, so multiply by physical core count
    /// - L3 cache is typically shared across all cores
    ///
    /// # Returns
    ///
    /// Returns a tuple of optional cache sizes in the format:
    /// `(L1d, L1i, L2, L3)` where each element is `Option<(per_core_kb, total_kb)>`
    /// Only total cache sizes are reported for each level.
    fn get_cache_info() -> Option<(Option<(u32, u32)>, Option<(u32, u32)>, Option<(u32, u32)>, Option<(u32, u32)>)> {
        use std::collections::HashMap;
        
        let mut cache_sizes: HashMap<String, u32> = HashMap::new();
        
        // Read cache information from cpu0 only to avoid double-counting
        let cpu0_cache_dir = std::path::Path::new("/sys/devices/system/cpu/cpu0/cache");
        if let Ok(cache_entries) = fs::read_dir(cpu0_cache_dir) {
            for cache_entry in cache_entries.flatten() {
                let cache_path = cache_entry.path();
                if let Some(index_name) = cache_path.file_name().and_then(|n| n.to_str()) {
                    if index_name.starts_with("index") {
                        // Read cache level, type, and size
                        let level_path = cache_path.join("level");
                        let type_path = cache_path.join("type");
                        let size_path = cache_path.join("size");
                        
                        if let (Ok(level_str), Ok(type_str), Ok(size_str)) = (
                            fs::read_to_string(&level_path),
                            fs::read_to_string(&type_path),
                            fs::read_to_string(&size_path)
                        ) {
                            let level = level_str.trim();
                            let cache_type = type_str.trim();
                            let size_str = size_str.trim();
                            
                            // Parse size (e.g., "32K" -> 32, "1024K" -> 1024)
                            if let Some(size_kb) = Self::parse_cache_size(size_str) {
                                let cache_key = format!("L{}_{}", level, cache_type);
                                cache_sizes.insert(cache_key, size_kb);
                            }
                        }
                    }
                }
            }
        }
        
        // Get physical core count for calculating totals
        let physical_cores = Self::get_physical_core_count().unwrap_or(1);
        
        // Calculate totals based on cache sharing characteristics
        let l1d_total = cache_sizes.get("L1_Data")
            .map(|&size| size * physical_cores); // L1 data is per-core
        let l1i_total = cache_sizes.get("L1_Instruction") 
            .map(|&size| size * physical_cores); // L1 instruction is per-core
        let l2_total = cache_sizes.get("L2_Unified")
            .map(|&size| size * physical_cores); // L2 is typically per-core
        let l3_total = cache_sizes.get("L3_Unified")
            .copied(); // L3 is typically shared across all cores
        
        Some((
            l1d_total.map(|total| (0, total)), // Only report total, per-core not used
            l1i_total.map(|total| (0, total)),
            l2_total.map(|total| (0, total)),
            l3_total.map(|total| (0, total)),
        ))
    }

    /// Parse cache size string from sysfs.
    ///
    /// This helper function parses cache size strings from sysfs files,
    /// which can be in formats like "32K", "1024K", "32768K", etc.
    ///
    /// # Arguments
    ///
    /// * `size_str` - Cache size string from sysfs (e.g., "32K")
    ///
    /// # Returns
    ///
    /// Returns the cache size in kilobytes, or `None` if parsing fails.
    fn parse_cache_size(size_str: &str) -> Option<u32> {
        if size_str.ends_with('K') {
            size_str[..size_str.len() - 1].parse::<u32>().ok()
        } else if size_str.ends_with("KB") {
            size_str[..size_str.len() - 2].parse::<u32>().ok()
        } else {
            // Try parsing as plain number (assume KB)
            size_str.parse::<u32>().ok()
        }
    }

    /// Print the CPU information in a horizontally aligned format with the vendor logo.
    ///
    /// This function displays comprehensive CPU information in a formatted layout
    /// alongside the appropriate vendor logo. The output includes CPU model, architecture,
    /// vendor information, frequency data, core counts, cache sizes, and CPU flags.
    ///
    /// The CPU flags are automatically wrapped to fit within the display width,
    /// and all information is aligned for easy reading.
    pub fn display_info(&self) {
        let logo_lines = get_logo_lines_for_vendor(&self.vendor).unwrap_or_else(|| vec![]);
        let info_lines = vec![
            format!("Name: {:<30}", self.model),
                format!("Architecture: {:<30}", self.architecture),
                    format!("Byte Order: {:<30}", self.byte_order),
                        format!("Vendor: {:<30}", self.vendor),
                            format!("Max Frequency: {:>7}", match self.max_mhz { Some(ghz) => format!("{:.3} GHz", ghz), None => "Unknown".to_string() }),
                                format!("Cores: {:>2} cores ({} threads)", self.physical_cores, self.logical_cores),
                                    format!("L1i Size: {}", match self.l1i_size { Some((_, total)) => Self::format_cache_size(total), None => "Unknown".to_string() }),
                                        format!("L1d Size: {}", match self.l1d_size { Some((_, total)) => Self::format_cache_size(total), None => "Unknown".to_string() }),
                                            format!("L1 Size: {}", match (self.l1i_size, self.l1d_size) {
                                                (Some((_, l1i_total)), Some((_, l1d_total))) => Self::format_cache_size(l1i_total + l1d_total),
                                                (Some((_, l1i_total)), None) => Self::format_cache_size(l1i_total),
                                                (None, Some((_, l1d_total))) => Self::format_cache_size(l1d_total),
                                                (None, None) => "Unknown".to_string()
                                            }),
                                                format!("L2 Size: {}", match self.l2_size { Some((_, total)) => Self::format_cache_size(total), None => "Unknown".to_string() }),
                                                    format!("L3 Size: {}", match self.l3_size { Some((_, total)) => Self::format_cache_size(total), None => "Unknown".to_string() }),
        ];

        let logo_width = logo_lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        let sep = "   ";
        let left_margin = logo_width + sep.len();
        let total_width = 100; // You can adjust this to your preferred terminal width
        let flag_label = "Flags: ";
        let indent = "       "; // 7 spaces
        let wrap_width = total_width - left_margin;

        // Wrap flags so that each line starts at the info column
        let mut flag_lines = Vec::new();
        let mut current_line = String::from(flag_label);
        for word in self.flags.split_whitespace() {
            if current_line.len() + word.len() + 1 > wrap_width {
                flag_lines.push(current_line);
                current_line = format!("{}{}", indent, word);
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

        let info_len = info_lines.len();
        let logo_len = logo_lines.len();
        let max_lines = std::cmp::max(logo_len, info_len + flag_lines.len());

        // Print logo and info side by side for overlapping lines
        let mut info_idx = 0;
        for i in 0..max_lines {
            let logo = logo_lines.get(i).map(|s| s.as_str()).unwrap_or("");
            let info = if info_idx < info_lines.len() {
                let s = &info_lines[info_idx];
                info_idx += 1;
                s.as_str().to_string()
            } else if !flag_lines.is_empty() {
                flag_lines.remove(0)
            } else {
                String::new()
            };
            println!("{:<width$}{}{}", logo, sep, info.as_str(), width=logo_width);
        }
    }

    /// Format cache size with appropriate units (KB or MB).
    ///
    /// This helper function formats cache sizes in a human-readable format,
    /// converting sizes above 1000KB to megabytes with decimal precision.
    ///
    /// # Arguments
    ///
    /// * `size_kb` - Cache size in kilobytes
    ///
    /// # Returns
    ///
    /// Returns a formatted string with appropriate units (e.g., "288KB" or "6.0MB")
    fn format_cache_size(size_kb: u32) -> String {
        if size_kb >= 1000 {
            let size_mb = size_kb as f32 / 1024.0;
            format!("{:.1}MB", size_mb)
        } else {
            format!("{}KB", size_kb)
        }
    }

    /// Get the number of physical CPU cores from /proc/cpuinfo.
    ///
    /// This helper function determines the number of physical cores by parsing
    /// /proc/cpuinfo and counting unique (physical_id, core_id) pairs.
    ///
    /// # Returns
    ///
    /// Returns the number of physical cores, or `None` if the count cannot be determined.
    fn get_physical_core_count() -> Option<u32> {
        let cpuinfo_content = fs::read_to_string("/proc/cpuinfo").ok()?;
        let mut physical_ids = std::collections::HashSet::new();
        let mut core_ids = std::collections::HashSet::new();
        
        for processor_block in cpuinfo_content.split("\n\n") {
            if processor_block.trim().is_empty() {
                continue;
            }
            
            let mut current_physical_id = None;
            let mut current_core_id = None;
            
            for line in processor_block.lines() {
                if let Some((key, value)) = line.split_once(':') {
                    match key.trim() {
                        "physical id" => {
                            if let Ok(id) = value.trim().parse::<u32>() {
                                current_physical_id = Some(id);
                            }
                        },
                        "core id" => {
                            if let Ok(id) = value.trim().parse::<u32>() {
                                current_core_id = Some(id);
                            }
                        },
                        _ => {}
                    }
                }
            }
            
            if let Some(phys_id) = current_physical_id {
                physical_ids.insert(phys_id);
            }
            if let (Some(phys_id), Some(core_id)) = (current_physical_id, current_core_id) {
                core_ids.insert((phys_id, core_id));
            }
        }
        
        if !core_ids.is_empty() {
            Some(core_ids.len() as u32)
        } else if !physical_ids.is_empty() {
            Some(physical_ids.len() as u32)
        } else {
            Some(1) // Default fallback
        }
    }
}

/// Intermediate struct for holding parsed CPU information from /proc/cpuinfo.
///
/// This struct is used internally during the parsing process to collect
/// information before creating the final `LinuxCpuInfo` struct.
struct ParsedCpuInfo {
    /// CPU model name
    model: String,
    /// CPU vendor ID
    vendor: String,
    /// CPU flags string
    flags: String,
    /// Number of physical CPU cores
    physical_cores: u32,
    /// Number of logical CPU cores (threads)
    logical_cores: u32,
    /// Maximum CPU frequency in GHz
    max_mhz: Option<f32>,
    /// L1 data cache information
    l1d_size: Option<(u32, u32)>,
    /// L1 instruction cache information
    l1i_size: Option<(u32, u32)>,
    /// L2 cache information
    l2_size: Option<(u32, u32)>,
    /// L3 cache information
    l3_size: Option<(u32, u32)>,
}
