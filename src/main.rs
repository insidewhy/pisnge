use clap::Parser;
use pisnge::parser::parse_pie_chart;
use pisnge::png::svg_to_png;
use pisnge::renderer::render_pie_chart_svg;
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(name = "pisnge")]
#[command(about = "A Rust-based diagram rendering library inspired by Mermaid.js")]
struct Cli {
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: String,

    #[arg(short, long, value_parser = ["png", "svg"])]
    format: Option<String>,

    #[arg(short, long)]
    verbose: bool,
}

fn detect_format_from_extension(output_path: &str) -> Option<String> {
    Path::new(output_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
        .and_then(|ext| match ext.as_str() {
            "png" => Some("png".to_string()),
            "svg" => Some("svg".to_string()),
            _ => None,
        })
}

fn main() {
    let cli = Cli::parse();

    println!("Pisnge - Diagram Renderer");

    // Determine output format: use -f flag if provided, otherwise detect from file extension
    let output_format = match cli.format {
        Some(format) => format,
        None => match detect_format_from_extension(&cli.output) {
            Some(format) => format,
            None => {
                eprintln!("Error: Could not detect output format from file extension '{}'. Please specify format using -f flag.", 
                    Path::new(&cli.output).extension().and_then(|ext| ext.to_str()).unwrap_or("(none)"));
                eprintln!("Supported formats: png, svg");
                std::process::exit(1);
            }
        },
    };

    if cli.verbose {
        println!("Input file: {}", cli.input);
        println!("Output file: {}", cli.output);
        println!("Output format: {}", output_format);
    }

    match fs::read_to_string(&cli.input) {
        Ok(content) => match parse_pie_chart(&content) {
            Ok((_, pie_chart)) => {
                if cli.verbose {
                    println!("\nParsed pie chart:");
                    println!("  Show data: {}", pie_chart.show_data);
                    if let Some(title) = &pie_chart.title {
                        println!("  Title: {}", title);
                    }
                    println!("  Data entries: {}", pie_chart.data.len());
                    for entry in &pie_chart.data {
                        println!("    \"{}\": {}", entry.label, entry.value);
                    }
                }

                match output_format.as_str() {
                    "svg" => {
                        let svg_document = render_pie_chart_svg(&pie_chart, 647, 450);
                        match fs::write(&cli.output, svg_document.to_string()) {
                            Ok(_) => println!("SVG saved to: {}", cli.output),
                            Err(e) => {
                                eprintln!("Failed to write SVG file: {}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                    "png" => {
                        let svg_document = render_pie_chart_svg(&pie_chart, 647, 450);
                        match svg_to_png(&svg_document.to_string(), 647, 450) {
                            Ok(png_data) => match fs::write(&cli.output, png_data) {
                                Ok(_) => println!("PNG saved to: {}", cli.output),
                                Err(e) => {
                                    eprintln!("Failed to write PNG file: {}", e);
                                    std::process::exit(1);
                                }
                            },
                            Err(e) => {
                                eprintln!("Failed to convert SVG to PNG: {}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                    _ => {
                        eprintln!(
                            "Unsupported format: {}. Supported formats: png, svg",
                            output_format
                        );
                        std::process::exit(1);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to parse pie chart: {:?}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("Failed to read input file: {}", e);
            std::process::exit(1);
        }
    }
}
