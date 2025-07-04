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

### Pie Chart

```
pie title My Pie Chart
    "Label 1": 42
    "Label 2": 30
    "Label 3": 28
```

#### With Data Display

```
pie showData title Story Points by Status
    "Done": 262
    "To Do": 129
    "In Progress": 87
```

#### With Theme Configuration

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

### XY Chart

```
xychart-beta
  title "Issues in review or ready for QA"
  x-axis [PJ-213, PJ-341, PJ-481, PJ-482, PJ-420]
  y-axis "Number of days in status" 0 --> 10
  bar [2, 0, 6, 8, 9]
  bar [8.5, 7, 5, 3, 1]
```

#### With Theme Configuration and Legend

```
%%{init: {'theme': 'base', 'themeVariables': {"xyChart":{"plotColorPalette":"#ff8b00, #9c1de9"}}}}%%
xychart-beta
  title "Issues in review or ready for QA"
  legend [In Review, Ready for QA]
  x-axis [NP-213, NP-341, NP-481, NP-482, NP-420]
  y-axis "Number of days in status" 0 --> 10
  bar [2, 0, 6, 8, 9]
  bar [8.5, 7, 5, 3, 1]
```

## New Chart Types

The `work-item-movement` chart shows how work items (e.g. jira tickets) change story points and statuse over time:

```
work-item-movement
  title 'Work Item Changes'
  columns [Not Existing, Draft, To Do, In Progress, In Review, In Test, Done]
  PJ-633 Not Existing: 0 -> Draft: 1
  PJ-491 In Review: 3 -> Done: 3
  PJ-1 In Progress: 5 -> Draft: 8
```

## Differences to Mermaid

This project currently supports two types charts from mermaid and one new chart, for all charts only the `base` theme is supported with different default colors.

### Pie Charts

The pie chart segments are rendered in the order they are specified rather than from biggest to smallest, and the overall spacing is better since font widths/heights are measured directly.

### XY Charts

For each set of bars in the same axis the tallest bars are drawn first to ensure that bars don't get entirely covered.
Bars are always drawn before lines.
Bars can have a height of `0`, unlike mermaid, which will cause them not to be visible (mermaid will draw a short bar in this circumstance).
When `pisnge` detects that x-axis labels overlap each other it will automatically switch their orientation to be vertical.

Only a limited number of theme variables are currently supported:

- `titleFontSize`
- `labelFontSize`
- `plotColorPalette`

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

- [ ] Additional Mermaid diagram types
