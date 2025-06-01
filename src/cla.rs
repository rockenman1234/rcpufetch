// Command Line Arguments Module
// This module handles parsing command line arguments for rcpufetch.

//! Manual command-line argument parsing for rcpufetch.
//!
//! This module provides the `Args` struct and associated parsing logic for all supported
//! command-line options. It does not use any external dependencies, ensuring maximum
//! portability and transparency. The module also provides functions to print help,
//! version, license, and shell completions for supported shells.
//!
//! # Example
//!
//! ```rust
//! let args = cla::Args::parse().unwrap();
//! if args.no_logo {
//!     // ...
//! }
//! ```

use std::env;

/// Command line arguments structure
///
/// Holds all supported CLI options for rcpufetch, including flags for help, version,
/// license, completions, logo override, and logo disabling.
#[derive(Debug, Default)]
pub struct Args {
    /// Disable logo display (`-n`/`--no-logo`)
    pub no_logo: bool,
    /// Override logo display with specific vendor (`-l`/`--logo <VENDOR>`)
    pub logo: Option<String>,
    /// Display license information (`--license`)
    pub license: bool,
    /// Display help information (`-h`/`--help`)
    pub help: bool,
    /// Display version information (`-V`/`--version`)
    pub version: bool,
    /// Generate shell completions (`--completions <SHELL>`)
    pub completions: Option<String>,
}

impl Args {
    /// Parse command line arguments manually.
    ///
    /// Iterates over `std::env::args()` and matches each argument, setting the appropriate
    /// fields in the struct. Handles both short and long options, as well as value-taking
    /// options (e.g., `--logo <VENDOR>`). Returns descriptive error messages for unknown
    /// or malformed arguments.
    ///
    /// # Returns
    ///
    /// * `Ok(Args)` if parsing succeeds
    /// * `Err(String)` if an unknown or malformed argument is encountered
    pub fn parse() -> Result<Self, String> {
        let args: Vec<String> = env::args().collect();
        let mut parsed_args = Args::default();
        let mut i = 1; // Skip program name

        while i < args.len() {
            match args[i].as_str() {
                "-h" | "--help" => {
                    parsed_args.help = true;
                }
                "-V" | "--version" => {
                    parsed_args.version = true;
                }
                "--license" => {
                    parsed_args.license = true;
                }
                "-n" | "--no-logo" => {
                    parsed_args.no_logo = true;
                }
                "-l" | "--logo" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("Error: --logo requires a value".to_string());
                    }
                    parsed_args.logo = Some(args[i].clone());
                }
                arg if arg.starts_with("--logo=") => {
                    let value = arg.strip_prefix("--logo=").unwrap();
                    if value.is_empty() {
                        return Err("Error: --logo requires a value".to_string());
                    }
                    parsed_args.logo = Some(value.to_string());
                }
                "--completions" => {
                    i += 1;
                    if i >= args.len() {
                        return Err("Error: --completions requires a shell name (fish, bash, zsh)".to_string());
                    }
                    parsed_args.completions = Some(args[i].clone());
                }
                arg => {
                    return Err(format!("Error: Unknown argument '{}'", arg));
                }
            }
            i += 1;
        }

        Ok(parsed_args)
    }
}

/// Print help information to stdout.
///
/// Prints usage, options, and example invocations for rcpufetch.
pub fn print_help() {
    println!("rcpufetch {}", env!("CARGO_PKG_VERSION"));
    println!("{}", env!("CARGO_PKG_DESCRIPTION"));
    println!();
    println!("USAGE:");
    println!("    rcpufetch [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help                   Print help information");
    println!("    -V, --version                Print version information");
    println!("        --license                Display license information");
    println!("        --completions <SHELL>    Generate shell completions (fish, bash, zsh)");
    println!("    -n, --no-logo                Disable logo display");
    println!("    -l, --logo <VENDOR>          Override logo display with specific vendor");
    println!("                                 Valid vendors: nvidia, powerpc, arm, amd, intel, apple");
    println!();
    println!("EXAMPLES:");
    println!("    rcpufetch                    Display CPU info with auto-detected logo");
    println!("    rcpufetch --no-logo          Display CPU info without logo");
    println!("    rcpufetch --logo intel       Display CPU info with Intel logo");
    println!("    rcpufetch --license          Show license information");
}

/// Print version information to stdout.
///
/// Prints the package name and version.
pub fn print_version() {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}

/// Print license information to stdout.
///
/// Prints copyright and license details for rcpufetch.
pub fn print_license() {
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

/// Generate shell completions for the specified shell.
///
/// # Arguments
///
/// * `shell` - The shell name ("fish", "bash", or "zsh").
///
/// Prints the appropriate shell completion script to stdout. Exits with an error for unsupported shells.
pub fn print_completions(shell: &str) {
    match shell.to_lowercase().as_str() {
        "fish" => print_fish_completions(),
        "bash" => print_bash_completions(),
        "zsh" => print_zsh_completions(),
        _ => {
            eprintln!("Error: Unsupported shell '{}'. Supported shells: fish, bash, zsh", shell);
            std::process::exit(1);
        }
    }
}

/// Generate fish shell completions and print to stdout.
fn print_fish_completions() {
    println!("# Fish completions for rcpufetch");
    println!("complete -c rcpufetch -s h -l help -d 'Print help information'");
    println!("complete -c rcpufetch -s V -l version -d 'Print version information'");
    println!("complete -c rcpufetch -l license -d 'Display license information'");
    println!("complete -c rcpufetch -s n -l no-logo -d 'Disable logo display'");
    println!("complete -c rcpufetch -s l -l logo -x -a 'nvidia powerpc arm amd intel apple' -d 'Override logo display with specific vendor'");
    println!("complete -c rcpufetch -l completions -x -a 'fish bash zsh' -d 'Generate shell completions'");
}

/// Generate bash shell completions and print to stdout.
fn print_bash_completions() {
    println!("# Bash completions for rcpufetch");
    println!("_rcpufetch() {{");
    println!("    local cur prev opts");
    println!("    COMPREPLY=()");
    println!("    cur=\"${{COMP_WORDS[COMP_CWORD]}}\"");
    println!("    prev=\"${{COMP_WORDS[COMP_CWORD-1]}}\"");
    println!("    opts=\"-h --help -V --version --license -n --no-logo -l --logo --completions\"");
    println!();
    println!("    case \"${{prev}}\" in");
    println!("        --logo|-l)");
    println!("            COMPREPLY=($(compgen -W \"nvidia powerpc arm amd intel apple\" -- \"${{cur}}\"))");
    println!("            return 0");
    println!("            ;;");
    println!("        --completions)");
    println!("            COMPREPLY=($(compgen -W \"fish bash zsh\" -- \"${{cur}}\"))");
    println!("            return 0");
    println!("            ;;");
    println!("    esac");
    println!();
    println!("    COMPREPLY=($(compgen -W \"${{opts}}\" -- \"${{cur}}\"))");
    println!("}}");
    println!("complete -F _rcpufetch rcpufetch");
}

/// Generate zsh shell completions and print to stdout.
fn print_zsh_completions() {
    println!("# Zsh completions for rcpufetch");
    println!("#compdef rcpufetch");
    println!();
    println!("_rcpufetch() {{");
    println!("    _arguments \\");
    println!("        '(-h --help){{-h,--help}}[Print help information]' \\");
    println!("        '(-V --version){{-V,--version}}[Print version information]' \\");
    println!("        '--license[Display license information]' \\");
    println!("        '(-n --no-logo){{-n,--no-logo}}[Disable logo display]' \\");
    println!("        '(-l --logo){{-l,--logo}}[Override logo display with specific vendor]:vendor:(nvidia powerpc arm amd intel apple)' \\");
    println!("        '--completions[Generate shell completions]:shell:(fish bash zsh)'");
    println!("}}");
    println!();
    println!("_rcpufetch \"$@\"");
}