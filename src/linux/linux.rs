use std::fs;
use std::collections::HashMap;
use crate::art;

/// Struct representing parsed Linux CPU information.
pub struct LinuxCpuInfo {
    /// CPU model name (e.g., "AMD Ryzen 5 9600X 6-Core Processor")
    model_name: String,
    /// CPU vendor ID (e.g., "AuthenticAMD", "GenuineIntel")
    vendor_id: String,
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
    /// Parse and return Linux CPU information from /proc/cpuinfo and sysfs.
    ///
    /// This function attempts to extract model name, vendor, core/thread counts, cache sizes, and max frequency.
    /// It prefers sysfs for cache and frequency info, falling back to /proc/cpuinfo if needed.
    pub fn new() -> Result<Self, String> {
        let cpuinfo_content = fs::read_to_string("/proc/cpuinfo")
            .map_err(|e| format!("Failed to read /proc/cpuinfo: {}", e))?;

        let mut model_name = String::new();
        let mut vendor_id = String::new();
        let mut max_mhz: Option<f32> = None;
        let mut cpu_cores = None;
        let mut siblings = None;
        let mut l2_sizes = Vec::new();
        let mut l3_sizes = Vec::new();
        let mut l1d_sizes = Vec::new();
        let mut l1i_sizes = Vec::new();
        let mut logical_cores = 0;

        // For AMD/Intel, L1i/L1d/L2/L3 are not always in /proc/cpuinfo, but some distros add them
        // We'll look for lines like: 'L1d cache', 'L1i cache', 'L2 cache', 'L3 cache'
        // Otherwise, fallback to 'cache size' (usually L2)
        for line in cpuinfo_content.lines() {
            if line.starts_with("model name") && model_name.is_empty() {
                model_name = line.split(':').nth(1).unwrap_or("").trim().to_string();
            } else if line.starts_with("vendor_id") && vendor_id.is_empty() {
                vendor_id = line.split(':').nth(1).unwrap_or("").trim().to_string();
            } else if line.starts_with("cpu cores") && cpu_cores.is_none() {
                cpu_cores = line.split(':').nth(1).and_then(|s| s.trim().parse::<u32>().ok());
            } else if line.starts_with("siblings") && siblings.is_none() {
                siblings = line.split(':').nth(1).and_then(|s| s.trim().parse::<u32>().ok());
            } else if line.starts_with("cpu MHz") {
                if let Some(mhz_str) = line.split(':').nth(1) {
                    if let Ok(mhz) = mhz_str.trim().parse::<f32>() {
                        max_mhz = Some(max_mhz.map_or(mhz, |cur| cur.max(mhz)));
                    }
                }
            } else if line.starts_with("cache size") {
                // This is usually L2 per core
                if let Some(size_str) = line.split(':').nth(1) {
                    let size = size_str.trim().split_whitespace().next().unwrap_or("");
                    if let Ok(kb) = size.parse::<u32>() {
                        l2_sizes.push(kb);
                    }
                }
            } else if line.starts_with("L1d cache") {
                if let Some(size_str) = line.split(':').nth(1) {
                    let size = size_str.trim().split_whitespace().next().unwrap_or("");
                    if let Ok(kb) = size.parse::<u32>() {
                        l1d_sizes.push(kb);
                    }
                }
            } else if line.starts_with("L1i cache") {
                if let Some(size_str) = line.split(':').nth(1) {
                    let size = size_str.trim().split_whitespace().next().unwrap_or("");
                    if let Ok(kb) = size.parse::<u32>() {
                        l1i_sizes.push(kb);
                    }
                }
            } else if line.starts_with("L3 cache") {
                if let Some(size_str) = line.split(':').nth(1) {
                    let size = size_str.trim().split_whitespace().next().unwrap_or("");
                    if let Ok(kb) = size.parse::<u32>() {
                        l3_sizes.push(kb);
                    }
                }
            } else if line.starts_with("processor") {
                logical_cores += 1;
            }
        }

        // Try to read cache sizes from /sys/devices/system/cpu/cpu0/cache/index*/
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        struct CacheKey {
            level: u32,
            typ: String,
        }
        let mut sysfs_cache: HashMap<CacheKey, Vec<u32>> = HashMap::new();
        if let Ok(entries) = fs::read_dir("/sys/devices/system/cpu/cpu0/cache/") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && path.file_name().and_then(|n| n.to_str()).map_or(false, |n| n.starts_with("index")) {
                    let type_path = path.join("type");
                    let size_path = path.join("size");
                    let level_path = path.join("level");
                    if let (Ok(cache_type), Ok(size_str), Ok(level_str)) = (fs::read_to_string(&type_path), fs::read_to_string(&size_path), fs::read_to_string(&level_path)) {
                        let cache_type = cache_type.trim().to_string();
                        let size_str = size_str.trim();
                        let level = level_str.trim().parse::<u32>().unwrap_or(0);
                        // size_str is like "32K" or "512K" or "16M"
                        let kb = if let Some(stripped) = size_str.strip_suffix('K') {
                            stripped.parse::<u32>().ok()
                        } else if let Some(stripped) = size_str.strip_suffix('M') {
                            stripped.parse::<u32>().ok().map(|m| m * 1024)
                        } else {
                            None
                        };
                        if let Some(kb) = kb {
                            sysfs_cache.entry(CacheKey { level, typ: cache_type }).or_default().push(kb);
                        }
                    }
                }
            }
        }
        // Helper to get per-core and total for a given level/type
        let cache_info = |level: u32, typ: &str| -> Option<(u32, u32)> {
            let key = CacheKey { level, typ: typ.to_string() };
            sysfs_cache.get(&key).map(|sizes| {
                let per = *sizes.iter().max().unwrap_or(&0);
                let total = sizes.iter().sum();
                (per, total)
            })
        };
        // L1i: level 1, type Instruction
        let l1i = cache_info(1, "Instruction")
            .or_else(|| if !l1i_sizes.is_empty() { let per = *l1i_sizes.iter().max().unwrap(); let total = l1i_sizes.iter().sum(); Some((per, total)) } else { None });
        // L1d: level 1, type Data
        let l1d = cache_info(1, "Data")
            .or_else(|| if !l1d_sizes.is_empty() { let per = *l1d_sizes.iter().max().unwrap(); let total = l1d_sizes.iter().sum(); Some((per, total)) } else { None });
        // L2: level 2, type Unified
        let l2 = cache_info(2, "Unified")
            .or_else(|| if !l2_sizes.is_empty() { let per = *l2_sizes.iter().max().unwrap(); let total = l2_sizes.iter().sum(); Some((per, total)) } else { None });
        // L3: level 3, type Unified (show only the largest, not sum)
        let l3 = cache_info(3, "Unified")
            .map(|(per, _)| (per, per))
            .or_else(|| if !l3_sizes.is_empty() { let per = *l3_sizes.iter().max().unwrap(); Some((per, per)) } else { None });

        let physical_cores = cpu_cores.unwrap_or(1);
        let logical_cores = siblings.unwrap_or(logical_cores);

        if model_name.is_empty() && vendor_id.is_empty() && logical_cores == 0 {
            return Err("Could not parse essential CPU info from /proc/cpuinfo".to_string());
        }

        // Try to get max frequency from /sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq (in kHz)
        let sysfs_max_freq = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq")
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .map(|khz| khz as f32 / 1_000_000.0); // GHz
        // Try to get base frequency from /sys/devices/system/cpu/cpu0/cpufreq/base_frequency (in kHz)
        let sysfs_base_freq = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/base_frequency")
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .map(|khz| khz as f32 / 1_000_000.0); // GHz

        let max_mhz = sysfs_max_freq.or(max_mhz.map(|mhz| mhz / 1000.0));

        Ok(LinuxCpuInfo {
            model_name,
            vendor_id,
            physical_cores,
            logical_cores,
            max_mhz,
            l1d_size: l1d,
            l1i_size: l1i,
            l2_size: l2,
            l3_size: l3,
        })
    }

    /// Print the CPU information in a horizontally aligned format with the vendor logo.
    ///
    /// This function prints the logo (if available) and all CPU info fields, aligned for easy reading.
    pub fn display_info(&self) {
        use crate::art::logos::get_logo_lines_for_vendor;
        let logo_lines = get_logo_lines_for_vendor(&self.vendor_id).unwrap_or_else(|| vec![]);
        // Try to get base frequency again for display
        let base_freq = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/base_frequency")
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .map(|khz| khz as f32 / 1_000_000.0);
        let mut info_lines = vec![
            format!("Name: {:<30}", self.model_name),
            format!("Vendor: {:<30}", self.vendor_id),
            format!("Max Frequency: {:>7}", match self.max_mhz { Some(ghz) => format!("{:.3} GHz", ghz), None => "Unknown".to_string() }),
            format!("Cores: {:>2} cores ({} threads)", self.physical_cores, self.logical_cores),
            format!("L1i Size: {}", match self.l1i_size { Some((per, total)) => format!("{}KB ({}KB Total)", per, total), None => "Unknown".to_string() }),
            format!("L1d Size: {}", match self.l1d_size { Some((per, total)) => format!("{}KB ({}KB Total)", per, total), None => "Unknown".to_string() }),
            format!("L2 Size: {}", match self.l2_size { Some((per, total)) => format!("{}KB ({}KB Total)", per, total), None => "Unknown".to_string() }),
            format!("L3 Size: {}", match self.l3_size { Some((per, total)) => format!("{}KB ({}KB Total)", per, total), None => "Unknown".to_string() }),
        ];
        // Pad info_lines to at least as many as logo_lines for alignment
        while info_lines.len() < logo_lines.len() {
            info_lines.push(String::new());
        }
        let logo_width = logo_lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        for (logo, info) in logo_lines.iter().zip(info_lines.iter()) {
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