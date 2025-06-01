mod linux; // Declares the linux module (src/linux/mod.rs)
mod art; // Declares the art module (src/art.rs)
mod windows; // Declares the windows module (src/windows/mod.rs)
mod macos; // Declares the macos module (src/macos/mod.rs)
mod cla; // Declares the command line arguments module (src/cla.rs)
use std::env; // Declares the standard library's env module for environment variable access

fn main() {
    let args = match cla::Args::parse() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("{}", e);
            eprintln!("Try 'rcpufetch --help' for more information.");
            std::process::exit(1);
        }
    };

    // Handle help flag
    if args.help {
        cla::print_help();
        return;
    }

    // Handle version flag
    if args.version {
        cla::print_version();
        return;
    }

    // Handle license flag
    if args.license {
        cla::print_license();
        return;
    }

    // Handle completions flag
    if let Some(shell) = args.completions {
        cla::print_completions(&shell);
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
            use crate::linux::linux::LinuxCpuInfo;
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