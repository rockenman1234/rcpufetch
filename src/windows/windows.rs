use std::process::Command;
use crate::art::logos::get_logo_lines_for_vendor;

pub struct WindowsCpuInfo {
    model: String,
    vendor: String,
    physical_cores: u32,
    logical_cores: u32,
    base_mhz: Option<f32>,
    l1_size: Option<(u32, u32)>,
    l2_size: Option<(u32, u32)>,
    l3_size: Option<(u32, u32)>,
}

impl WindowsCpuInfo {
    pub fn new() -> Result<Self, String> {
        // Get detailed CPU information using PowerShell
        let output = Command::new("powershell")
            .args([
                "-Command",
                r#"Get-WmiObject -Class Win32_Processor | Select-Object Name, NumberOfCores, NumberOfLogicalProcessors, CurrentClockSpeed, L3CacheSize | ConvertTo-Json"#
            ])
            .output()
            .map_err(|e| e.to_string())?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // Parse the JSON output
        let json: serde_json::Value = serde_json::from_str(&output_str)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        let model = json["Name"]
            .as_str()
            .unwrap_or("Unknown CPU")
            .to_string();

        let physical_cores = json["NumberOfCores"]
            .as_u64()
            .unwrap_or(0) as u32;

        let logical_cores = json["NumberOfLogicalProcessors"]
            .as_u64()
            .unwrap_or(0) as u32;

        let base_mhz = json["CurrentClockSpeed"]
            .as_u64()
            .map(|mhz| mhz as f32 / 1000.0); // Convert MHz to GHz

        // For Windows, we'll use "Intel" or "AMD" based on the model name
        let vendor = if model.to_lowercase().contains("intel") {
            "GenuineIntel"
        } else if model.to_lowercase().contains("amd") {
            "AuthenticAMD"
        } else {
            "Unknown"
        }.to_string();

        // Get cache information using a more accurate PowerShell command
        let cache_output = Command::new("powershell")
            .args([
                "-Command",
                r#"$cpu = Get-WmiObject -Class Win32_Processor; $cache = Get-WmiObject -Class Win32_CacheMemory; @{
                    L1 = ($cache | Where-Object {$_.Level -eq 3} | Measure-Object -Property MaxCacheSize -Sum).Sum;
                    L2 = ($cache | Where-Object {$_.Level -eq 1} | Measure-Object -Property MaxCacheSize -Sum).Sum;
                    L3 = $cpu.L3CacheSize
                } | ConvertTo-Json"#
            ])
            .output()
            .ok();

        let mut l1_size = None;
        let mut l2_size = None;
        let mut l3_size = None;

        if let Some(cache_output) = cache_output {
            if let Ok(cache_json) = serde_json::from_str::<serde_json::Value>(&String::from_utf8_lossy(&cache_output.stdout)) {
                if let Some(l1) = cache_json["L1"].as_u64() {
                    l1_size = Some((l1 as u32 / physical_cores, l1 as u32));
                }
                if let Some(l2) = cache_json["L2"].as_u64() {
                    l2_size = Some((l2 as u32 / physical_cores, l2 as u32));
                }
                if let Some(l3) = json["L3CacheSize"].as_u64() {
                    l3_size = Some((l3 as u32, l3 as u32));
                }
            }
        }

        // If we still don't have L1/L2 cache info, try another approach
        if l1_size.is_none() || l2_size.is_none() {
            let alt_cache_output = Command::new("powershell")
                .args([
                    "-Command",
                    r#"$cpu = Get-WmiObject -Class Win32_Processor; @{
                        L1 = $cpu.L1CacheSize;
                        L2 = $cpu.L2CacheSize;
                        L3 = $cpu.L3CacheSize
                    } | ConvertTo-Json"#
                ])
                .output()
                .ok();

            if let Some(alt_cache_output) = alt_cache_output {
                if let Ok(alt_cache_json) = serde_json::from_str::<serde_json::Value>(&String::from_utf8_lossy(&alt_cache_output.stdout)) {
                    if l1_size.is_none() {
                        if let Some(l1) = alt_cache_json["L1"].as_u64() {
                            l1_size = Some((l1 as u32 / physical_cores, l1 as u32));
                        }
                    }
                    if l2_size.is_none() {
                        if let Some(l2) = alt_cache_json["L2"].as_u64() {
                            l2_size = Some((l2 as u32 / physical_cores, l2 as u32));
                        }
                    }
                }
            }
        }

        Ok(WindowsCpuInfo {
            model,
            vendor,
            physical_cores,
            logical_cores,
            base_mhz,
            l1_size,
            l2_size,
            l3_size,
        })
    }

    pub fn display_info(&self) {
        let logo_lines = get_logo_lines_for_vendor(&self.vendor).unwrap_or_else(|| vec![]);
        let info_lines = vec![
            format!("Name: {:<30}", self.model),
            format!("Vendor: {:<30}", self.vendor),
            format!("Base Frequency: {:>7}", match self.base_mhz { Some(ghz) => format!("{:.3} GHz", ghz), None => "Unknown".to_string() }),
            format!("Cores: {:>2} cores ({} threads)", self.physical_cores, self.logical_cores),
            format!("L1 Size: {}", match self.l1_size { Some((per, total)) => format!("{}KB ({}KB Total)", per, total), None => "Unknown".to_string() }),
            format!("L2 Size: {}", match self.l2_size { Some((per, total)) => format!("{}KB ({}KB Total)", per, total), None => "Unknown".to_string() }),
            format!("L3 Size: {}", match self.l3_size { Some((per, total)) => format!("{}KB ({}KB Total)", per, total), None => "Unknown".to_string() }),
        ];

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