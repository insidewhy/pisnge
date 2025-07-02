use clap::Parser;
use pisnge::common::parser::{parse_config_and_detect_type, ChartType};
use pisnge::pie_chart::{parse_pie_chart_content, render_pie_chart_svg};
use pisnge::png::svg_to_png;
use pisnge::work_item_movement::{parse_work_item_movement, render_work_item_movement_svg};
use pisnge::xychart::{parse_xychart_content, render_xychart_svg};
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

    #[arg(short, long, default_value = "800")]
    width: u32,

    #[arg(short = 'H', long, default_value = "600")]
    height: u32,

    #[arg(long, default_value = "Liberation Sans")]
    font: String,
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
        Ok(content) => {
            // Ensure content ends with newline for easier parsing
            let normalized_content = if content.ends_with('\n') {
                content
            } else {
                format!("{}\n", content)
            };

            // First parse config and detect chart type
            match parse_config_and_detect_type(&normalized_content) {
                Ok((_, (config, chart_type, remaining_content))) => {
                    if cli.verbose {
                        println!("\nDetected chart type: {:?}", chart_type);
                        if let Some(ref config) = config {
                            println!("Theme: {}", config.theme);
                            if !config.theme_variables.is_empty() {
                                println!("Theme variables: {:?}", config.theme_variables);
                            }
                        }
                    }

                    match chart_type {
                        ChartType::Pie => {
                            match parse_pie_chart_content(remaining_content, config) {
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
                                            let (svg_document, _, _) = render_pie_chart_svg(
                                                &pie_chart, cli.width, cli.height, &cli.font,
                                            );
                                            match fs::write(&cli.output, svg_document.to_string()) {
                                                Ok(_) => println!("SVG saved to: {}", cli.output),
                                                Err(e) => {
                                                    eprintln!("Failed to write SVG file: {}", e);
                                                    std::process::exit(1);
                                                }
                                            }
                                        }
                                        "png" => {
                                            let (svg_document, actual_width, actual_height) =
                                                render_pie_chart_svg(
                                                    &pie_chart, cli.width, cli.height, &cli.font,
                                                );
                                            match svg_to_png(
                                                &svg_document.to_string(),
                                                actual_width,
                                                actual_height,
                                                &cli.font,
                                            ) {
                                                Ok(png_data) => {
                                                    match fs::write(&cli.output, png_data) {
                                                        Ok(_) => {
                                                            println!("PNG saved to: {}", cli.output)
                                                        }
                                                        Err(e) => {
                                                            eprintln!(
                                                                "Failed to write PNG file: {}",
                                                                e
                                                            );
                                                            std::process::exit(1);
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    eprintln!(
                                                        "Failed to convert SVG to PNG: {}",
                                                        e
                                                    );
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
                            }
                        }
                        ChartType::XY => match parse_xychart_content(remaining_content, config) {
                            Ok((_, xychart)) => {
                                if cli.verbose {
                                    println!("\nParsed XY chart:");
                                    if let Some(title) = &xychart.title {
                                        println!("  Title: {}", title);
                                    }
                                    println!("  X-axis labels: {:?}", xychart.x_axis.labels);
                                    println!(
                                        "  Y-axis: \"{}\" {} -> {}",
                                        xychart.y_axis.title,
                                        xychart.y_axis.min,
                                        xychart.y_axis.max
                                    );
                                    println!("  Series count: {}", xychart.series.len());
                                    for (i, series) in xychart.series.iter().enumerate() {
                                        println!(
                                            "    Series {}: {:?} {:?}",
                                            i, series.series_type, series.data
                                        );
                                    }
                                }

                                match output_format.as_str() {
                                    "svg" => {
                                        let (svg_document, _, _) = render_xychart_svg(
                                            &xychart, cli.width, cli.height, &cli.font,
                                        );
                                        match fs::write(&cli.output, svg_document.to_string()) {
                                            Ok(_) => println!("SVG saved to: {}", cli.output),
                                            Err(e) => {
                                                eprintln!("Failed to write SVG file: {}", e);
                                                std::process::exit(1);
                                            }
                                        }
                                    }
                                    "png" => {
                                        let (svg_document, actual_width, actual_height) =
                                            render_xychart_svg(
                                                &xychart, cli.width, cli.height, &cli.font,
                                            );
                                        match svg_to_png(
                                            &svg_document.to_string(),
                                            actual_width,
                                            actual_height,
                                            &cli.font,
                                        ) {
                                            Ok(png_data) => {
                                                match fs::write(&cli.output, png_data) {
                                                    Ok(_) => {
                                                        println!("PNG saved to: {}", cli.output)
                                                    }
                                                    Err(e) => {
                                                        eprintln!(
                                                            "Failed to write PNG file: {}",
                                                            e
                                                        );
                                                        std::process::exit(1);
                                                    }
                                                }
                                            }
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
                                eprintln!("Failed to parse XY chart: {:?}", e);
                                std::process::exit(1);
                            }
                        },
                        ChartType::WorkItemMovement => {
                            match parse_work_item_movement(remaining_content, config) {
                                Ok((_, work_item_movement)) => {
                                    if cli.verbose {
                                        println!("\nParsed work item movement chart:");
                                        if let Some(title) = &work_item_movement.title {
                                            println!("  Title: {}", title);
                                        }
                                        println!("  Columns: {:?}", work_item_movement.columns);
                                        println!(
                                            "  Work items: {}",
                                            work_item_movement.items.len()
                                        );
                                        for item in &work_item_movement.items {
                                            println!(
                                                "    {}: {} ({}) -> {} ({})",
                                                item.id,
                                                item.from_state,
                                                item.from_points,
                                                item.to_state,
                                                item.to_points
                                            );
                                        }
                                    }

                                    match output_format.as_str() {
                                        "svg" => {
                                            let (svg_document, _, _) =
                                                render_work_item_movement_svg(
                                                    &work_item_movement,
                                                    cli.width,
                                                    &cli.font,
                                                );
                                            match fs::write(&cli.output, svg_document.to_string()) {
                                                Ok(_) => println!("SVG saved to: {}", cli.output),
                                                Err(e) => {
                                                    eprintln!("Failed to write SVG file: {}", e);
                                                    std::process::exit(1);
                                                }
                                            }
                                        }
                                        "png" => {
                                            let (svg_document, actual_width, actual_height) =
                                                render_work_item_movement_svg(
                                                    &work_item_movement,
                                                    cli.width,
                                                    &cli.font,
                                                );
                                            match svg_to_png(
                                                &svg_document.to_string(),
                                                actual_width,
                                                actual_height,
                                                &cli.font,
                                            ) {
                                                Ok(png_data) => {
                                                    match fs::write(&cli.output, png_data) {
                                                        Ok(_) => {
                                                            println!("PNG saved to: {}", cli.output)
                                                        }
                                                        Err(e) => {
                                                            eprintln!(
                                                                "Failed to write PNG file: {}",
                                                                e
                                                            );
                                                            std::process::exit(1);
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    eprintln!(
                                                        "Failed to convert SVG to PNG: {}",
                                                        e
                                                    );
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
                                    eprintln!("Failed to parse work item movement chart: {:?}", e);
                                    std::process::exit(1);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Failed to parse chart (unknown type or invalid config): {:?}",
                        e
                    );
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to read input file: {}", e);
            std::process::exit(1);
        }
    }
}
