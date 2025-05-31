mod linux; // Declares the linux module (src/linux/mod.rs)
mod art; // Declares the art module (src/art.rs)
mod windows; // Declares the windows module (src/windows/mod.rs)
mod macos; // Declares the macos module (src/macos/mod.rs)

// Use the LinuxCpuInfo struct from the nested linux module
use crate::linux::linux::LinuxCpuInfo;
use clap::Parser;
use std::env;

/// Print license information
fn print_license() {
    println!("Copyright (C) 2025 - Present: Kenneth A. Jenkins, Alan D. Aguilar, & contributors.");
    println!("Licensed under the GNU GPLv3: GNU General Public License version 3.");
    println!("rcpufetch comes with ABSOLUTELY NO WARRANTY.");
    println!();
    println!("A copy of the GNU General Public License Version 3 should");
    println!("have been provided with rcpufetch. If not, you can");
    println!("find it at: <https://www.gnu.org/licenses/gpl-3.0.html>.");
    println!();
    println!("This is free software, and you are welcome to redistribute it");
    println!("under certain conditions, as described above. Type `rcpufetch --help` for assistance.");
}

#[derive(Parser)]
#[command(author, version, about = "A CPU information fetcher", long_about = None)]
struct Args {
    /// Disable logo display
    #[arg(short = 'n', long = "no-logo")]
    no_logo: bool,
    
    /// Override logo display with specific vendor (nvidia, powerpc, arm, amd, intel, apple)
    #[arg(short = 'l', long = "logo", value_name = "VENDOR")]
    logo: Option<String>,
    
    /// Display license information
    #[arg(long = "license")]
    license: bool,
}

fn main() {
    let args = Args::parse();

    // Handle license flag
    if args.license {
        print_license();
        return;
    }

    // Convert logo argument to vendor ID format if provided
    let logo_override = args.logo.as_ref().and_then(|logo| {
        match logo.to_lowercase().as_str() {
            "nvidia" => Some("NVIDIA"),
            "powerpc" => Some("PowerPC"),
            "arm" => Some("ARM"),
            "amd" => Some("AuthenticAMD"),
            "intel" => Some("GenuineIntel"),
            "apple" => Some("Apple"),
            _ => {
                eprintln!("Warning: Unknown logo vendor '{}'. Valid options: nvidia, powerpc, arm, amd, intel, apple", logo);
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
        "macos" => {
            use crate::macos::macos::MacOSCpuInfo;
            match MacOSCpuInfo::new() {
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
