use super::{SeriesType, XYChart};
use crate::font::{load_system_font_bytes, measure_text_height, measure_text_width};
use svg::node::element::{Group, Path, Rectangle, Style, Text};
use svg::Document;

const DEFAULT_COLORS: [&str; 10] = [
    "#ff8b00", "#9c1de9", "#2ca02c", "#d62728", "#9467bd", "#8c564b", "#e377c2", "#7f7f7f",
    "#bcbd22", "#17becf",
];

pub fn render_xychart_svg(
    xychart: &XYChart,
    width: u32,
    height: u32,
    font_name: &str,
) -> (Document, u32) {
    let font_data = load_system_font_bytes(font_name);

    // Consistent margins around the chart
    let margin = 35.0; // Same as pie chart

    // Get configurable font sizes - match mermaid defaults more closely
    let title_font_size = get_theme_variable(xychart, "xyChart.titleFontSize", "20")
        .parse::<f32>()
        .unwrap_or(20.0);
    let label_font_size = get_theme_variable(xychart, "xyChart.labelFontSize", "16")
        .parse::<f32>()
        .unwrap_or(16.0);
    let axis_title_font_size = 16.0; // Match mermaid axis title size

    // Calculate title height and spacing
    let (title_height, title_to_chart_gap) = if xychart.title.is_some() {
        let text_height = if let Some(ref font_data) = font_data {
            measure_text_height(font_data, title_font_size) as f64
        } else {
            title_font_size as f64
        };
        (text_height, 20.0) // Title height + gap between title and chart
    } else {
        (0.0, 0.0) // No title, no gap
    };

    // Calculate the width of the longest Y-axis label
    let num_ticks = 11; // 0 to 10
    let max_y_label_width = if let Some(ref font_data) = font_data {
        let mut max_width = 0.0f32;
        for i in 0..num_ticks {
            let value = xychart.y_axis.max
                - (i as f64 * (xychart.y_axis.max - xychart.y_axis.min) / (num_ticks - 1) as f64);
            let label_text = format!("{}", value as i32);
            let width = measure_text_width(&label_text, font_data, label_font_size);
            max_width = max_width.max(width);
        }
        max_width as f64
    } else {
        // Fallback estimation
        label_font_size as f64 * 0.6 * 2.0 // Assume max 2 characters
    };

    // Check if we'll need vertical labels to calculate proper spacing
    let should_use_vertical_labels = if let Some(ref font_data) = font_data {
        let num_categories = xychart.x_axis.labels.len();
        let estimated_category_width =
            (width as f64 - (margin * 2.0) - (max_y_label_width + 35.0)) / num_categories as f64;
        check_label_overlap(
            &xychart.x_axis.labels,
            estimated_category_width,
            font_data,
            label_font_size,
        )
    } else {
        false
    };

    // Calculate the maximum label width for vertical labels
    let max_x_label_width = if should_use_vertical_labels && font_data.is_some() {
        let font_data = font_data.as_ref().unwrap();
        xychart
            .x_axis
            .labels
            .iter()
            .map(|label| measure_text_width(label, font_data, label_font_size) as f64)
            .fold(0.0, f64::max)
    } else {
        0.0
    };

    // Space needed for axes
    let y_axis_label_space = max_y_label_width + 35.0; // Measured label width + space for title and margins
    let x_axis_label_space = if should_use_vertical_labels {
        max_x_label_width + 20.0 // Width of longest label + margin
    } else {
        40.0 // Space for horizontal X-axis labels
    };

    // Calculate available space for the chart area
    let chart_width = width as f64 - (margin * 2.0) - y_axis_label_space;
    let chart_height =
        height as f64 - (margin * 2.0) - title_height - title_to_chart_gap - x_axis_label_space;

    // Calculate positions
    let chart_left = margin + y_axis_label_space;
    let chart_top = margin + title_height + title_to_chart_gap;
    let chart_bottom = chart_top + chart_height;
    let chart_right = chart_left + chart_width;

    let mut document = Document::new()
        .set("viewBox", (0, 0, width, height))
        .set("width", "100%")
        .set("height", height)
        .set("xmlns", "http://www.w3.org/2000/svg")
        .set(
            "style",
            format!("max-width: {}px; background-color: white;", width),
        );

    // Add CSS styles
    let style = Style::new(&format!(
        r#"
            .chart-title {{ text-anchor: middle; font-size: {}px; fill: #131300; font-family: "{}", sans-serif; }}
            .axis-line {{ stroke: #131300; stroke-width: 2px; fill: none; }}
            .axis-label {{ font-size: {}px; fill: #131300; font-family: "{}", sans-serif; }}
            .axis-title {{ font-size: {}px; fill: #131300; font-family: "{}", sans-serif; }}
            .tick {{ stroke: #131300; stroke-width: 2px; fill: none; }}
        "#,
        title_font_size, font_name, label_font_size, font_name, axis_title_font_size, font_name
    ));
    document = document.add(style);

    // Background
    document = document.add(
        Rectangle::new()
            .set("class", "background")
            .set("fill", "white")
            .set("width", width)
            .set("height", height),
    );

    // Main chart group
    let mut main_group = Group::new().set("class", "main");

    // Title
    if let Some(title) = &xychart.title {
        let title_y = margin + title_height / 2.0;
        main_group = main_group.add(
            Group::new().set("class", "chart-title").add(
                Text::new(title)
                    .set("class", "chart-title")
                    .set("x", width as f64 / 2.0)
                    .set("y", title_y)
                    .set("text-anchor", "middle")
                    .set("dominant-baseline", "middle"),
            ),
        );
    }

    // Calculate bar positioning for stacked bars
    let num_categories = xychart.x_axis.labels.len();
    let category_width = chart_width / num_categories as f64;
    let bar_width = category_width * 0.8; // Single width for stacked bars

    // Y-axis scaling
    let y_range = xychart.y_axis.max - xychart.y_axis.min;
    let y_scale = chart_height / y_range;

    // Create chart plot group
    let mut plot_group = Group::new().set("class", "plot");

    // Render bars first (so lines appear on top)
    for data_idx in 0..num_categories {
        let mut bars_for_position: Vec<(usize, f64, &str)> = Vec::new();

        // Collect all bars for this x position
        for (series_idx, series) in xychart.series.iter().enumerate() {
            if let SeriesType::Bar = series.series_type {
                if data_idx < series.data.len() {
                    let color = get_color_for_series(&xychart, series_idx);
                    bars_for_position.push((series_idx, series.data[data_idx], color));
                }
            }
        }

        // Sort by height (tallest first) so they render back to front
        bars_for_position
            .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Render bars for this position (tallest to shortest)
        for (series_idx, value, color) in bars_for_position {
            let x =
                chart_left + data_idx as f64 * category_width + (category_width - bar_width) / 2.0;
            let bar_height = (value - xychart.y_axis.min) * y_scale;
            let y = chart_bottom - bar_height;

            plot_group = plot_group.add(
                Rectangle::new()
                    .set("stroke-width", "0")
                    .set("stroke", color)
                    .set("fill", color)
                    .set("x", x)
                    .set("y", y)
                    .set("width", bar_width)
                    .set("height", bar_height)
                    .set("class", format!("bar-plot-{}", series_idx)),
            );
        }
    }

    // Render lines second (so they appear on top of bars)
    for (series_idx, series) in xychart.series.iter().enumerate() {
        if let SeriesType::Line = series.series_type {
            let color = get_color_for_series(&xychart, series_idx);
            let mut path_data = String::new();

            for (data_idx, &value) in series.data.iter().enumerate() {
                if data_idx >= num_categories {
                    break;
                }

                let x = chart_left + data_idx as f64 * category_width + category_width / 2.0;
                let y = chart_bottom - (value - xychart.y_axis.min) * y_scale;

                if data_idx == 0 {
                    path_data.push_str(&format!("M {},{}", x, y));
                } else {
                    path_data.push_str(&format!(" L {},{}", x, y));
                }
            }

            if !path_data.is_empty() {
                plot_group = plot_group.add(
                    Path::new()
                        .set("d", path_data)
                        .set("stroke", color)
                        .set("stroke-width", "2")
                        .set("fill", "none")
                        .set("class", format!("line-plot-{}", series_idx)),
                );
            }
        }
    }

    main_group = main_group.add(plot_group);

    // X-axis
    let mut x_axis_group = Group::new().set("class", "bottom-axis");

    // X-axis line
    x_axis_group = x_axis_group.add(Group::new().set("class", "axis-line").add(
        Path::new().set("class", "axis-line").set(
            "d",
            format!(
                "M {},{} L {},{}",
                chart_left, chart_bottom, chart_right, chart_bottom
            ),
        ),
    ));

    // Calculate label height for vertical labels if needed
    let label_height = if let Some(ref font_data) = font_data {
        measure_text_height(font_data, label_font_size) as f64
    } else {
        label_font_size as f64
    };

    // X-axis labels and ticks
    let mut x_labels_group = Group::new().set("class", "label");
    let mut x_ticks_group = Group::new().set("class", "ticks");

    for (i, label) in xychart.x_axis.labels.iter().enumerate() {
        let x = chart_left + i as f64 * category_width + category_width / 2.0;

        // Label - adjust positioning based on orientation
        if should_use_vertical_labels {
            x_labels_group = x_labels_group.add(
                Text::new(label)
                    .set("class", "axis-label")
                    .set("x", x)
                    .set("y", chart_bottom + 10.0 + label_height / 2.0)
                    .set("text-anchor", "end")
                    .set("dominant-baseline", "middle")
                    .set(
                        "transform",
                        format!(
                            "rotate(-90, {}, {})",
                            x,
                            chart_bottom + 10.0 + label_height / 2.0
                        ),
                    ),
            );
        } else {
            x_labels_group = x_labels_group.add(
                Text::new(label)
                    .set("class", "axis-label")
                    .set("x", x)
                    .set("y", chart_bottom + 20.0)
                    .set("text-anchor", "middle")
                    .set("dominant-baseline", "text-before-edge"),
            );
        }

        // Tick
        x_ticks_group = x_ticks_group.add(Path::new().set("class", "tick").set(
            "d",
            format!(
                "M {},{} L {},{}",
                x,
                chart_bottom + 1.0,
                x,
                chart_bottom + 6.0
            ),
        ));
    }

    x_axis_group = x_axis_group.add(x_labels_group);
    x_axis_group = x_axis_group.add(x_ticks_group);
    main_group = main_group.add(x_axis_group);

    // Y-axis
    let mut y_axis_group = Group::new().set("class", "left-axis");

    // Y-axis line
    y_axis_group = y_axis_group.add(Group::new().set("class", "axisl-line").add(
        Path::new().set("class", "axis-line").set(
            "d",
            format!(
                "M {},{} L {},{}",
                chart_left, chart_top, chart_left, chart_bottom
            ),
        ),
    ));

    // Y-axis labels and ticks
    let mut y_labels_group = Group::new().set("class", "label");
    let mut y_ticks_group = Group::new().set("class", "ticks");

    // Generate Y-axis ticks from max to min
    let num_ticks = 11; // 0 to 10
    for i in 0..num_ticks {
        let value = xychart.y_axis.max
            - (i as f64 * (xychart.y_axis.max - xychart.y_axis.min) / (num_ticks - 1) as f64);
        let y = chart_top + i as f64 * chart_height / (num_ticks - 1) as f64;

        // Label - position based on measured width
        y_labels_group = y_labels_group.add(
            Text::new(format!("{}", value as i32))
                .set("class", "axis-label")
                .set("x", chart_left - 10.0)
                .set("y", y)
                .set("text-anchor", "end")
                .set("dominant-baseline", "middle"),
        );

        // Tick
        y_ticks_group = y_ticks_group.add(Path::new().set("class", "tick").set(
            "d",
            format!("M {},{} L {},{}", chart_left - 1.0, y, chart_left - 6.0, y),
        ));
    }

    y_axis_group = y_axis_group.add(y_labels_group);
    y_axis_group = y_axis_group.add(y_ticks_group);

    // Y-axis title - position so left edge matches right margin distance
    let y_title_x = margin; // Same distance from left edge as chart is from right edge
    let y_title_y = chart_top + chart_height / 2.0;
    y_axis_group = y_axis_group.add(
        Group::new().set("class", "title").add(
            Text::new(&xychart.y_axis.title)
                .set("class", "axis-title")
                .set("x", y_title_x)
                .set("y", y_title_y)
                .set("text-anchor", "middle")
                .set("dominant-baseline", "text-before-edge")
                .set(
                    "transform",
                    format!("rotate(270, {}, {})", y_title_x, y_title_y),
                ),
        ),
    );

    main_group = main_group.add(y_axis_group);

    document = document.add(main_group);

    (document, height)
}

fn get_theme_variable<'a>(xychart: &'a XYChart, key: &str, default: &'a str) -> &'a str {
    if let Some(config) = &xychart.config {
        if let Some(value) = config.theme_variables.get(key) {
            return value;
        }
    }
    default
}

fn check_label_overlap(
    labels: &[String],
    category_width: f64,
    font_data: &[u8],
    font_size: f32,
) -> bool {
    let min_gap = 5.0; // Minimum gap between labels in pixels

    for label in labels.iter() {
        let label_width = measure_text_width(label, font_data, font_size) as f64;
        // If any label width + minimum gap exceeds category width, we need vertical labels
        if label_width + min_gap > category_width {
            return true;
        }
    }
    false
}

fn get_color_for_series(xychart: &XYChart, index: usize) -> &str {
    if let Some(config) = &xychart.config {
        // Check for xyChart.plotColorPalette
        if let Some(palette) = config.theme_variables.get("xyChart.plotColorPalette") {
            let colors: Vec<&str> = palette.split(',').map(|s| s.trim()).collect();
            if index < colors.len() {
                return colors[index];
            }
        }
    }
    DEFAULT_COLORS[index % DEFAULT_COLORS.len()]
}
