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

    /// Display CPU information with logo (side-by-side layout).
    ///
    /// This function displays comprehensive CPU information alongside a vendor logo
    /// in a side-by-side layout. The logo can be overridden to display a different
    /// vendor's logo regardless of the actual CPU vendor.
    pub fn display_info_with_logo(&self, logo_override: Option<&str>) {
        let vendor_to_use = logo_override.unwrap_or(&self.vendor);
        let logo_lines = get_logo_lines_for_vendor(vendor_to_use).unwrap_or_else(|| vec![]);
        
        let info_lines = self.get_info_lines();
        
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
    }

    /// Get the formatted information lines for display.
    ///
    /// This helper function generates the formatted CPU information lines
    /// that are used by both logo and no-logo display methods.
    fn get_info_lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!("Name: {}", self.model),
            format!("Vendor: {}", self.vendor),
            format!("Cores: {} cores ({} threads)", self.physical_cores, self.logical_cores),
        ];
        
        if let Some(mhz) = self.base_mhz {
            lines.push(format!("Base Frequency: {:.2} MHz", mhz));
        }
        
        if let Some((l1, l1_count)) = self.l1_size {
            lines.push(format!("L1 Cache Size: {} KB ({} cores)", l1, l1_count));
        }
        
        if let Some((l2, l2_count)) = self.l2_size {
            lines.push(format!("L2 Cache Size: {} KB ({} cores)", l2, l2_count));
        }
        
        if let Some((l3, l3_count)) = self.l3_size {
            lines.push(format!("L3 Cache Size: {} KB ({} cores)", l3, l3_count));
        }
        
        lines
    }
} 