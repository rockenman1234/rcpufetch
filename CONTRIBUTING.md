# Contributing to rcpufetch

Thank you for your interest in contributing to **rcpufetch**! This document will help you get started, understand the codebase, and make your first contribution.

## Table of Contents
- [Project Overview](#project-overview)
- [Codebase Structure](#codebase-structure)
- [Architecture Overview](#architecture-overview)
- [CLI Implementation Details](#cli-implementation-details)
- [Linux Implementation Deep Dive](#linux-implementation-deep-dive)
- [How to Contribute](#how-to-contribute)
- [Adding New Features for Existing OS](#adding-new-features-for-existing-os)
- [Implementation Notes](#implementation-notes)
- [ASCII Art and Logo System](#ascii-art-and-logo-system)
- [Coding Style](#coding-style)
- [Testing Guidelines](#testing-guidelines)
- [Reporting Issues](#reporting-issues)
- [Pull Request Process](#pull-request-process)

___

## Project Overview

**rcpufetch** is a fast, cross-platform CLI tool that displays detailed CPU information in a visually appealing way, including vendor ASCII art. It is written in Rust and currently supports Linux, Windows, and macOS, with plans for additional operating system support.

The tool provides comprehensive CPU information including:
- CPU model name and vendor
- Architecture and byte order information
- Physical and logical core counts
- Maximum frequency information
- Multi-level cache sizes (L1, L2, L3)
- CPU flags and capabilities
- Colorized vendor logos displayed alongside the information

___

## Codebase Structure

The project follows a modular, OS-specific architecture:

```
rcpufetch/
├── Cargo.toml               # Project dependencies and metadata
├── src/
│   ├── main.rs             # Entry point, CLI parsing, OS dispatch
│   ├── linux/              # Linux-specific implementation
│   │   ├── linux.rs        # Core Linux CPU info parsing and display
│   │   └── mod.rs          # Linux module declaration
│   ├── windows/            # Windows-specific implementation
│   │   ├── windows.rs      # Windows CPU info via PowerShell/WMI
│   │   └── mod.rs          # Windows module declaration
│   ├── macos/              # macOS-specific implementation
│   │   ├── macos.rs        # macOS CPU info via sysctl and system APIs
│   │   └── mod.rs          # macOS module declaration
│   └── art/                # ASCII art and visual components
│       ├── logos.rs        # Vendor ASCII art and color definitions
│       └── mod.rs          # Art module declaration
```

### Key Files Explained

- **`src/main.rs`**: Entry point that handles CLI argument parsing using `clap`, detects the operating system, and dispatches to the appropriate OS-specific module.
- **`src/linux/linux.rs`**: Contains the complete Linux implementation including `/proc/cpuinfo` parsing, sysfs cache information reading, and formatted display output.
- **`src/windows/windows.rs`**: Windows implementation using PowerShell and WMI queries to gather CPU information.
- **`src/macos/macos.rs`**: macOS implementation using `sysctl` command and system APIs to gather CPU information, with special handling for Apple Silicon performance levels.
- **`src/art/logos.rs`**: Contains ASCII art for different CPU vendors (AMD, Intel, ARM, NVIDIA, PowerPC, Apple) with color formatting support.

___

## Architecture Overview

The application follows a consistent pattern across operating systems:

1. **CLI Parsing**: `main.rs` uses `clap` to parse command-line arguments (`--logo`, `--no-logo`)
2. **OS Detection**: `main.rs` uses `std::env::consts::OS` to detect the current operating system
3. **Module Dispatch**: Based on OS detection, the appropriate module is called (Linux, Windows, macOS)
4. **Information Gathering**: Each OS module implements a `new()` method that gathers CPU information using OS-specific APIs
5. **Display Formatting**: Each module implements display methods that format and display the information:
   - `display_info_with_logo(logo_override)` - Shows info with vendor logo (actual or CLI-overridden)
   - `display_info_no_logo()` - Shows info without any logo for clean text output

### Common Struct Pattern

Each OS implementation follows this pattern:

```rust
pub struct OSCpuInfo {
    model: String,
    vendor: String,
    // ... OS-specific fields
}

impl OSCpuInfo {
    pub fn new() -> Result<Self, String> {
        // OS-specific information gathering
    }
    
    pub fn display_info_with_logo(&self, logo_override: Option<&str>) {
        // Display with vendor logo (actual or overridden)
    }
    
    pub fn display_info_no_logo(&self) {
        // Display without any logo
    }
    
    fn get_info_lines(&self) -> Vec<String> {
        // Helper method to generate formatted info lines
    }
}
```

The display methods integrate with the CLI system:
- `display_info_with_logo()` is called when no `--no-logo` flag is present
- `display_info_no_logo()` is called when `--no-logo` flag is present
- The `logo_override` parameter contains the vendor ID when `--logo` flag is used

___

## CLI Implementation Details

The CLI argument parsing is handled in `src/main.rs` using the following structure:

```rust
#[derive(Parser)]
#[command(author, version, about = "A CPU information fetcher", long_about = None)]
struct Args {
    /// Disable logo display
    #[arg(short = 'n', long = "no-logo")]
    no_logo: bool,
    
    /// Override logo display with specific vendor
    #[arg(short = 'l', long = "logo", value_name = "VENDOR")]
    logo: Option<String>,
}
```

#### Logo Processing Logic

1. **Logo Override Processing**: The logo argument is converted to the internal vendor ID format:
   ```rust
   let logo_override = args.logo.as_ref().map(|logo| {
       match logo.to_lowercase().as_str() {
           "nvidia" => "NVIDIA",
           "powerpc" => "PowerPC", 
           "arm" => "ARM",
           "amd" => "AuthenticAMD",
           "intel" => "GenuineIntel",
           _ => {
               // Warning for unknown vendors
               return None;
           }
       }
   }).flatten();
   ```

2. **Display Method Selection**: Based on the CLI arguments, the appropriate display method is called:
   ```rust
   if args.no_logo {
       cpu_info.display_info_no_logo();
   } else {
       cpu_info.display_info_with_logo(logo_override);
   }
   ```

#### Error Handling

- **Unknown logo vendors**: Display a warning message but continue execution with the actual vendor logo
- **Invalid arguments**: Handled automatically by `clap` with helpful error messages
- **Conflicting options**: `--no-logo` takes precedence over `--logo` if both are specified

### Adding New CLI Options

When adding new CLI options, follow this pattern:

1. **Add to Args struct** in `src/main.rs`:
   ```rust
   /// New option description
   #[arg(long = "new-option")]
   new_option: bool,
   ```

2. **Update display method calls** to pass the new option:
   ```rust
   cpu_info.display_info_with_options(logo_override, args.new_option);
   ```

3. **Implement option handling** in each OS module's display methods.

4. **Update documentation** in both README.md and CONTRIBUTING.md.

___

## Linux Implementation Deep Dive

The Linux implementation in `src/linux/linux.rs` is the most comprehensive and serves as the reference implementation. Understanding this code is crucial for contributors.

### Data Sources

The Linux implementation gathers information from multiple sources:

1. **`/proc/cpuinfo`**: Primary source for CPU model, vendor, flags, core counts, and basic frequency information
2. **`/sys/devices/system/cpu/cpu*/cache/`**: Detailed cache information (L1, L2, L3 sizes and characteristics)
3. **`/sys/devices/system/cpu/cpu*/cpufreq/`**: CPU frequency scaling information
4. **`uname -m`**: System architecture information

### LinuxCpuInfo Structure

```rust
pub struct LinuxCpuInfo {
    model: String,          // CPU model name from /proc/cpuinfo
    vendor: String,         // Vendor ID (AuthenticAMD, GenuineIntel, etc.)
    architecture: String,   // Architecture from uname (x86_64, aarch64, etc.)
    byte_order: String,     // Little/Big Endian from compile-time detection
    flags: String,          // CPU capabilities/flags from /proc/cpuinfo
    physical_cores: u32,    // Physical core count (calculated from unique core IDs)
    logical_cores: u32,     // Logical core count (thread count)
    max_mhz: Option<f32>,   // Maximum frequency in GHz
    l1d_size: Option<(u32, u32)>,  // L1 data cache (per-core, total) in KB
    l1i_size: Option<(u32, u32)>,  // L1 instruction cache (per-core, total) in KB
    l2_size: Option<(u32, u32)>,   // L2 cache (per-core, total) in KB
    l3_size: Option<(u32, u32)>,   // L3 cache (per-core, total) in KB
}
```

### Core Count Calculation

The Linux implementation uses a sophisticated approach to calculate physical vs logical cores:

1. **Parse processor entries**: Each logical core appears as a separate "processor" entry in `/proc/cpuinfo`
2. **Track physical IDs**: The "physical id" field identifies separate CPU sockets
3. **Track core IDs**: The "core id" field identifies separate cores within a socket
4. **Calculate physical cores**: Count unique `(physical_id, core_id)` pairs
5. **Calculate logical cores**: Count total processor entries

This approach correctly handles:
- Single-socket systems with multiple cores
- Multi-socket systems
- Hyperthreading/SMT configurations
- Systems without explicit physical/core ID information

### Cache Information Parsing

The implementation uses a two-tier approach for cache information:

1. **Primary source - sysfs**: Reads from `/sys/devices/system/cpu/cpu0/cache/index*/` for detailed cache information
2. **Fallback - /proc/cpuinfo**: Uses "cache size" field when sysfs is unavailable

#### Sysfs Cache Parsing Process

```rust
// For each cache index in /sys/devices/system/cpu/cpu0/cache/
// Read level (1, 2, 3), type (Data, Instruction, Unified), and size
// Calculate totals based on sharing characteristics:
// - L1/L2: typically per-core, multiply by physical core count
// - L3: typically shared across all cores, use raw value
```

### Frequency Information

Multiple sources are checked for frequency information:
1. **cpufreq scaling_max_freq**: Preferred source from `/sys/devices/system/cpu/cpu*/cpufreq/scaling_max_freq`
2. **cpuinfo CPU MHz**: Fallback from `/proc/cpuinfo` "cpu MHz" field
3. **Conversion**: All frequencies converted to GHz for consistent display

### Display Formatting

The `display_info()` method implements sophisticated formatting:

1. **Logo integration**: Retrieves vendor-specific ASCII art
2. **Side-by-side layout**: Displays logo and information in aligned columns
3. **Flag wrapping**: Automatically wraps long CPU flag lists with proper indentation
4. **Cache size formatting**: Converts KB to MB when appropriate (>1000KB)
5. **Alignment**: Ensures consistent spacing and readability

### Error Handling

The Linux implementation uses comprehensive error handling:
- File I/O errors are wrapped with descriptive messages
- Missing information is handled gracefully with `None` values
- Parsing errors fall back to default values where appropriate
- All public methods return `Result<T, String>` for proper error propagation

___

## How to Contribute

### Prerequisites

Before contributing, ensure you have:
- **Rust toolchain**: Install via [rustup.rs](https://rustup.rs/)
- **Git**: For version control
- **Target OS**: Access to the OS you're developing for (Linux, Windows, macOS, etc.)

### Development Workflow

1. **Fork the repository** and clone your fork locally:
   ```bash
   git clone https://github.com/yourusername/rcpufetch.git
   cd rcpufetch
   ```

2. **Create a new branch** for your feature or bugfix:
   ```bash
   git checkout -b feature/my-new-feature
   ```

3. **Build and test** the project:
   ```bash
   cargo build
   cargo run
   ```

4. **Make your changes** following the guidelines in this document

5. **Test thoroughly**:
   ```bash
   # Test on your development system
   cargo run
   
   # Test different scenarios if applicable
   cargo run -- --no-logo
   
   # Check for compilation warnings
   cargo clippy
   
   # Format your code
   cargo fmt
   ```

6. **Commit and push** your changes:
   ```bash
   git add .
   git commit -m "feat: add support for CPU temperature readings"
   git push origin feature/my-new-feature
   ```

7. **Open a Pull Request** on GitHub with:
   - Clear description of changes
   - Reference to any related issues
   - Screenshots/examples if relevant
   - Testing information

### Development Tips

- **Use cargo watch**: Install with `cargo install cargo-watch`, then run `cargo watch -x run` for automatic rebuilds
- **Test on multiple systems**: If possible, test your changes on different CPU architectures and OS versions
- **Check edge cases**: Consider systems with unusual configurations (very old CPUs, virtual machines, containers)
- **Performance considerations**: The tool should remain fast and lightweight

___


## Adding New Features for Existing OS

### Linux Development Guidelines

Since Linux is the reference implementation, most new features should be implemented here first.

#### Adding New CPU Information Fields

To add a new field (e.g., CPU temperature, power consumption, microcode version):

1. **Add the field to `LinuxCpuInfo`**:
   ```rust
   pub struct LinuxCpuInfo {
       // ...existing fields...
       temperature: Option<f32>,  // New field
   }
   ```

2. **Implement parsing logic**:
   ```rust
   impl LinuxCpuInfo {
       fn get_temperature() -> Option<f32> {
           // Read from /sys/class/thermal/ or /sys/class/hwmon/
           // Parse temperature data
           // Return temperature in Celsius
       }
   }
   ```

3. **Update the `new()` method**:
   ```rust
   pub fn new() -> Result<Self, String> {
       // ...existing parsing...
       let temperature = Self::get_temperature();
       
       Ok(LinuxCpuInfo {
           // ...existing fields...
           temperature,
       })
   }
   ```

4. **Update the display methods**:
   ```rust
   // Update both display methods to include the new field
   fn get_info_lines(&self) -> Vec<String> {
       vec![
           // ...existing lines...
           format!("Temperature: {}", match self.temperature {
               Some(temp) => format!("{:.1}°C", temp),
               None => "Unknown".to_string()
           }),
       ]
   }
   ```

   Note: Since the CLI refactoring, each OS module now implements:
   - `display_info_with_logo(logo_override: Option<&str>)` - For logo display with optional override
   - `display_info_no_logo()` - For text-only output
   - `get_info_lines()` - Helper method that generates formatted info lines

#### Working with Linux System Files

Common data sources and their purposes:

- **`/proc/cpuinfo`**: Basic CPU information, vendor, model, flags
- **`/sys/devices/system/cpu/`**: CPU-specific information, frequency scaling, cache details
- **`/sys/class/thermal/`**: Temperature sensors
- **`/sys/class/hwmon/`**: Hardware monitoring (temperatures, voltages, fan speeds)
- **`/proc/meminfo`**: Memory information (if adding memory features)
- **`/proc/loadavg`**: System load information

#### Error Handling Best Practices

```rust
// Good: Descriptive error messages
let content = fs::read_to_string("/proc/cpuinfo")
    .map_err(|e| format!("Failed to read /proc/cpuinfo: {}", e))?;

// Good: Graceful fallbacks
let temperature = Self::get_temperature().unwrap_or_else(|| {
    eprintln!("Warning: Could not read CPU temperature");
    None
});

// Good: Use Result<T, String> for functions that can fail
fn parse_cache_size(size_str: &str) -> Result<u32, String> {
    // parsing logic
}
```

___ 

## Implementation Notes

### Windows Development Guidelines
TBD

#### Windows-Specific Considerations
TBD

#### Adding Windows Features
TBD

### macOS Development Guidelines

The macOS implementation in `src/macos/macos.rs` provides comprehensive CPU information gathering using `sysctl` APIs and handles both Intel and Apple Silicon architectures with special consideration for performance cores and efficiency cores.

#### Data Sources

The macOS implementation gathers information from multiple `sysctl` keys:

1. **`machdep.cpu.brand_string`**: Primary source for CPU model name
2. **`machdep.cpu.core_count`/`machdep.cpu.cores_per_package`**: Physical core count
3. **`machdep.cpu.thread_count`/`machdep.cpu.logical_per_package`**: Logical core count (with hyperthreading)
4. **`hw.cachesize`**: Traditional cache size information (Intel Macs)
5. **`hw.cacheconfig`**: Cache configuration and count information
6. **`hw.perflevel*.l*cachesize`**: Performance-level specific cache information (Apple Silicon)
7. **`machdep.cpu.max_basic`**: Base frequency information (when available)
8. **`hw.optional.arm.*`**: CPU feature flags and capabilities (Apple Silicon)

#### MacOSCpuInfo Structure

```rust
pub struct MacOSCpuInfo {
    model: String,           // CPU model name from sysctl
    vendor: String,          // Vendor (Intel, AMD, Apple, or Unknown)
    physical_cores: u32,     // Physical core count
    logical_cores: u32,      // Logical core count (including hyperthreading)
    base_mhz: Option<f32>,   // Base frequency in MHz (if available)
    flags: String,           // CPU feature flags and capabilities
    l1_size: Option<(u32, u32)>,  // L1 cache (size in KB, count)
    l2_size: Option<(u32, u32)>,  // L2 cache (size in KB, count)
    l3_size: Option<(u32, u32)>,  // L3 cache (size in KB, count)
}
```

#### Vendor Detection

The macOS implementation uses brand string analysis to determine the vendor:

1. **Intel detection**: Searches for "intel" in the brand string
2. **AMD detection**: Searches for "amd" in the brand string  
3. **Apple detection**: Searches for "apple" in the brand string (Apple Silicon)
4. **Fallback**: Uses "Unknown" for unrecognized vendors

#### CPU Flags Detection

The macOS implementation detects CPU feature flags and capabilities, particularly for Apple Silicon processors:

**Apple Silicon Flags (ARM64):**
The implementation queries `hw.optional.arm.*` sysctl keys to detect ARM-specific CPU features:

1. **Query all ARM optional features**: Execute `sysctl hw.optional.arm.` command
2. **Parse boolean values**: Each key returns 0 (disabled) or 1 (enabled)
3. **Filter enabled features**: Only include features with value 1
4. **Strip prefix**: Remove "hw.optional.arm." to get just the feature name
5. **Format as CSV**: Return comma-separated list of enabled features

**Example ARM features detected:**
- `FEAT_AES`: AES encryption instructions
- `FEAT_SHA256`/`FEAT_SHA512`: SHA hashing instructions  
- `FEAT_CRC32`: CRC32 checksum instructions
- `FEAT_LSE`: Large System Extensions
- `FEAT_FP16`: Half-precision floating point
- `FEAT_DotProd`: Dot product instructions
- `AdvSIMD`: Advanced SIMD instructions

**Implementation details:**
```rust
fn get_cpu_flags() -> String {
    // Execute: sysctl hw.optional.arm.
    // Parse each line: hw.optional.arm.FEATURE_NAME: 1
    // Extract features with value 1
    // Return: "FEAT_AES,FEAT_SHA256,AdvSIMD,..."
}
```

The flags are displayed in the same format as Linux CPU flags for consistency across platforms.

#### Apple Silicon Specific Handling

Apple Silicon chips (M1, M2, M3, etc.) have a unique architecture with performance cores (P-cores) and efficiency cores (E-cores), each with different cache configurations:

**Performance Level Cache Information:**
- **`hw.perflevel0.*`**: Performance core (P-core) cache information
- **`hw.perflevel1.*`**: Efficiency core (E-core) cache information

**Special Cache Display Logic:**
```rust
// For Apple Silicon, display detailed performance-level cache info
if self.vendor == "Apple" {
    // Show P-Core L1 Cache: [size] I + [size] D
    // Show E-Core L1 Cache: [size] I + [size] D  
    // Show P-Core L2 Cache: [size]
    // Show E-Core L2 Cache: [size]
}
```

#### Core Count Calculation

The macOS implementation handles core counting with fallbacks:

1. **Primary method**: Use `machdep.cpu.core_count` and `machdep.cpu.thread_count`
2. **Fallback method**: Use `machdep.cpu.cores_per_package` and `machdep.cpu.logical_per_package`
3. **Logical core fallback**: If logical count unavailable, use physical count

#### Cache Information Parsing

The implementation uses a two-tier approach optimized for different Mac architectures:

**Traditional Intel Mac Approach:**
1. **Parse `hw.cachesize`**: Space-separated cache sizes in bytes
2. **Parse `hw.cacheconfig`**: Corresponding cache counts/configuration
3. **Convert to KB**: All cache sizes standardized to kilobytes
4. **Handle L3 fallback**: Use performance level caches if traditional L3 unavailable

**Apple Silicon Approach:**
1. **Query performance levels**: Check `hw.perflevel0` and `hw.perflevel1` sysctl keys
2. **Separate I/D caches**: Handle instruction and data caches separately for L1
3. **Performance differentiation**: Display P-core and E-core caches separately
4. **Fallback integration**: Use larger performance level L2 as equivalent "shared" cache

#### Sysctl Helper Functions

The implementation provides utility functions for sysctl interaction:

```rust
fn get_sysctl_string(key: &str) -> Result<String, String>
fn get_sysctl_u32(key: &str) -> Result<u32, String>
```

These functions:
- Execute `sysctl -n [key]` commands
- Handle command execution errors
- Parse string/numeric values with proper error handling
- Return descriptive error messages for debugging

#### Error Handling Best Practices

```rust
// Good: Graceful fallbacks with descriptive errors
let physical_cores = Self::get_sysctl_u32("machdep.cpu.core_count")
    .unwrap_or_else(|_| Self::get_sysctl_u32("machdep.cpu.cores_per_package").unwrap_or(0));

// Good: Optional frequency information
let base_mhz = Self::get_sysctl_string("machdep.cpu.max_basic")
    .ok()
    .and_then(|s| s.parse::<f32>().ok());

// Good: Error context in sysctl calls
fn get_sysctl_string(key: &str) -> Result<String, String> {
    // Provides context about which sysctl key failed
}
```

#### Adding macOS Features

To add new CPU information for macOS:

1. **Research sysctl keys**: Use `sysctl -a | grep cpu` or `sysctl -a | grep hw` to find relevant keys
2. **Test across architectures**: Verify keys work on both Intel and Apple Silicon Macs
3. **Handle architecture differences**: Apple Silicon may have different or additional keys
4. **Add parsing logic**: Use the existing helper functions for consistency
5. **Update display methods**: Add the new information to `get_info_lines()`

#### macOS-Specific Considerations

- **sysctl dependency**: All information gathering relies on the `sysctl` command being available
- **Architecture variations**: Intel Macs vs Apple Silicon may provide different information
- **Performance levels**: Apple Silicon's dual-core-type architecture requires special handling
- **Limited frequency info**: macOS doesn't always expose detailed frequency information
- **Privilege requirements**: Some sysctl keys may require elevated privileges
- **CPU flags support**: ARM feature flags are available on Apple Silicon via `hw.optional.arm.*` keys

___

## Adding Support for New Operating Systems

To add support for a new OS (e.x. Solaris):

1. **Create OS directory structure**:
   ```
   src/
   ├── solaris/         # New OS directory
   │   ├── solaris.rs   # Implementation file
   │   └── mod.rs       # Module declaration
   ```

2. **Implement the standard interface**:
   ```rust
   pub struct SolarisCpuInfo {
       model: String,
       vendor: String,
       // OS-specific fields
   }
   
   impl SolarisCpuInfo {
       pub fn new() -> Result<Self, String> {
           // OS-specific information gathering
       }
       
       pub fn display_info_with_logo(&self, logo_override: Option<&str>) {
           // Display logic using get_logo_lines_for_vendor
       }
       
       pub fn display_info_no_logo(&self) {
           // Text-only display
       }
   }
   ```

3. **Update `main.rs`**:
   ```rust
   mod solaris;  // Add module declaration
   
   match os {
       "linux" => { /* existing code */ },
       "windows" => { /* existing code */ },
       "macos" => { /* existing code */ },
       "solaris" => {
           use crate::solaris::solaris::SolarisCpuInfo;
           // Implementation
       },
       _ => { /* unsupported OS */ }
   }
   ```

4. **Research OS-specific APIs**:
   - **Ex. Solaris**: `psrinfo`, `kstat`, `prtconf`, `/proc/cpuinfo` (if available)

#### Solaris Implementation Guidelines

For Solaris specifically, key data sources include:

**Command-based Information:**
- **`psrinfo -v`**: Detailed processor information including model, frequency
- **`kstat cpu_info`**: Kernel statistics for CPU information
- **`prtconf -v`**: System configuration including CPU details
- **`isainfo -v`**: Instruction set architecture information

**File-based Information:**
- **`/proc/cpuinfo`**: May be available on some Solaris systems
- **`/dev/cpu/`**: CPU device information (if accessible)

**Sample Solaris Implementation Structure:**
```rust
impl SolarisCpuInfo {
    fn get_psrinfo_data() -> Result<String, String> {
        // Execute 'psrinfo -v' command
        // Parse processor information
    }
    
    fn get_kstat_data() -> Result<String, String> {
        // Execute 'kstat cpu_info' command
        // Parse kernel statistics
    }
    
    fn parse_cpu_model(psrinfo_output: &str) -> String {
        // Extract CPU model from psrinfo output
    }
    
    fn parse_cpu_vendor(psrinfo_output: &str) -> String {
        // Determine vendor from processor information
    }
}
```

___

## ASCII Art and Logo System

The logo system in `src/art/logos.rs` provides vendor-specific ASCII art with color support and integrates with the CLI `--logo` flag for logo override functionality.

### Current Vendor Support

- **AMD**: Red and white color scheme (`AuthenticAMD`)
- **Intel**: Cyan color scheme (`GenuineIntel`)
- **ARM**: Cyan color scheme (`ARM`)
- **NVIDIA**: Green and white color scheme (`NVIDIA`)
- **PowerPC**: Yellow color scheme (`PowerPC`)
- **Apple**: Rainbow color scheme (`Apple`)

### CLI Integration

The logo system integrates with the command-line interface in the following ways:

1. **Logo Override**: The `--logo` flag maps user-friendly names to internal vendor IDs:
   - `nvidia` → `NVIDIA`
   - `amd` → `AuthenticAMD`
   - `intel` → `GenuineIntel`
   - `arm` → `ARM`
   - `powerpc` → `PowerPC`
   - `apple` → `Apple`

2. **Logo Display**: The `get_logo_lines_for_vendor()` function is called with either:
   - The actual CPU vendor ID (default behavior)
   - The overridden vendor ID (when `--logo` is used)
   - No logo (when `--no-logo` is used)

### Adding New Vendor Logos

1. **Create ASCII art**: Design ASCII art that fits within reasonable terminal width (< 50 characters)

2. **Add color placeholders**:
   ```rust
   const ASCII_NEW_VENDOR: &str = "\
   $C1  ####    ##  ##   ####     \n\
   $C1 ##  ##  ##    ##  ##  ##   \n\
   $C2 ##  ##  ########  ##  ##   \n\
   $C1  ####    ##  ##   ####     \n";
   ```

3. **Update the vendor matching**:
   ```rust
   fn logo_lines_for_vendor(vendor_id: &str) -> Option<Vec<String>> {
       let (raw_logo, colors): (&str, &[&str]) = match vendor_id {
           "AuthenticAMD" => (ASCII_AMD, &[C_FG_WHITE, C_FG_RED]),
           "GenuineIntel" => (ASCII_INTEL_NEW, &[C_FG_CYAN]),
           "ARM" => (ASCII_ARM, &[C_FG_CYAN]),
           "NVIDIA" => (ASCII_NVIDIA, &[C_FG_GREEN, C_FG_WHITE]),
           "PowerPC" => (ASCII_POWERPC, &[C_FG_YELLOW]),
           "NewVendorID" => (ASCII_NEW_VENDOR, &[C_FG_BLUE, C_FG_WHITE]),
           _ => return None,
       };
       // ...rest of function
   }
   ```

4. **Update CLI mapping** in `src/main.rs` to include the user-friendly name:
   ```rust
   match logo.to_lowercase().as_str() {
       // ...existing mappings...
       "newvendor" => "NewVendorID",
       _ => {
           eprintln!("Warning: Unknown logo vendor...");
           return None;
       }
   }
   ```

### Color System

Available colors are defined as ANSI escape sequences:
- `C_FG_*`: Standard colors (black, red, green, yellow, blue, magenta, cyan, white)
- `C_FG_B_*`: Bright variants
- `COLOR_RESET`: Reset formatting

Use `$C1`, `$C2`, etc. as placeholders in ASCII art, then specify the color array in the match statement.

___

## Coding Style

### Rust Conventions

- **Follow rustfmt**: Use `cargo fmt` to format code automatically
- **Use clippy**: Run `cargo clippy` and fix warnings
- **Documentation**: Use `///` doc comments for all public items
- **Error handling**: Use `Result<T, String>` for fallible operations
- **Naming**: Use snake_case for variables/functions, PascalCase for types

### Documentation Standards

```rust
/// Brief description of the function.
///
/// Longer description explaining the purpose, behavior, and any important
/// details about the function's operation.
///
/// # Arguments
///
/// * `param1` - Description of the first parameter
/// * `param2` - Description of the second parameter
///
/// # Returns
///
/// Description of what the function returns, including error conditions.
///
/// # Errors
///
/// This function will return an error if:
/// - Specific error condition 1
/// - Specific error condition 2
///
/// # Examples
///
/// ```
/// let result = my_function("input");
/// assert_eq!(result, expected_output);
/// ```
pub fn my_function(param1: &str, param2: u32) -> Result<String, String> {
    // Implementation
}
```

### Code Organization

- **Small functions**: Keep functions focused and under 50 lines when possible
- **Logical grouping**: Group related functionality together
- **Constants**: Use `const` for values that don't change
- **Error messages**: Make error messages descriptive and actionable

### Performance Guidelines

- **Avoid unnecessary allocations**: Use `&str` instead of `String` when possible
- **Lazy evaluation**: Only compute expensive operations when needed
- **Caching**: Cache expensive system calls when appropriate
- **Early returns**: Use early returns to avoid deep nesting

___

## Testing Guidelines

### Manual Testing

Since the project deals with system information, manual testing is crucial:

1. **Test on target systems**: Run on actual hardware for the OS you're targeting
2. **Test edge cases**: Try on systems with unusual configurations
3. **Performance testing**: Ensure the tool remains fast even on slower systems
4. **Output validation**: Verify that displayed information is accurate
5. **Error resilience**: Test behavior when sysctl commands fail or return unexpected output

### Automated Testing Considerations

While comprehensive unit testing is challenging for system information tools, consider:

- **Parser testing**: Test parsing functions with known input data
- **Error handling**: Test error conditions with invalid input
- **Format testing**: Test output formatting functions
- **Mock data**: Create sample `/proc/cpuinfo` files for testing

### Testing Checklist

Before submitting a PR, verify:
- [ ] Compiles without warnings (`cargo build`)
- [ ] Passes clippy checks (`cargo clippy`)
- [ ] Formatted correctly (`cargo fmt`)
- [ ] Works on intended operating system
- [ ] Handles error conditions gracefully
- [ ] Provides reasonable fallbacks for missing information
- [ ] Output is readable and properly aligned
- [ ] Performance is acceptable (< 1 second startup time)

___

## Reporting Issues

When reporting bugs or requesting features, please include:

### For Bug Reports

- **OS and version**: e.g., "Ubuntu 22.04", "Windows 11", "macOS 14.0"
- **CPU information**: Model, vendor, core count
- **Error message**: Full error output if applicable
- **Expected behavior**: What should have happened
- **Steps to reproduce**: How to trigger the issue
- **Output of relevant system files**: `/proc/cpuinfo` content for Linux issues, `sysctl` output for macOS issues

### For Feature Requests

- **Use case**: Why is this feature needed?
- **OS scope**: Which operating systems should support this?
- **Data source**: Where would this information come from?
- **Priority**: How important is this feature?

### Issue Templates

Use these formats for consistency:

```markdown
**Bug Report**
- OS: Ubuntu 22.04 / Windows 11 / macOS 14.0
- CPU: AMD Ryzen 5 9600X / Intel i7-13700K / Apple M3 Pro
- rcpufetch version: 0.1.0
- Error: [paste error message]
- Expected: [describe expected behavior]

**Feature Request**
- Feature: CPU temperature display
- Rationale: Users want to monitor thermal performance
- OS support: Linux (primary), macOS, Windows (future)
- Data source: /sys/class/thermal/ on Linux, sysctl on macOS
```

___

## Pull Request Process

### PR Guidelines

1. **Single responsibility**: One feature or fix per PR
2. **Descriptive title**: Use conventional commit format
3. **Detailed description**: Explain what changes and why
4. **Testing evidence**: Show that your changes work
5. **Documentation updates**: Update docs if adding features

### PR Template

```markdown
## Summary
Brief description of the changes made.

## Changes
- Added CPU temperature monitoring for Linux
- Updated display format to include temperature
- Added error handling for missing thermal sensors

## Testing
- Tested on AMD Ryzen system with thermal sensors
- Tested on older Intel system without sensors
- Verified graceful fallback behavior

## Screenshots (if applicable)
[Include before/after screenshots]

## Related Issues
Closes #123
```

### Review Process

1. **Automated checks**: Ensure CI passes (formatting, compilation)
2. **Code review**: Maintainer will review code quality and design
3. **Testing verification**: Changes will be tested on multiple systems
4. **Documentation review**: Ensure docs are updated appropriately
5. **Merge**: Once approved, changes will be merged

### Commit Message Format

Use conventional commit format:
```
type(scope): brief description

Longer description if needed.

- Detail 1
- Detail 2
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation updates
- `style`: Code formatting
- `refactor`: Code restructuring
- `test`: Adding tests
- `chore`: Maintenance tasks

Examples:
```
feat(linux): add CPU temperature monitoring
feat(macos): add Apple Silicon performance level support
fix(windows): handle missing cache information gracefully
docs(contributing): update macOS implementation details
```

---

Thank you for helping make **rcpufetch** better! Your contributions help create a more comprehensive and reliable tool for system information gathering.
