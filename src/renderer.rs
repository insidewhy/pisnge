use crate::PieChart;
use std::f64::consts::PI;
use svg::node::element::{Circle, Group, Path, Rectangle, Style, Text};
use svg::Document;

const DEFAULT_COLORS: [&str; 10] = [
    "#1f77b4", "#ff7f0e", "#2ca02c", "#d62728", "#9467bd", "#8c564b", "#e377c2", "#7f7f7f",
    "#bcbd22", "#17becf",
];

pub fn render_pie_chart_svg(pie_chart: &PieChart, width: u32, height: u32) -> Document {
    // Calculate the actual legend width needed
    let legend_width = calculate_legend_width(pie_chart);
    let legend_margin = 15.0; // Closer spacing between chart and legend
    let chart_area_width = width as f64 - legend_width - legend_margin;

    let center_x = chart_area_width / 2.0;
    let center_y = (height as f64 / 2.0) + 20.0; // Move down by 20px
    let radius = (chart_area_width.min(height as f64) / 2.0) * 0.85; // Larger pie chart

    let total: f64 = pie_chart.data.iter().map(|d| d.value).sum();

    let mut document = Document::new()
        .set("viewBox", (0, 0, width, height))
        .set("width", "100%")
        .set("xmlns", "http://www.w3.org/2000/svg")
        .set("style", "max-width: 647.141px; background-color: white;");

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
        .pieCircle {{ stroke: {}; stroke-width: {}; opacity: {}; }}
        .pieOuterCircle {{ stroke: {}; stroke-width: {}; fill: none; }}
        .pieTitleText {{ text-anchor: middle; font-size: {}; fill: {}; font-family: "Liberation Sans", "DejaVu Sans", "Noto Sans", sans-serif; }}
        .slice {{ font-family: "Liberation Sans", "DejaVu Sans", "Noto Sans", sans-serif; fill: {}; font-size: {}; text-anchor: middle; }}
        .legend text {{ fill: {}; font-family: "Liberation Sans", "DejaVu Sans", "Noto Sans", sans-serif; font-size: {}; }}
    "#,
        pie_stroke_color,
        pie_stroke_width,
        pie_opacity,
        pie_outer_stroke_color,
        pie_outer_stroke_width,
        pie_title_text_size,
        pie_title_text_color,
        pie_section_text_color,
        pie_section_text_size,
        pie_legend_text_color,
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

        let path_data = create_pie_slice_path(0.0, 0.0, radius, current_angle, end_angle);

        main_group = main_group.add(
            Path::new()
                .set("class", "pieCircle")
                .set("fill", color)
                .set("d", path_data),
        );

        if pie_chart.show_data {
            let mid_angle = current_angle + slice_angle / 2.0;
            let label_radius = radius * 0.75;
            let label_x = label_radius * mid_angle.cos();
            let label_y = label_radius * mid_angle.sin();

            let percentage = ((data.value / total) * 100.0).round();

            main_group = main_group.add(
                Text::new(format!("{}%", percentage))
                    .set("class", "slice")
                    .set("x", label_x)
                    .set("y", label_y)
                    .set(
                        "font-family",
                        "Liberation Sans, DejaVu Sans, Noto Sans, sans-serif",
                    )
                    .set("font-size", "17")
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
            .set("r", radius)
            .set("cx", 0)
            .set("cy", 0),
    );

    if let Some(title) = &pie_chart.title {
        main_group = main_group.add(
            Text::new(title.clone())
                .set("class", "pieTitleText")
                .set("x", 0)
                .set("y", -radius - 30.0)
                .set(
                    "font-family",
                    "Liberation Sans, DejaVu Sans, Noto Sans, sans-serif",
                )
                .set("text-anchor", "middle"),
        );
    }

    // Add legend outside the main group, positioned in the reserved space
    for (i, data) in pie_chart.data.iter().enumerate() {
        let legend_x = chart_area_width + legend_margin; // Start of legend area with proper spacing
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
                    .set("stroke", color),
            )
            .add(
                Text::new(format!("{} [{}]", data.label, data.value))
                    .set("x", 22)
                    .set("y", 14)
                    .set(
                        "font-family",
                        "Liberation Sans, DejaVu Sans, Noto Sans, sans-serif",
                    ),
            );

        document = document.add(legend_group);
    }

    document.add(main_group)
}

fn calculate_legend_width(pie_chart: &PieChart) -> f64 {
    let font_size = 17.0;
    let char_width = font_size * 0.6; // Approximate character width
    let icon_width = 18.0; // Width of the color rectangle
    let icon_margin = 22.0; // Space between icon and text
    let margin = 5.0; // Smaller right margin from edge

    // Find the longest legend text
    let max_text_length = pie_chart
        .data
        .iter()
        .map(|data| format!("{} [{}]", data.label, data.value).len())
        .max()
        .unwrap_or(0) as f64;

    icon_width + icon_margin + (max_text_length * char_width) + margin
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
