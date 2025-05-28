# Contributing to rcpufetch

Thank you for your interest in contributing to **rcpufetch**! This document will help you get started, understand the codebase, and make your first contribution.

## Table of Contents
- [Project Overview](#project-overview)
- [Codebase Structure](#codebase-structure)
- [How to Contribute](#how-to-contribute)
- [Adding Features for Each OS](#adding-features-for-each-os)
- [Coding Style](#coding-style)
- [Reporting Issues](#reporting-issues)
- [Pull Request Process](#pull-request-process)

## Project Overview

**rcpufetch** is a fast, cross-platform CLI tool that displays detailed CPU information in a visually appealing way, including vendor ASCII art. It is written in Rust and aims to support Linux, and potentially other operating systems in the future.

## Codebase Structure

The project is organized as follows:

```
rcpufetch/
├── Cargo.toml
├── src/
│   ├── main.rs           # Entry point, handles CLI and OS dispatch
│   ├── linux/            # Linux-specific code
│   │   ├── linux.rs      # Linux CPU info parsing and display logic
│   │   └── mod.rs        # Linux module declaration
│   ├── art/              # ASCII art and logo rendering
│   │   ├── logos.rs      # Vendor ASCII art and color codes
│   │   └── mod.rs        # Art module declaration
```

- **src/main.rs**: Entry point. Detects OS and calls the appropriate module (currently Linux only).
- **src/linux/linux.rs**: All Linux-specific logic for parsing `/proc/cpuinfo`, sysfs, and formatting output.
- **src/art/logos.rs**: Contains ASCII art for CPU vendors and color constants.

## How to Contribute

1. **Fork the repository** and clone your fork locally.
2. **Create a new branch** for your feature or bugfix:
   ```
   git checkout -b my-feature
   ```
3. **Make your changes**. See [Adding Features for Each OS](#adding-features-for-each-os) below for tips.
4. **Test your changes** locally:
   ```
   cargo run
   ```
5. **Commit and push** your branch:
   ```
   git add .
   git commit -m "Describe your change"
   git push origin my-feature
   ```
6. **Open a Pull Request** on GitHub. Describe your changes and reference any related issues.

## Adding Features for Each OS

### Linux
- All Linux-specific code lives in `src/linux/linux.rs`.
- To add a new field (e.g., CPU temperature):
  1. Parse the relevant file in `/proc` or `/sys`.
  2. Add a new field to the `LinuxCpuInfo` struct.
  3. Update the `new()` method to extract and store the value.
  4. Update `display_info()` to print the new field.
- For new logo art, add it to `src/art/logos.rs` and update the vendor match logic.

### Other OSes
- To add support for another OS (e.g., Solaris, Haiku, etc):
  1. Create a new folder under `src/` (e.g., `src/haiku/`).
  2. Implement a module similar to `src/linux/linux.rs` for that OS.
  3. Add a `mod.rs` in the new folder.
  4. Update `main.rs` to detect the OS and call your new module.
  5. Add OS-specific ASCII art to `src/art/logos.rs` if needed.

## Coding Style
- Use Rust doc comments (`///`) for all public structs, fields, and functions.
- Keep code modular and prefer small, focused functions.
- Use `Result<T, String>` for error handling in public APIs.
- Follow [Rust formatting conventions](https://doc.rust-lang.org/1.0.0/style/style/whitespace.html).

## Reporting Issues
- Please use GitHub Issues to report bugs, request features, or ask questions.
- Include your OS, CPU model, and a copy of `/proc/cpuinfo` (if relevant).

## Pull Request Process
- All PRs are reviewed for correctness, style, and clarity.
- Please document your code and update this file if you add new major features or OS support.
- Tests are not required but are encouraged for complex logic.

---

Thank you for helping make **rcpufetch** better!