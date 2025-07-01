# Pisnge

A Rust-based diagram rendering library compatible with [mermaidjs](https://mermaid.js.org).

## Features

- **Mermaid Compatibility**: Parses Mermaid pie chart syntax (.mmd files)
- **SVG Output**: Generates clean, scalable SVG files with full text support
- **PNG Output**: Generates raster PNG files with full text support
- **CLI Interface**: Simple command-line tool for batch processing
- **Fast**: Much faster, smaller and more memory efficient than mermaid, does not need puppeteer

## Installation

### Pre-built Binaries (Ubuntu 22.04)

Download the latest binary from [Releases](../../releases):

```bash
# Download the binary
wget https://github.com/insidewhy/pisnge/releases/latest/download/pisnge-ubuntu-22.04

# Make it executable  
chmod +x pisnge-ubuntu-22.04

# Use it directly
./pisnge-ubuntu-22.04 --help
```

### From crates.io

```bash
cargo install pisnge
```

### From source

```bash
git clone https://github.com/insidewhy/pisnge
cd pisnge
cargo build --release
```

## Usage

### Command Line

```bash
# Basic usage
pisnge -i input.mmd -o output.svg

# Specify format (defaults to png)
pisnge -i chart.mmd -o chart.svg -f svg
pisnge -i chart.mmd -o chart.png -f png

# Custom font and dimensions
pisnge -i chart.mmd -o chart.svg --font "Arial" --width 1000 --height 800

# Verbose output
pisnge -i chart.mmd -o chart.svg --verbose

# If running from source
cargo run -- -i input.mmd -o output.svg
```

### Required Arguments

- `-i, --input`: Input Mermaid file (.mmd)
- `-o, --output`: Output file path

### Optional Arguments

- `-f, --format`: Output format - "png" or "svg" (defaults to "png")
- `-w, --width`: Desired maximum width
- `-H, --height`: Desired maximum height
- `--font`: Font family name (defaults to "Liberation Sans")
- `-v, --verbose`: Show detailed parsing information

Charts will be rendered up to the maximum of `width`/`height` and the unfilled dimension will be reduced in the output rather than introducing borders into the image.

## Mermaid Syntax Support

### Basic Pie Chart

```
pie title My Pie Chart
    "Label 1": 42
    "Label 2": 30
    "Label 3": 28
```

### With Data Display

```
pie showData title Story Points by Status
    "Done": 262
    "To Do": 129
    "In Progress": 87
```

### With Theme Configuration

```
%%{init: {'theme': 'base', 'themeVariables': {'pie1': '#ff6b6b', 'pie2': '#4ecdc4'}}}%%
pie showData title Custom Colors
    "Category A": 60
    "Category B": 40
```

```bash
# Render an example pie chart
pisnge -i examples/storypoints-by-status-pie.mmd -o output.svg

# Or if running from source
cargo run -- -i examples/storypoints-by-status-pie.mmd -o output.svg
```

## Architecture

- **Parser** (`src/parser.rs`): Uses nom to parse Mermaid pie chart syntax
- **Renderer** (`src/renderer.rs`): Generates SVG using the `svg` crate
- **Data Structures** (`src/lib.rs`): Defines `PieChart`, `PieChartData`, and `PieChartConfig`

## Differences to Mermaid

This project currently only supports pie charts and all mermaid options should be supported, however only the `base` theme is supported and the default colors are different.

The pie chart segments are rendered in the order they are specified rather than from biggest to smallest, and the overall spacing is better since font widths/heights are measured directly.

## Development

### Running Tests

```bash
cargo test
```

### Building

```bash
cargo build
```

## Dependencies

- `clap`: Command-line argument parsing
- `nom`: Parser combinator used to parse mmd syntax
- `svg`: SVG generation
- `resvg`: SVG to PNG conversion
- `tiny-skia`: 2D graphics rasterization
- `font-kit`: For loading system fonts
- `rusttype`: For measuring text widths/heights

## License

MIT License - see LICENSE file for details.

## Contributing

Contributions welcome! This library currently focuses on pie charts but could be extended to support other Mermaid diagram types.

## Roadmap

- [x] PNG output format
- [ ] Additional Mermaid diagram types (xy chart is planned soon)
- [ ] New diagram types not supported by mermaid
- [ ] More theming options than provided by mermaid
