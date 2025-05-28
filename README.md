# rcpufetch

**rcpufetch** is a fast, cross-platform command-line tool written in Rust that displays detailed information about your CPU in a visually appealing way, including vendor ASCII art logos.

## Features
- Shows CPU model, vendor, core and thread count, cache sizes, and maximum frequency
- Displays a colorful ASCII art logo for your CPU vendor (AMD, Intel, ARM, NVIDIA, etc.)
- Supports Linux (other OS support can be added—see CONTRIBUTING.md)
- Reads from `/proc/cpuinfo` and `/sys` for accurate hardware details
- Clean, horizontally aligned output for easy reading

## Example Output
```
          '###############                Name: AMD Ryzen 5 9600X 6-Core Processor
             ,#############               Vendor: AuthenticAMD
                      .####               Max Frequency: 5.486 GHz
              #.      .####               Cores:  6 cores (12 threads)
            :##.      .####               L1i Size: 32KB (192KB Total)
           :###.      .####               L1d Size: 48KB (288KB Total)
           #########.   :##               L2 Size: 1024KB (6144KB Total)
           #######.       ;               L3 Size: 32768KB (32768KB Total)
```

## Installation
1. Clone the repository:
   ```
   git clone https://github.com/yourusername/rcpufetch.git
   cd rcpufetch
   ```
2. Build with Cargo:
   ```
   cargo build --release
   ```
3. Run:
   ```
   ./target/release/rcpufetch
   ```

## Contributing
See [CONTRIBUTING.md](CONTRIBUTING.md) for details on how to add features, support new operating systems, and contribute code or ASCII art.

## License
MIT
