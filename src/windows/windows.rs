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
       // TODO: Implement this later
       return Ok(Self {
            model: "Unknown".to_string(),
            vendor: "Unknown".to_string(),
            physical_cores: 0,
            logical_cores: 0,
            base_mhz: None,
            l1_size: None,
            l2_size: None,
            l3_size: None,
        });
    }

    pub fn display_info(&self) {
        println!("CPU Model: {}", self.model);
        println!("Vendor: {}", self.vendor);
        println!("Physical Cores: {}", self.physical_cores);
        println!("Logical Cores: {}", self.logical_cores);
        if let Some(mhz) = self.base_mhz {
            println!("Base Frequency: {:.2} MHz", mhz);
        }
        if let Some((l1, l1_count)) = self.l1_size {
            println!("L1 Cache Size: {} KB ({} cores)", l1, l1_count);
        }
        if let Some((l2, l2_count)) = self.l2_size {
            println!("L2 Cache Size: {} KB ({} cores)", l2, l2_count);
        }
        if let Some((l3, l3_count)) = self.l3_size {
            println!("L3 Cache Size: {} KB ({} cores)", l3, l3_count);
        }

        // Display logo
        if let Some(logo_lines) = get_logo_lines_for_vendor(&self.vendor) {
            for line in logo_lines {
                println!("{}", line);
            }
        }
    }
} 