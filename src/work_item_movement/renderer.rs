use super::WorkItemMovement;
use crate::font::{load_system_font_bytes, measure_text_height, measure_text_width};
use svg::node::element::{Circle, Group, Line, Path, Rectangle, Style, Text};
use svg::Document;

pub fn render_work_item_movement_svg(
    chart: &WorkItemMovement,
    default_width: u32,
    font_name: &str,
) -> (Document, u32, u32) {
    // Use config width if present, otherwise use default
    let width = chart
        .config
        .as_ref()
        .and_then(|c| c.width)
        .unwrap_or(default_width);
    let font_data = load_system_font_bytes(font_name);

    // Layout constants
    let margin = 20.0;
    let title_font_size = 20.0;
    let column_font_size = 16.0;
    let item_font_size = 14.0;
    let column_height = 40.0;
    let item_height = 50.0;
    let circle_radius = 15.0;
    let arrow_size = 12.0;
    let vertical_label_offset = 5.0; // Distance from line to start of text for vertical arrows

    // Calculate title height
    let (title_height, title_gap) = if chart.title.is_some() {
        let text_height = if let Some(ref font_data) = font_data {
            measure_text_height(font_data, title_font_size) as f64
        } else {
            title_font_size as f64
        };
        (text_height, 20.0)
    } else {
        (0.0, 0.0)
    };

    // Measure column text widths to calculate proper positioning
    let column_widths: Vec<f64> = chart
        .columns
        .iter()
        .map(|col| {
            if let Some(ref font_data) = font_data {
                measure_text_width(col, font_data, column_font_size) as f64
            } else {
                col.len() as f64 * 8.0
            }
        })
        .collect();

    // Calculate column positions based on vertical line placement
    let num_columns = chart.columns.len();

    let column_positions: Vec<f64> = if num_columns == 1 {
        vec![width as f64 / 2.0]
    } else {
        // Calculate the exact positions for first and last lines
        let first_line_pos = margin + column_widths[0] / 2.0;
        let last_line_pos = width as f64 - margin - column_widths[num_columns - 1] / 2.0;

        // Create positions array with first and last fixed, middle distributed evenly
        let mut positions = vec![0.0; num_columns];
        positions[0] = first_line_pos;
        positions[num_columns - 1] = last_line_pos;

        // Distribute middle positions evenly between first and last
        if num_columns > 2 {
            let spacing = (last_line_pos - first_line_pos) / (num_columns - 1) as f64;
            for i in 1..num_columns - 1 {
                positions[i] = first_line_pos + i as f64 * spacing;
            }
        }

        positions
    };

    // Calculate height needed - account for vertical arrows that need extra space
    let content_top = margin + title_height + title_gap;
    let items_top = content_top + column_height + 20.0;

    // Calculate total height needed, accounting for vertical arrows
    let line_extension = 15.0; // Must match the line_extension used for column lines
    let vertical_arrow_spacing = 80.0; // Space between circles in vertical arrows

    let height = if chart.items.is_empty() {
        (items_top + margin) as u32
    } else {
        // Calculate total height by simulating the same logic as rendering
        let mut calc_y = items_top;
        for item in &chart.items {
            let from_idx = chart
                .columns
                .iter()
                .position(|c| c == &item.from_state)
                .unwrap_or(0);
            let to_idx = chart
                .columns
                .iter()
                .position(|c| c == &item.to_state)
                .unwrap_or(0);

            if from_idx == to_idx {
                // Vertical arrow takes more space
                calc_y += vertical_arrow_spacing + item_height;
            } else {
                // Regular horizontal arrow
                calc_y += item_height;
            }
        }
        // Subtract one item_height since we added it for the last item
        // Then add circle radius and margin
        let final_y = calc_y - item_height + circle_radius;
        (final_y + line_extension + margin) as u32
    };

    // Create SVG document
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
            .column-label {{ font-size: {}px; fill: #131300; font-family: "{}", sans-serif; text-anchor: middle; }}
            .column-line {{ stroke: #e0e0e0; stroke-width: 1px; }}
            .item-label {{ font-size: {}px; fill: #131300; font-family: "{}", sans-serif; text-anchor: middle; }}
            .item-circle {{ fill: #131300; }}
            .item-arrow {{ stroke: #131300; stroke-width: 1px; fill: none; }}
            .arrow-head {{ fill: #131300; }}
            .circle-text {{ fill: white; font-size: {}px; font-family: \"{}\", sans-serif; text-anchor: middle; dominant-baseline: middle; font-weight: bold; }}
        "#,
        title_font_size,
        font_name,
        column_font_size,
        font_name,
        item_font_size,
        font_name,
        16.0,
        font_name
    ));
    document = document.add(style);

    // Background
    document = document.add(
        Rectangle::new()
            .set("fill", "white")
            .set("width", width)
            .set("height", height),
    );

    // Main group
    let mut main_group = Group::new().set("class", "main");

    // Title
    if let Some(title) = &chart.title {
        let title_y = margin + title_height / 2.0;
        main_group = main_group.add(
            Text::new(title)
                .set("class", "chart-title")
                .set("x", width as f64 / 2.0)
                .set("y", title_y)
                .set("text-anchor", "middle")
                .set("dominant-baseline", "middle"),
        );
    }

    // Column labels and lines
    for (i, column) in chart.columns.iter().enumerate() {
        let x = column_positions[i];

        // Column label
        main_group = main_group.add(
            Text::new(column)
                .set("class", "column-label")
                .set("x", x)
                .set("y", content_top + column_height / 2.0),
        );

        // Vertical line
        // Calculate where the first and last items would be
        let line_extension = 15.0; // Extra pixels above/below circles
        let first_item_y = items_top;

        // Calculate the actual last item position accounting for vertical arrows
        let last_item_y = if chart.items.is_empty() {
            first_item_y
        } else {
            let mut calc_y = items_top;
            for item in &chart.items {
                let from_idx = chart
                    .columns
                    .iter()
                    .position(|c| c == &item.from_state)
                    .unwrap_or(0);
                let to_idx = chart
                    .columns
                    .iter()
                    .position(|c| c == &item.to_state)
                    .unwrap_or(0);

                if from_idx == to_idx {
                    calc_y += vertical_arrow_spacing;
                }
                calc_y += item_height;
            }
            calc_y - item_height // Subtract the last increment
        };

        main_group = main_group.add(
            Line::new()
                .set("class", "column-line")
                .set("x1", x)
                .set("y1", first_item_y - circle_radius - line_extension)
                .set("x2", x)
                .set("y2", last_item_y + circle_radius + line_extension),
        );
    }

    // Work items - calculate Y positions accounting for vertical arrows
    let mut current_y = items_top;

    for (_item_idx, item) in chart.items.iter().enumerate() {
        let y = current_y;

        // Find column indices
        let from_idx = chart
            .columns
            .iter()
            .position(|c| c == &item.from_state)
            .unwrap_or(0);
        let to_idx = chart
            .columns
            .iter()
            .position(|c| c == &item.to_state)
            .unwrap_or(0);

        let from_x = column_positions[from_idx];
        let to_x = column_positions[to_idx];

        if from_idx == to_idx {
            // Same column - draw vertical arrow
            let x = from_x;

            // Draw circle at start (top)
            main_group = main_group.add(
                Circle::new()
                    .set("class", "item-circle")
                    .set("cx", x)
                    .set("cy", y)
                    .set("r", circle_radius),
            );

            // Add from points text in start circle
            main_group = main_group.add(
                Text::new(&item.from_points.to_string())
                    .set("class", "circle-text")
                    .set("x", x)
                    .set("y", y)
                    .set("text-anchor", "middle")
                    .set("dominant-baseline", "middle"),
            );

            // Draw circle at end (bottom)
            let end_y = y + vertical_arrow_spacing; // Use longer spacing
            main_group = main_group.add(
                Circle::new()
                    .set("class", "item-circle")
                    .set("cx", x)
                    .set("cy", end_y)
                    .set("r", circle_radius),
            );

            // Add to points text in end circle
            main_group = main_group.add(
                Text::new(&item.to_points.to_string())
                    .set("class", "circle-text")
                    .set("x", x)
                    .set("y", end_y)
                    .set("text-anchor", "middle")
                    .set("dominant-baseline", "middle"),
            );

            // Draw vertical arrow line
            let arrow_start_y = y + circle_radius;
            let arrow_end_y = end_y - circle_radius - arrow_size;

            main_group = main_group.add(
                Line::new()
                    .set("class", "item-arrow")
                    .set("x1", x)
                    .set("y1", arrow_start_y)
                    .set("x2", x)
                    .set("y2", arrow_end_y),
            );

            // Draw downward arrow head
            let arrow_tip_y = end_y - circle_radius;
            let arrow_points = format!(
                "{},{} {},{} {},{}",
                x,
                arrow_tip_y,
                x - arrow_size / 2.0,
                arrow_tip_y - arrow_size,
                x + arrow_size / 2.0,
                arrow_tip_y - arrow_size
            );

            main_group = main_group.add(
                Path::new()
                    .set("class", "arrow-head")
                    .set("d", format!("M {} Z", arrow_points)),
            );

            // Draw item label - position based on column
            let label_y = (y + end_y) / 2.0; // Middle of the arrow

            let mut label_text = item.id.clone();
            let points_change = item.points_change();
            if points_change != 0 {
                label_text.push_str(&format!(
                    ": {}{}",
                    if points_change > 0 { "+" } else { "" },
                    points_change
                ));
            }

            // Check if this is the last column
            let is_last_column = from_idx == chart.columns.len() - 1;

            if is_last_column {
                // For last column, put label on the left
                let label_x = x - vertical_label_offset;

                main_group = main_group.add(
                    Text::new(label_text)
                        .set("class", "item-label")
                        .set("x", label_x)
                        .set("y", label_y)
                        .set("style", "text-anchor: end") // End anchor so text ends at x position
                        .set("dominant-baseline", "middle"),
                );
            } else {
                // For other columns, put label on the right
                let label_x = x + vertical_label_offset;

                main_group = main_group.add(
                    Text::new(label_text)
                        .set("class", "item-label")
                        .set("x", label_x)
                        .set("y", label_y)
                        .set("style", "text-anchor: start") // Use inline style to override CSS class
                        .set("dominant-baseline", "middle"),
                );
            }
        } else {
            // Different columns - draw horizontal arrow
            // Draw circle at start
            main_group = main_group.add(
                Circle::new()
                    .set("class", "item-circle")
                    .set("cx", from_x)
                    .set("cy", y)
                    .set("r", circle_radius),
            );

            // Add from points text in start circle
            main_group = main_group.add(
                Text::new(&item.from_points.to_string())
                    .set("class", "circle-text")
                    .set("x", from_x)
                    .set("y", y)
                    .set("text-anchor", "middle")
                    .set("dominant-baseline", "middle"),
            );

            // Draw circle at end
            main_group = main_group.add(
                Circle::new()
                    .set("class", "item-circle")
                    .set("cx", to_x)
                    .set("cy", y)
                    .set("r", circle_radius),
            );

            // Add to points text in end circle
            main_group = main_group.add(
                Text::new(&item.to_points.to_string())
                    .set("class", "circle-text")
                    .set("x", to_x)
                    .set("y", y)
                    .set("text-anchor", "middle")
                    .set("dominant-baseline", "middle"),
            );

            // Draw horizontal arrow line
            let arrow_start_x = if from_idx < to_idx {
                from_x + circle_radius
            } else {
                from_x - circle_radius
            };
            let arrow_end_x = if from_idx < to_idx {
                to_x - circle_radius - arrow_size
            } else {
                to_x + circle_radius + arrow_size
            };

            main_group = main_group.add(
                Line::new()
                    .set("class", "item-arrow")
                    .set("x1", arrow_start_x)
                    .set("y1", y)
                    .set("x2", arrow_end_x)
                    .set("y2", y),
            );

            // Draw horizontal arrow head pointing to circle edge
            let arrow_points = if from_idx < to_idx {
                // Right-pointing arrow
                let arrow_tip_x = to_x - circle_radius;
                format!(
                    "{},{} {},{} {},{}",
                    arrow_tip_x,
                    y,
                    arrow_tip_x - arrow_size,
                    y - arrow_size / 2.0,
                    arrow_tip_x - arrow_size,
                    y + arrow_size / 2.0
                )
            } else {
                // Left-pointing arrow
                let arrow_tip_x = to_x + circle_radius;
                format!(
                    "{},{} {},{} {},{}",
                    arrow_tip_x,
                    y,
                    arrow_tip_x + arrow_size,
                    y - arrow_size / 2.0,
                    arrow_tip_x + arrow_size,
                    y + arrow_size / 2.0
                )
            };

            main_group = main_group.add(
                Path::new()
                    .set("class", "arrow-head")
                    .set("d", format!("M {} Z", arrow_points)),
            );

            // Draw item label above the line
            let label_x = (from_x + to_x) / 2.0;
            let label_y = y - 5.0; // Just 5 pixels above the line

            let mut label_text = item.id.clone();
            let points_change = item.points_change();
            if points_change != 0 {
                label_text.push_str(&format!(
                    ": {}{}",
                    if points_change > 0 { "+" } else { "" },
                    points_change
                ));
            }

            main_group = main_group.add(
                Text::new(label_text)
                    .set("class", "item-label")
                    .set("x", label_x)
                    .set("y", label_y)
                    .set("dominant-baseline", "text-after-edge"),
            );
        }

        // Update current_y for next item
        if from_idx == to_idx {
            // Vertical arrow takes more space
            current_y += vertical_arrow_spacing + item_height;
        } else {
            // Regular horizontal arrow
            current_y += item_height;
        }
    }

    document = document.add(main_group);

    (document, width, height)
}
