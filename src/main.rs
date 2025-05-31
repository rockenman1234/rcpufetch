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
}

fn main() {
    let _args = Args::parse();

    // Detect OS and use appropriate module
    let os = env::consts::OS;
    
    match os {
        "linux" => {
            match LinuxCpuInfo::new() {
                Ok(cpu_info) => {
                    cpu_info.display_info();
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
                    cpu_info.display_info();
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
