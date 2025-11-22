# Bromcom Timetable Formatter

[![CI](https://github.com/RichardSlater/bromcom-timetable-formatter/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/RichardSlater/bromcom-timetable-formatter/actions/workflows/ci.yml)
[![Release Workflow](https://github.com/RichardSlater/bromcom-timetable-formatter/actions/workflows/release.yml/badge.svg?event=push)](https://github.com/RichardSlater/bromcom-timetable-formatter/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

A small Rust workspace that parses Bromcom-produced PDF timetables and renders a printable A4 SVG-style weekly timetable, with a color-coded timetable grid and an embedded school map highlighting departments.

> [!IMPORTANT]
> I built this for my daughter who has some additional needs and finds it easier to associate colours with rooms; it is based upon parsing the information out of Bromcom circa 2025; PDF parsing is notably brittle and so this may not work without significant fixes for other timetables. I'm happy to accept Pull Requests if you find an issue, ideally with some anonymized examples.

This repository contains two crates:
- `crates/core` — library with parsing, configuration, overrides, map processing and rendering code
- `crates/cli` — command-line utility which wires config + PDF + map -> SVG output

## Quick Features

- Parse Bromcom timetable PDFs and reconstruct lessons (Subject, Room, Teacher, Class code)
- Configurable room-to-department mapping with separate background and foreground colors
- Per-week/day/period overrides via `config.toml` to correct parsing errors or make manual adjustments
- Render A4 sized SVGs containing a timetable grid and an embedded school map
- CLI flags to supply student name/form manually (fallback when PDF doesn't contain an extractable name)

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Configuration](#configuration)
- [Step-by-Step Tutorial](#step-by-step-tutorial)
- [Troubleshooting](#troubleshooting)
- [Architecture](#architecture)
- [Development](#development)
- [Contributing](#contributing)
- [License](#license)

## Installation

### Option 1: Install from Binary Release

Download pre-built binaries from the [Releases page](https://github.com/RichardSlater/bromcom-timetable-formatter/releases) for your platform:

- **Linux**: `timetable_cli-linux-x86_64.tar.gz` (or arm64, riscv64)
- **macOS**: `timetable_cli-macos-x86_64.tar.gz` or `timetable_cli-macos-arm64.tar.gz` (Apple Silicon)
- **Windows**: `timetable_cli-windows-x86_64.zip` (or arm64)
- **FreeBSD**: `timetable_cli-freebsd-x86_64.tar.gz`

Extract and run:

```bash
# Linux/macOS
tar -xzf timetable_cli-*.tar.gz
./timetable_cli --help

# Windows
# Extract the ZIP file, then run in PowerShell or CMD
timetable_cli.exe --help
```

### Option 2: Build from Source

Requires Rust 1.70+ ([install Rust](https://rustup.rs/)).

```bash
git clone https://github.com/RichardSlater/bromcom-timetable-formatter.git
cd bromcom-timetable-formatter
cargo build --release

# Binary will be at: target/release/timetable_cli
./target/release/timetable_cli --help
```

### Option 3: Install via Cargo

Once published to crates.io (package metadata added, publication pending):

```bash
cargo install timetable_cli
```

## Quick Start

```bash
# Build from source
cargo build --release

# Run with example (after creating config.toml and obtaining a PDF)
./target/release/timetable_cli \
  --input input/Sample_Student_Timetable.pdf \
  --config config.toml \
  --map resources/SchoolMap.svg \
  --output output/ \
  --student-name "Alex Testington" \
  --form "11XX"
```

Output SVG files will be in the `output/` directory, one per week.

## Configuration

Create a `config.toml` file with room mappings and optional overrides.

### Room Mappings

Maps room code prefixes to colors and map element IDs:

```toml
[[mappings]]
prefix = "MA"              # Matches MA1, MA2, MA3, etc.
bg_color = "#fcdcd8"       # Background color for cell and map
fg_color = "#e8a490"       # Foreground/text color for labels
map_id = "Maths_Rooms"     # SVG element ID in map file
label = "Maths"            # Human-readable label

[[mappings]]
prefix = "SC"
bg_color = "#fad7e6"
fg_color = "#e68cb8"
map_id = "Science_Rooms"
label = "Science"
```

**Fields**:
- `prefix` — Room code prefix to match (case-sensitive)
- `bg_color` — Hex color for cell background and map highlight
- `fg_color` — Hex color for label text (optional, defaults to `#231f20`)
- `map_id` — SVG element `id` or `data-name` attribute to highlight in map
- `label` — Display name for department (optional)

### Lesson Overrides

Correct parsing errors or make manual adjustments:

```toml
[[overrides]]
week = 2                   # Week number (1-based)
day = "Wednesday"          # Monday-Friday (or Mon-Fri)
period = "L3"              # PD, L1-L5
subject = "Geography"      # Optional: override subject
room = "HU3"               # Optional: override room
teacher = "Mr Smith"       # Optional: override teacher
class_code = "HU9"         # Optional: override class code
```

Only the fields you specify will be overridden—others remain from the PDF parse.

## Step-by-Step Tutorial

### 1. Obtain Required Files

**You need**:
- Bromcom timetable PDF (export from Bromcom system)
- School map SVG with labeled room areas
- Sample config.toml (modify the included example)

### 2. Create Your Configuration

Copy the example `config.toml` and customize:

```toml
# Map room prefixes to departments and colors
[[mappings]]
prefix = "MA"
bg_color = "#fcdcd8"
fg_color = "#e8a490"
map_id = "Maths_Rooms"
label = "Maths"

# Add more mappings for each department
# ...

# Add overrides if needed
[[overrides]]
week = 1
day = "Monday"
period = "L1"
room = "MA5"  # Correct parsing error
```

### 3. Prepare Your School Map

Your map SVG should have identifiable elements:

```svg
<svg>
  <g id="Maths_Rooms">
    <rect x="10" y="10" width="50" height="30" />
  </g>
  <g id="Science_Rooms">
    <rect x="70" y="10" width="50" height="30" />
  </g>
</svg>
```

The `id` (or `data-name`) attributes must match the `map_id` values in your config.

### 4. Run the Tool

```bash
./target/release/timetable_cli \
  --input input/my_timetable.pdf \
  --config config.toml \
  --map resources/SchoolMap.svg \
  --output output/
```

Optional flags:
- `--student-name "Name"` — Override extracted student name
- `--form "11XX"` — Override extracted form code

### 5. Check the Output

Find generated SVG files in `output/`:
- `Week_1_1.svg`
- `Week_2_2.svg`

Open in a web browser or vector editor (Inkscape, Illustrator) to preview. Print directly or export to PDF.

### 6. Troubleshooting Issues

See [Troubleshooting](#troubleshooting) below for common problems.

## Troubleshooting

### Problem: No SVG files generated

**Possible causes**:
- PDF file is not a valid Bromcom timetable
- PDF doesn't contain extractable text (scanned image PDF)
- Output directory doesn't exist

**Solutions**:
- Verify the PDF opens in a PDF reader and contains text (not just images)
- Create the output directory: `mkdir output`
- Check terminal output for error messages

### Problem: Lessons are missing or in wrong cells

**Possible causes**:
- PDF format variation (coordinate system differs)
- Text grouping tolerance too strict

**Solutions**:
- Use overrides to correct specific lessons
- Report the issue with an anonymized PDF sample
- Adjust tolerance values (requires code modification)

### Problem: Room colors not applied

**Possible causes**:
- Room prefix doesn't match configuration
- Room code format differs from expected

**Solutions**:
- Check that room codes in PDF match your `prefix` config
- Use longer, more specific prefixes (e.g., "MA1" instead of "M")
- Print debug info: Check console output for room codes found

### Problem: Map elements not highlighted

**Possible causes**:
- `map_id` doesn't match SVG element ID
- Map SVG structure is nested or uses different attributes

**Solutions**:
- Inspect your map SVG in a text editor
- Find the correct `id` or `data-name` attributes
- Ensure elements are direct children or descendants of labeled groups

### Problem: Student name or form not extracted

**Possible causes**:
- PDF doesn't contain name/form in expected location
- Text format differs from expected pattern

**Solutions**:
- Use CLI flags to override: `--student-name "Name" --form "XX"`
- These flags will be used when extraction fails

### Problem: Text is garbled or incorrect

**Possible causes**:
- Bromcom character encoding issue (uses +29 offset)
- PDF uses non-standard fonts

**Solutions**:
- The parser automatically handles standard Bromcom encoding
- Report the issue with PDF sample if characters are still wrong

### Problem: Build errors or missing dependencies

**Possible causes**:
- Rust version too old
- Missing system dependencies

**Solutions**:
- Update Rust: `rustup update`
- Ensure Rust 1.70+ is installed: `rustc --version`
- On Linux, install build essentials: `sudo apt install build-essential`

### Getting Help

- Check [GitHub Issues](https://github.com/RichardSlater/bromcom-timetable-formatter/issues) for similar problems
- Open a new issue with:
  - Exact command you ran
  - Full error output
  - OS and Rust version
  - Anonymized PDF sample if possible
- See [SUPPORT.md](SUPPORT.md) for more help channels

## Architecture

For detailed architecture documentation, module dependencies, and data flow diagrams, see [ARCHITECTURE.md](ARCHITECTURE.md).

Key architectural concepts:
- **Coordinate-based parsing**: Reconstructs grid from PDF text positions
- **Workspace structure**: Core library + CLI binary separation
- **Override system**: User can correct parsing errors without code changes
- **Embedded maps**: SVG maps are highlighted and embedded in output

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for development workflow, testing, and PR guidelines.

For a deeper look at unit vs integration tests, fixtures, and coverage goals, see [docs/testing.md](docs/testing.md).

Quick commands:

```bash
# Run tests
cargo test --workspace

# Format code
cargo fmt --all

# Lint code
cargo clippy --all-targets --all-features -- -D warnings

# Build release binary
cargo build --release

# Generate API documentation
cargo doc --open --no-deps
```

### Pre-commit Hooks

This repository includes a `.pre-commit-config.yaml` to run common checks before committing and pushing.

Install pre-commit (requires Python):

```bash
pip install pre-commit
pre-commit install
pre-commit install --hook-type pre-push
```

Hooks configured:
- YAML/formatting checks, whitespace trimming
- `cargo fmt --all -- --check` on commit
- `cargo clippy --all-targets --all-features -- -D warnings` on commit
- `cargo test --workspace` on push

**Note**: Running clippy and tests on every commit/push can be slow; you can skip via `git commit --no-verify` if necessary.

## Contributing

- See [docs/TODO.md](docs/TODO.md) for the current roadmap and planned features
- Review [CONTRIBUTING.md](CONTRIBUTING.md) for development, testing, and commit guidelines
- Please follow the [Code of Conduct](CODE_OF_CONDUCT.md) and [Support policy](SUPPORT.md)
- Security concerns should be reported privately (see [SECURITY.md](SECURITY.md))
- Open issues for bugs or feature requests using the provided templates
- Contributions welcome!

## License

MIT (see `LICENSE`)

## Author / Contact

[Richard Slater](https://richard-slater.co.uk)
