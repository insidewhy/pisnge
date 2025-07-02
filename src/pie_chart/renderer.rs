use crate::font::{load_system_font_bytes, measure_text_height, measure_text_width};
use crate::PieChart;
use std::f64::consts::PI;
use svg::node::element::{Circle, Group, Path, Rectangle, Style, Text};
use svg::Document;

const DEFAULT_COLORS: [&str; 10] = [
    "#1f77b4", "#ff7f0e", "#2ca02c", "#d62728", "#9467bd", "#8c564b", "#e377c2", "#7f7f7f",
    "#bcbd22", "#17becf",
];

pub fn render_pie_chart_svg(
    pie_chart: &PieChart,
    default_width: u32,
    height: u32,
    font_name: &str,
) -> (Document, u32, u32) {
    // Use config width if present, otherwise use default
    let width = pie_chart
        .config
        .as_ref()
        .and_then(|c| c.width)
        .unwrap_or(default_width);
    // Load font data once for both title and legend calculations
    let font_data = load_system_font_bytes(font_name);

    // Calculate the actual legend width needed
    let legend_width = calculate_legend_width(pie_chart, &font_data);

    // Calculate title height and spacing
    let (title_height, title_to_chart_gap) = if pie_chart.title.is_some() {
        let title_font_size = parse_font_size(
            get_theme_variable(pie_chart, "pieTitleTextSize", "25px"),
            25.0,
        );

        let text_height = if let Some(ref font_data) = font_data {
            measure_text_height(font_data, title_font_size) as f64
        } else {
            title_font_size as f64
        };
        (text_height, 20.0) // Title height + gap between title and chart
    } else {
        (0.0, 0.0) // No title, no gap
    };

    // Use consistent margins and spacing
    let vertical_margin = 35.0; // Equal top and bottom margin
    let side_margin = 30.0; // Equal left and right margin
    let chart_to_legend_gap = 20.0; // Gap between chart and legend

    // Calculate available space for the pie chart (width-constrained)
    let available_chart_width =
        width as f64 - (side_margin * 2.0) - legend_width - chart_to_legend_gap;

    // Calculate optimal radius based on width only (let height grow as needed)
    let radius = (available_chart_width / 2.0) * 0.9;

    // Calculate the actual height needed based on optimized content
    let legend_height = pie_chart.data.len() as f64 * 22.0; // 22px per legend item
    let chart_diameter = radius * 2.0;
    let content_height = chart_diameter.max(legend_height);
    let optimal_height = vertical_margin * 2.0 + title_height + title_to_chart_gap + content_height;

    // If optimal height exceeds CLI height, apply height constraint
    let (final_radius, actual_height) = if optimal_height > height as f64 {
        let available_chart_height =
            height as f64 - (vertical_margin * 2.0) - title_height - title_to_chart_gap;
        let constrained_radius =
            ((available_chart_width / 2.0).min(available_chart_height / 2.0)) * 0.9;
        (constrained_radius, height as f64)
    } else {
        (radius, optimal_height)
    };

    // Position elements
    let center_x = side_margin + available_chart_width / 2.0;
    let final_content_height = (final_radius * 2.0).max(legend_height);
    let center_y = vertical_margin + title_height + title_to_chart_gap + final_content_height / 2.0;

    let total: f64 = pie_chart.data.iter().map(|d| d.value).sum();

    let mut document = Document::new()
        .set("viewBox", (0, 0, width, actual_height as u32))
        .set("width", "100%")
        .set("height", actual_height as u32)
        .set("xmlns", "http://www.w3.org/2000/svg")
        .set(
            "style",
            format!("max-width: {}px; background-color: white;", width),
        );

    // Get theme variables with defaults
    let pie_opacity = get_theme_variable(pie_chart, "pieOpacity", "0.7");
    let pie_stroke_color = get_theme_variable(pie_chart, "pieStrokeColor", "black");
    let pie_outer_stroke_color = get_theme_variable(pie_chart, "pieOuterStrokeColor", "black");
    let pie_section_text_color = get_theme_variable(pie_chart, "pieSectionTextColor", "black");
    let pie_stroke_width = get_theme_variable(pie_chart, "pieStrokeWidth", "2px");
    let pie_outer_stroke_width = get_theme_variable(pie_chart, "pieOuterStrokeWidth", "2px");
    let pie_title_text_size = get_theme_variable(pie_chart, "pieTitleTextSize", "25px");
    let pie_title_text_color = get_theme_variable(pie_chart, "pieTitleTextColor", "black");
    let pie_section_text_size = get_theme_variable(pie_chart, "pieSectionTextSize", "17px");
    let pie_legend_text_size = get_theme_variable(pie_chart, "pieLegendTextSize", "17px");
    let pie_legend_text_color = get_theme_variable(pie_chart, "pieLegendTextColor", "black");

    let style = Style::new(&format!(
        r#"
            .pieCircle {{ stroke: {}; stroke-width: {}; fill-opacity: {}; }}
            .pieOuterCircle {{ stroke: {}; stroke-width: {}; fill: none; }}
            .pieTitleText {{ text-anchor: middle; font-size: {}; fill: {}; font-family: "{}", sans-serif; }}
            .slice {{ font-family: "{}", sans-serif; fill: {}; font-size: {}; text-anchor: middle; }}
            .legend text {{ fill: {}; font-family: "{}", sans-serif; font-size: {}; }}
        "#,
        pie_stroke_color,
        pie_stroke_width,
        pie_opacity,
        pie_outer_stroke_color,
        pie_outer_stroke_width,
        pie_title_text_size,
        pie_title_text_color,
        font_name,
        font_name,
        pie_section_text_color,
        pie_section_text_size,
        pie_legend_text_color,
        font_name,
        pie_legend_text_size
    ));

    document = document.add(style);

    let mut main_group =
        Group::new().set("transform", format!("translate({},{})", center_x, center_y));

    let mut current_angle = -PI / 2.0;

    for (i, data) in pie_chart.data.iter().enumerate() {
        let slice_angle = (data.value / total) * 2.0 * PI;
        let end_angle = current_angle + slice_angle;

        let color = get_color_for_slice(pie_chart, i);

        let path_data = create_pie_slice_path(0.0, 0.0, final_radius, current_angle, end_angle);

        main_group = main_group.add(
            Path::new()
                .set("class", "pieCircle")
                .set("fill", color)
                .set("d", path_data),
        );

        if pie_chart.show_data {
            let mid_angle = current_angle + slice_angle / 2.0;
            let label_radius = final_radius * 0.75;
            let label_x = label_radius * mid_angle.cos();
            let label_y = label_radius * mid_angle.sin();

            let percentage = ((data.value / total) * 100.0).round();

            let section_font_size = parse_font_size(
                get_theme_variable(pie_chart, "pieSectionTextSize", "17px"),
                17.0,
            );
            main_group = main_group.add(
                Text::new(format!("{}%", percentage))
                    .set("class", "slice")
                    .set("x", label_x)
                    .set("y", label_y)
                    .set("font-family", format!("{}, sans-serif", font_name))
                    .set("font-size", section_font_size.to_string())
                    .set("text-anchor", "middle")
                    .set("dominant-baseline", "central"),
            );
        }

        current_angle = end_angle;
    }

    // Add outer circle after segments to cover their outer stroke
    main_group = main_group.add(
        Circle::new()
            .set("class", "pieOuterCircle")
            .set("r", final_radius)
            .set("cx", 0)
            .set("cy", 0),
    );

    if let Some(title) = &pie_chart.title {
        main_group = main_group.add(
            Text::new(title.clone())
                .set("class", "pieTitleText")
                .set("x", 0)
                .set("y", -final_radius - 30.0)
                .set("font-family", format!("{}, sans-serif", font_name))
                .set("text-anchor", "middle"),
        );
    }

    // Add legend outside the main group, positioned with consistent right margin
    for (i, data) in pie_chart.data.iter().enumerate() {
        let legend_x = width as f64 - side_margin - legend_width; // Start of legend area with right margin
        let legend_y = center_y - (pie_chart.data.len() as f64 * 11.0) + (i as f64 * 22.0);
        let color = get_color_for_slice(pie_chart, i);

        let legend_group = Group::new()
            .set("class", "legend")
            .set("transform", format!("translate({},{})", legend_x, legend_y));

        let legend_group = legend_group
            .add(
                Rectangle::new()
                    .set("width", 18)
                    .set("height", 18)
                    .set("fill", color)
                    .set("stroke", pie_stroke_color)
                    .set("fill-opacity", pie_opacity),
            )
            .add(
                Text::new(format!("{} [{}]", data.label, data.value))
                    .set("x", 22)
                    .set("y", 14)
                    .set("font-family", format!("{}, sans-serif", font_name)),
            );

        document = document.add(legend_group);
    }

    (document.add(main_group), width, actual_height as u32)
}

fn calculate_legend_width(pie_chart: &PieChart, font_data: &Option<Vec<u8>>) -> f64 {
    let font_size = parse_font_size(
        get_theme_variable(pie_chart, "pieLegendTextSize", "17px"),
        17.0,
    );
    let icon_width = 18.0; // Width of the color rectangle
    let icon_margin = 22.0; // Space between icon and text
    let margin = 20.0; // Add more right margin for safety

    // Find the longest legend text
    let max_text_length = if let Some(font_data) = font_data {
        pie_chart
            .data
            .iter()
            .map(|data| {
                measure_text_width(
                    &format!("{} [{}]", data.label, data.value),
                    font_data,
                    font_size,
                )
            })
            .fold(0.0f32, f32::max) as f64
    } else {
        // Fallback to character width estimation if font loading fails
        let char_width = font_size as f64 * 0.53;
        pie_chart
            .data
            .iter()
            .map(|data| format!("{} [{}]", data.label, data.value).len() as f64 * char_width)
            .fold(0.0, f64::max)
    };

    icon_width + icon_margin + max_text_length + margin
}

fn get_color_for_slice(pie_chart: &PieChart, index: usize) -> &str {
    if let Some(config) = &pie_chart.config {
        let pie_key = format!("pie{}", index + 1);
        if let Some(color) = config.theme_variables.get(&pie_key) {
            return color;
        }
    }
    DEFAULT_COLORS[index % DEFAULT_COLORS.len()]
}

fn get_theme_variable<'a>(pie_chart: &'a PieChart, key: &str, default: &'a str) -> &'a str {
    if let Some(config) = &pie_chart.config {
        if let Some(value) = config.theme_variables.get(key) {
            return value;
        }
    }
    default
}

fn parse_font_size(font_size_str: &str, default: f32) -> f32 {
    if let Some(size_without_px) = font_size_str.strip_suffix("px") {
        size_without_px.parse().unwrap_or_else(|_| {
            eprintln!(
                "Warning: Invalid font size '{}', using default {}px",
                font_size_str, default
            );
            default
        })
    } else {
        eprintln!(
            "Warning: Font size '{}' must end with 'px', using default {}px",
            font_size_str, default
        );
        default
    }
}

fn create_pie_slice_path(
    cx: f64,
    cy: f64,
    radius: f64,
    start_angle: f64,
    end_angle: f64,
) -> String {
    let start_x = cx + radius * start_angle.cos();
    let start_y = cy + radius * start_angle.sin();
    let end_x = cx + radius * end_angle.cos();
    let end_y = cy + radius * end_angle.sin();

    let large_arc_flag = if end_angle - start_angle > PI { 1 } else { 0 };

    format!(
        "M{},{} A{},{},0,{},1,{},{} L{},{} Z",
        start_x, start_y, radius, radius, large_arc_flag, end_x, end_y, cx, cy
    )
}
