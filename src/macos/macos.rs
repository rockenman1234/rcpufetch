use crate::art::logos::get_logo_lines_for_vendor;
use std::process::Command;
pub struct MacOSCpuInfo {
    model: String,
    vendor: String,
    architecture: String,
    byte_order: String,
    physical_cores: u32,
    logical_cores: u32,
    base_mhz: Option<f32>,
    l1_size: Option<(u32, u32)>,
    l2_size: Option<(u32, u32)>,
    l3_size: Option<(u32, u32)>,
    flags: String,
}

impl MacOSCpuInfo {
    pub fn new() -> Result<Self, String> {
        // Get CPU brand string
        let model = Self::get_sysctl_string("machdep.cpu.brand_string")?;
        
        // Get architecture using uname -m
        let architecture = Self::get_architecture()?;
        
        // Get byte order from sysctl and format it
        let byte_order = Self::get_sysctl_string("hw.byteorder")
            .map(|order| {
                match order.trim() {
                    "1234" => "Little Endian".to_string(),
                    "4321" => "Big Endian".to_string(),
                    _ => format!("Unknown ({})", order)
                }
            })
            .unwrap_or_else(|_| "Unknown".to_string());
        
        // Determine vendor from brand string
        let vendor = if model.to_lowercase().contains("intel") {
            "Intel".to_string()
        } else if model.to_lowercase().contains("amd") {
            "AMD".to_string()
        } else if model.to_lowercase().contains("apple") {
            "Apple".to_string()
        } else {
            "Unknown".to_string()
        };
        
        // Get core counts
        let physical_cores = Self::get_sysctl_u32("machdep.cpu.core_count")
            .unwrap_or_else(|_| Self::get_sysctl_u32("machdep.cpu.cores_per_package").unwrap_or(0));
        let logical_cores = Self::get_sysctl_u32("machdep.cpu.thread_count")
            .unwrap_or_else(|_| Self::get_sysctl_u32("machdep.cpu.logical_per_package").unwrap_or(physical_cores));
        
        // Get base frequency (if available)
        let base_mhz = Self::get_sysctl_string("machdep.cpu.max_basic")
            .ok()
            .and_then(|s| s.parse::<f32>().ok());
        
        // Parse cache information - prefer detailed perflevel cache info for Apple Silicon
        let (l1_size, l2_size, l3_size) = Self::get_cache_info();
        
        // Get CPU flags
        let flags = Self::get_cpu_flags();
        
        Ok(Self {
            model,
            vendor,
            architecture,
            byte_order,
            physical_cores,
            logical_cores,
            base_mhz,
            l1_size,
            l2_size,
            l3_size,
            flags,
        })
    }
    
    /// Helper function to format cache size with appropriate units (KB or MB)
    fn format_cache_size(size_kb: u32) -> String {
        if size_kb >= 1000 {
            format!("{:.1}MB", size_kb as f32 / 1024.0)
        } else {
            format!("{}KB", size_kb)
        }
    }

    /// Helper function to get comprehensive cache information
    fn get_cache_info() -> (Option<(u32, u32)>, Option<(u32, u32)>, Option<(u32, u32)>) {
        // First try the traditional hw.cachesize approach
        let cache_sizes = Self::get_sysctl_string("hw.cachesize").unwrap_or_default();
        let cache_config = Self::get_sysctl_string("hw.cacheconfig").unwrap_or_default();
        
        let size_parts: Vec<&str> = cache_sizes.split_whitespace().collect();
        let config_parts: Vec<&str> = cache_config.split_whitespace().collect();
        
        let l1_size = if size_parts.len() >= 2 && config_parts.len() >= 2 {
            let size_bytes = size_parts[1].parse::<u32>().unwrap_or(0);
            let count = config_parts[1].parse::<u32>().unwrap_or(0);
            if size_bytes > 0 && count > 0 { 
                Some((size_bytes / 1024, count)) // Convert bytes to KB
            } else { None }
        } else { None };
        
        let l2_size = if size_parts.len() >= 3 && config_parts.len() >= 3 {
            let size_bytes = size_parts[2].parse::<u32>().unwrap_or(0);
            let count = config_parts[2].parse::<u32>().unwrap_or(0);
            if size_bytes > 0 && count > 0 { 
                Some((size_bytes / 1024, count)) // Convert bytes to KB
            } else { None }
        } else { None };
        
        let mut l3_size = if size_parts.len() >= 4 && config_parts.len() >= 4 {
            let size_bytes = size_parts[3].parse::<u32>().unwrap_or(0);
            let count = config_parts[3].parse::<u32>().unwrap_or(0);
            if size_bytes > 0 && count > 0 { 
                Some((size_bytes / 1024, count)) // Convert bytes to KB
            } else { None }
        } else { None };
        
        // For Apple Silicon, if L3 is not available from hw.cachesize, check performance level caches
        if l3_size.is_none() {
            // Check if we have performance level cache information (Apple Silicon)
            let perf0_l2 = Self::get_sysctl_u32("hw.perflevel0.l2cachesize").ok();
            let perf1_l2 = Self::get_sysctl_u32("hw.perflevel1.l2cachesize").ok();
            
            if let (Some(p0_l2), Some(p1_l2)) = (perf0_l2, perf1_l2) {
                // If we have different performance levels with different L2 sizes,
                // report the larger one as "shared cache" equivalent
                if p0_l2 != p1_l2 {
                    let larger_cache = std::cmp::max(p0_l2, p1_l2);
                    l3_size = Some((larger_cache / 1024, 1)); // Convert bytes to KB, show as 1 unit
                }
            }
        }
        
        (l1_size, l2_size, l3_size)
    }

    /// Helper function to get a string value from sysctl
    fn get_sysctl_string(key: &str) -> Result<String, String> {
        let output = Command::new("sysctl")
            .arg("-n")
            .arg(key)
            .output()
            .map_err(|e| format!("Failed to execute sysctl: {}", e))?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(format!("sysctl command failed for key: {}", key))
        }
    }
    
    /// Helper function to get a u32 value from sysctl
    fn get_sysctl_u32(key: &str) -> Result<u32, String> {
        let value_str = Self::get_sysctl_string(key)?;
        value_str.parse::<u32>()
            .map_err(|e| format!("Failed to parse '{}' as u32: {}", value_str, e))
    }

    /// Get system architecture using uname -m
    fn get_architecture() -> Result<String, String> {
        let output = Command::new("uname")
            .arg("-m")
            .output()
            .map_err(|e| format!("Failed to execute uname: {}", e))?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err("uname command failed".to_string())
        }
    }

    /// Get CPU flags from sysctl hw.optional.arm.* keys.
    ///
    /// This function queries all available ARM CPU feature flags via sysctl and returns
    /// a comma-separated string of enabled features, similar to Linux /proc/cpuinfo flags.
    /// Only features with value '1' (enabled) are included in the output.
    ///
    /// # Returns
    ///
    /// Returns a comma-separated string of enabled CPU feature flags (e.g., "FEAT_AES,FEAT_SHA256,FEAT_CRC32")
    /// or an empty string if no flags are available or if not running on ARM architecture.
    fn get_cpu_flags() -> String {
        // Try to get a list of all hw.optional.arm.* sysctl keys
        let output = Command::new("sysctl")
            .arg("hw.optional.arm.")
            .output();
        
        match output {
            Ok(result) if result.status.success() => {
                let output_str = String::from_utf8_lossy(&result.stdout);
                let mut enabled_flags = Vec::new();
                
                for line in output_str.lines() {
                    if let Some((key, value)) = line.split_once(": ") {
                        // Parse the value - only include flags that are enabled (value = 1)
                        if value.trim() == "1" {
                            // Extract the flag name from the key (everything after "hw.optional.arm.")
                            if let Some(flag_name) = key.strip_prefix("hw.optional.arm.") {
                                enabled_flags.push(flag_name.to_string());
                            }
                        }
                    }
                }
                
                enabled_flags.join(",")
            }
            _ => String::new() // Return empty string if sysctl fails (e.g., not ARM architecture)
        }
    }

    /// Display CPU information with logo (side-by-side layout).
    ///
    /// This function displays comprehensive CPU information alongside a vendor logo
    /// in a side-by-side layout. The logo can be overridden to display a different
    /// vendor's logo regardless of the actual CPU vendor.
    pub fn display_info_with_logo(&self, logo_override: Option<&str>) {
        let vendor_to_use = logo_override.unwrap_or(&self.vendor);
        let logo_lines = get_logo_lines_for_vendor(vendor_to_use).unwrap_or_else(|| vec![]);
        
        let mut info_lines = self.get_info_lines();
        
        // Handle flags wrapping
        if !self.flags.is_empty() {
            let logo_width = logo_lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
            let sep = "   ";
            let left_margin = logo_width + sep.len();
            let total_width = 100; // Terminal width
            let wrap_width = total_width - left_margin;
            
            // Wrap flags text
            let flag_label = "Flags: ";
            let indent = "       "; // 7 spaces to align with "Flags: "
            let mut flag_lines = Vec::new();
            let mut current_line = String::from(flag_label);
            
            for word in self.flags.split(',') {
                let word = word.trim();
                if current_line.len() + word.len() + 2 > wrap_width { // +2 for ", "
                    flag_lines.push(current_line);
                    current_line = format!("{}{}", indent, word);
                } else {
                    if current_line.trim_end().ends_with(":") {
                        current_line.push_str(word);
                    } else {
                        current_line.push_str(", ");
                        current_line.push_str(word);
                    }
                }
            }
            if !current_line.trim().is_empty() {
                flag_lines.push(current_line);
            }
            
            // Add flag lines to info_lines
            info_lines.extend(flag_lines);
        }
        
        let logo_width = logo_lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        let sep = "   ";
        let max_lines = std::cmp::max(logo_lines.len(), info_lines.len());

        // Print logo and info side by side
        for i in 0..max_lines {
            let logo = logo_lines.get(i).map(|s| s.as_str()).unwrap_or("");
            let info = info_lines.get(i).map(|s| s.as_str()).unwrap_or("");
            println!("{:<width$}{}{}", logo, sep, info, width=logo_width);
        }
    }

    /// Display CPU information without any logo.
    ///
    /// This function displays comprehensive CPU information in a simple list format
    /// without any vendor logo or side-by-side alignment.
    pub fn display_info_no_logo(&self) {
        let info_lines = self.get_info_lines();
        
        // Print CPU information without logo
        for line in info_lines {
            println!("{}", line);
        }
        
        // Print flags with wrapping
        if !self.flags.is_empty() {
            print!("Flags: ");
            let wrap_width = 80; // Standard terminal width
            let mut current_line_len = 7; // "Flags: " length
            let mut first_flag = true;
            
            for word in self.flags.split(',') {
                let word = word.trim();
                if !first_flag && current_line_len + word.len() + 2 > wrap_width { // +2 for ", "
                    println!();
                    print!("       {}", word); // 7 spaces to align with "Flags: "
                    current_line_len = 7 + word.len();
                } else {
                    if first_flag {
                        print!("{}", word);
                        current_line_len += word.len();
                        first_flag = false;
                    } else {
                        print!(", {}", word);
                        current_line_len += word.len() + 2; // +2 for ", "
                    }
                }
            }
            println!(); // Final newline
        }
    }

    /// Get the formatted information lines for display.
    ///
    /// This helper function generates the formatted CPU information lines
    /// that are used by both logo and no-logo display methods.
    fn get_info_lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!("Name: {}", self.model),
            format!("Architecture: {}", self.architecture),
            format!("Byte Order: {}", self.byte_order),
            format!("Vendor: {}", self.vendor),
            format!("Cores: {} cores ({} threads)", self.physical_cores, self.logical_cores),
        ];
        
        if let Some(mhz) = self.base_mhz {
            lines.push(format!("Base Frequency: {:.2} MHz", mhz));
        }
        
        // For Apple Silicon, provide more detailed cache information
        if self.vendor == "Apple" {
            // Try to get performance level specific cache info
            if let Ok(perf0_l1i) = Self::get_sysctl_u32("hw.perflevel0.l1icachesize") {
                if let Ok(perf0_l1d) = Self::get_sysctl_u32("hw.perflevel0.l1dcachesize") {
                    let l1i_formatted = Self::format_cache_size(perf0_l1i / 1024);
                    let l1d_formatted = Self::format_cache_size(perf0_l1d / 1024);
                    lines.push(format!("P-Core L1 Cache: {} I + {} D", l1i_formatted, l1d_formatted));
                }
            }
            if let Ok(perf1_l1i) = Self::get_sysctl_u32("hw.perflevel1.l1icachesize") {
                if let Ok(perf1_l1d) = Self::get_sysctl_u32("hw.perflevel1.l1dcachesize") {
                    let l1i_formatted = Self::format_cache_size(perf1_l1i / 1024);
                    let l1d_formatted = Self::format_cache_size(perf1_l1d / 1024);
                    lines.push(format!("E-Core L1 Cache: {} I + {} D", l1i_formatted, l1d_formatted));
                }
            }
            if let Ok(perf0_l2) = Self::get_sysctl_u32("hw.perflevel0.l2cachesize") {
                let l2_formatted = Self::format_cache_size(perf0_l2 / 1024);
                lines.push(format!("P-Core L2 Cache: {}", l2_formatted));
            }
            if let Ok(perf1_l2) = Self::get_sysctl_u32("hw.perflevel1.l2cachesize") {
                let l2_formatted = Self::format_cache_size(perf1_l2 / 1024);
                lines.push(format!("E-Core L2 Cache: {}", l2_formatted));
            }
        } else {
            // For non-Apple systems, use traditional cache display
            if let Some((l1, l1_count)) = self.l1_size {
                let l1_formatted = Self::format_cache_size(l1);
                lines.push(format!("L1 Cache Size: {} ({} cores)", l1_formatted, l1_count));
            }
            
            if let Some((l2, l2_count)) = self.l2_size {
                let l2_formatted = Self::format_cache_size(l2);
                lines.push(format!("L2 Cache Size: {} ({} cores)", l2_formatted, l2_count));
            }
            
            if let Some((l3, l3_count)) = self.l3_size {
                let l3_formatted = Self::format_cache_size(l3);
                lines.push(format!("L3 Cache Size: {} ({} cores)", l3_formatted, l3_count));
            }
        }
        
        // Don't add flags here - they will be handled separately with wrapping
        
        lines
    }
} 