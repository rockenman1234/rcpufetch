# rcpufetch

**rcpufetch** is a fast, cross-platform command-line tool written in Rust that displays detailed information about your CPU in a visually appealing way, including vendor ASCII art logos.

> [!WARNING]
> This project is still in early development, please report bugs under the issues tab. Check back in the future for the first release, thanks!

## Features
- Shows CPU model, vendor, core and thread count, cache sizes, and maximum frequency
- Displays a colorful ASCII art logo for your CPU vendor (AMD, Intel, ARM, NVIDIA, etc.)
- Clean codebase - nothing but Rust in here!
- Horizontally aligned output for easy reading, complete with logo support.

## Screenshot
![Main Screenshot](.github/assets/Screenshot_AMD.png)

## 1. Support

| OS          | x86_64 / x86       | ARM                | RISC-V             | PowerPC            |
|:-----------:|:------------------:|:------------------:|:------------------:|:------------------:|
| GNU / Linux | :heavy_check_mark: | :x:                | :x:                | :x:                |
| Windows     | :heavy_check_mark: | :x:                | :x:                | :x:                |
| macOS       | :x:                | :x:                | :x:                | :x:                |
| FreeBSD     | :x:                | :x:                | :x:                | :x:                |


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
GPLv3
