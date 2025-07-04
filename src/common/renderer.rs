use crate::font::measure_text_width;
use svg::node::element::{Group, Rectangle, Text};

/// Configuration for legend rendering
pub struct LegendConfig {
    pub font_name: String,
    pub font_size: f64,
    pub icon_width: f64,
    pub icon_height: f64,
    pub icon_to_text_gap: f64,
    pub item_spacing: f64,
    pub right_margin: f64,
}

impl Default for LegendConfig {
    fn default() -> Self {
        Self {
            font_name: "Arial".to_string(),
            font_size: 17.0,
            icon_width: 18.0,
            icon_height: 18.0,
            icon_to_text_gap: 4.0,
            item_spacing: 22.0,
            right_margin: 20.0,
        }
    }
}

/// Calculate the width needed for the legend
pub fn calculate_legend_width(
    labels: &[String],
    font_data: &Option<Vec<u8>>,
    config: &LegendConfig,
) -> f64 {
    let icon_total_width = config.icon_width + config.icon_to_text_gap;

    // Find the longest legend text
    let max_text_width = if let Some(font_data) = font_data {
        labels
            .iter()
            .map(|label| measure_text_width(label, font_data, config.font_size as f32) as f64)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    } else {
        // Fallback: estimate based on character count
        labels
            .iter()
            .map(|label| label.len() as f64 * 8.0)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    };

    icon_total_width + max_text_width + config.right_margin
}

/// Render a legend at the specified position
pub fn render_legend(
    labels: &[String],
    colors: &[String],
    x: f64,
    y: f64,
    config: &LegendConfig,
) -> Group {
    let mut legend_group = Group::new().set("class", "legend");

    for (i, label) in labels.iter().enumerate() {
        let item_y = y + (i as f64 * config.item_spacing);
        let color = colors.get(i).map(|c| c.as_str()).unwrap_or("#000000");

        let item_group = Group::new().set("transform", format!("translate({},{})", x, item_y));

        let item_group = item_group
            .add(
                Rectangle::new()
                    .set("width", config.icon_width)
                    .set("height", config.icon_height)
                    .set("fill", color)
                    .set("stroke", "#000000")
                    .set("stroke-width", "1px")
                    .set("fill-opacity", "1"),
            )
            .add(
                Text::new(label.clone())
                    .set("x", config.icon_width + config.icon_to_text_gap)
                    .set("y", config.icon_height * 0.75) // Vertically center the text
                    .set("font-family", format!("{}, sans-serif", config.font_name))
                    .set("font-size", config.font_size.to_string()),
            );

        legend_group = legend_group.add(item_group);
    }

    legend_group
}

/// Calculate legend height based on number of items
pub fn calculate_legend_height(num_items: usize, config: &LegendConfig) -> f64 {
    num_items as f64 * config.item_spacing
}
