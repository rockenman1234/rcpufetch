mod linux; // Declares the linux module (src/linux/mod.rs)
mod art; // Add this to declare the art module
mod windows;

// Use the LinuxCpuInfo struct from the nested linux module
use crate::linux::linux::LinuxCpuInfo;
use clap::Parser;
use std::env;

#[derive(Parser)]
#[command(author, version, about = "A CPU information fetcher", long_about = None)]
struct Args {
    /// Disable logo display
    #[arg(short = 'n', long = "no-logo")]
    no_logo: bool,
    
    /// Override logo display with specific vendor (nvidia, powerpc, arm, amd, intel)
    #[arg(short = 'l', long = "logo", value_name = "VENDOR")]
    logo: Option<String>,
}

fn main() {
    let args = Args::parse();

    // Convert logo argument to vendor ID format if provided
    let logo_override = args.logo.as_ref().and_then(|logo| {
        match logo.to_lowercase().as_str() {
            "nvidia" => Some("NVIDIA"),
            "powerpc" => Some("PowerPC"),
            "arm" => Some("ARM"),
            "amd" => Some("AuthenticAMD"),
            "intel" => Some("GenuineIntel"),
            _ => {
                eprintln!("Warning: Unknown logo vendor '{}'. Valid options: nvidia, powerpc, arm, amd, intel", logo);
                None
            }
        }
    });

    // Detect OS and use appropriate module
    let os = env::consts::OS;
    
    match os {
        "linux" => {
            match LinuxCpuInfo::new() {
                Ok(cpu_info) => {
                    if args.no_logo {
                        cpu_info.display_info_no_logo();
                    } else {
                        cpu_info.display_info_with_logo(logo_override);
                    }
                }
                Err(e) => {
                    eprintln!("Error fetching CPU info: {}", e);
                }
            }
        }
        "windows" => {
            use crate::windows::windows::WindowsCpuInfo;
            match WindowsCpuInfo::new() {
                Ok(cpu_info) => {
                    if args.no_logo {
                        cpu_info.display_info_no_logo();
                    } else {
                        cpu_info.display_info_with_logo(logo_override);
                    }
                }
                Err(e) => {
                    eprintln!("Error fetching CPU info: {}", e);
                }
            }
        }
        _ => {
            eprintln!("Unsupported operating system: {}", os);
        }
    }
}
