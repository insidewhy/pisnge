use nom::{
    bytes::complete::{tag, take_until},
    character::complete::{char, multispace0, space0},
    combinator::opt,
    multi::separated_list0,
    sequence::{delimited, preceded, tuple},
    IResult,
};

use super::{Series, SeriesType, XAxis, XYChart, YAxis};
use crate::common::{config_line, number};

fn xy_header(input: &str) -> IResult<&str, Option<String>> {
    let (input, _) = tag("xychart-beta")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, title) = opt(preceded(tag("title "), quoted_string))(input)?;
    Ok((input, title.map(|s| s.to_string())))
}

fn quoted_string(input: &str) -> IResult<&str, &str> {
    delimited(char('"'), take_until("\""), char('"'))(input)
}

fn quoted_string_single(input: &str) -> IResult<&str, &str> {
    delimited(char('\''), take_until("'"), char('\''))(input)
}

fn parse_label(input: &str) -> IResult<&str, String> {
    let (input, _) = multispace0(input)?;

    // Try parsing as double-quoted string
    if let Ok((input, content)) = quoted_string(input) {
        return Ok((input, content.to_string()));
    }

    // Try parsing as single-quoted string
    if let Ok((input, content)) = quoted_string_single(input) {
        return Ok((input, content.to_string()));
    }

    // Parse as unquoted string (until comma or closing bracket)
    let (input, content) = take_until_any(&[',', ']'])(input)?;
    Ok((input, content.trim().to_string()))
}

fn parse_labels_list(input: &str) -> IResult<&str, Vec<String>> {
    let mut labels = Vec::new();
    let mut remaining = input;

    loop {
        // Skip whitespace
        let (input, _) = multispace0(remaining)?;
        remaining = input;

        // Check if we've reached the end bracket
        if remaining.starts_with(']') {
            break;
        }

        // Parse a label
        let (input, label) = parse_label(remaining)?;
        labels.push(label);
        remaining = input;

        // Skip whitespace
        let (input, _) = multispace0(remaining)?;
        remaining = input;

        // Check for comma or end bracket
        if remaining.starts_with(',') {
            let (input, _) = char(',')(remaining)?;
            remaining = input;
        } else if remaining.starts_with(']') {
            break;
        } else {
            return Err(nom::Err::Error(nom::error::Error::new(
                remaining,
                nom::error::ErrorKind::Char,
            )));
        }
    }

    Ok((remaining, labels))
}

fn x_axis_line(input: &str) -> IResult<&str, XAxis> {
    let (input, _) = tag("x-axis")(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('[')(input)?;
    let (input, labels) = parse_labels_list(input)?;
    let (input, _) = char(']')(input)?;

    Ok((input, XAxis { labels }))
}

fn take_until_any(chars: &[char]) -> impl Fn(&str) -> IResult<&str, &str> + '_ {
    move |input: &str| {
        let mut end = 0;
        for (i, ch) in input.char_indices() {
            if chars.contains(&ch) {
                break;
            }
            end = i + ch.len_utf8();
        }
        if end == 0 {
            Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::TakeUntil,
            )))
        } else {
            Ok((&input[end..], &input[..end]))
        }
    }
}

fn y_axis_line(input: &str) -> IResult<&str, YAxis> {
    let (input, _) = tag("y-axis")(input)?;
    let (input, _) = space0(input)?;
    let (input, title) = quoted_string(input)?;
    let (input, _) = space0(input)?;
    let (input, min) = number(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = tag("-->")(input)?;
    let (input, _) = space0(input)?;
    let (input, max) = number(input)?;

    Ok((
        input,
        YAxis {
            title: title.to_string(),
            min,
            max,
        },
    ))
}

fn series_line(input: &str) -> IResult<&str, Series> {
    let (input, series_type_str) = take_until_any(&[' ', '\t'])(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('[')(input)?;
    let (input, data) = separated_list0(tuple((space0, char(','), space0)), number)(input)?;
    let (input, _) = char(']')(input)?;

    let series_type = match series_type_str.trim() {
        "bar" => SeriesType::Bar,
        "line" => SeriesType::Line,
        _ => SeriesType::Bar, // Default to bar
    };

    Ok((input, Series { series_type, data }))
}

fn chart_content(input: &str) -> IResult<&str, (Option<String>, XAxis, YAxis, Vec<Series>)> {
    let (input, title) = xy_header(input)?;
    let (input, _) = multispace0(input)?;
    let (input, x_axis) = x_axis_line(input)?;
    let (input, _) = multispace0(input)?;
    let (input, y_axis) = y_axis_line(input)?;
    let (input, _) = multispace0(input)?;
    let (input, series) = separated_list0(multispace0, series_line)(input)?;

    Ok((input, (title, x_axis, y_axis, series)))
}

pub fn parse_xychart(input: &str) -> IResult<&str, XYChart> {
    let (input, config) = opt(preceded(multispace0, config_line))(input)?;
    let (input, _) = multispace0(input)?;
    let (input, (title, x_axis, y_axis, series)) = chart_content(input)?;
    let (input, _) = multispace0(input)?;

    Ok((
        input,
        XYChart {
            config,
            title,
            x_axis,
            y_axis,
            series,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_xychart() {
        let input = r##"%%{init: {'theme': 'base', 'themeVariables': {"xyChart":{"plotColorPalette":"#ff8b00, #9c1de9"}}}}%%
xychart-beta
  title "Issues in review or ready for QA"
  x-axis [NP-213, NP-341, NP-481, NP-482, NP-420]
  y-axis "Number of days in status" 0 --> 10
  bar [2, 4, 6, 8, 9]
  bar [8.5, 7, 5, 3, 1]
"##;

        let result = parse_xychart(input);
        assert!(result.is_ok(), "Failed to parse: {:?}", result);

        let (_, xychart) = result.unwrap();
        assert!(xychart.config.is_some());
        let config = xychart.config.as_ref().unwrap();
        assert_eq!(config.theme, "base");
        assert_eq!(
            xychart.title,
            Some("Issues in review or ready for QA".to_string())
        );
        assert_eq!(xychart.x_axis.labels.len(), 5);
        assert_eq!(xychart.x_axis.labels[0], "NP-213");
        assert_eq!(xychart.y_axis.title, "Number of days in status".to_string());
        assert_eq!(xychart.y_axis.min, 0.0);
        assert_eq!(xychart.y_axis.max, 10.0);
        assert_eq!(xychart.series.len(), 2);
        assert_eq!(xychart.series[0].series_type, SeriesType::Bar);
        assert_eq!(xychart.series[0].data, vec![2.0, 4.0, 6.0, 8.0, 9.0]);
        assert_eq!(xychart.series[1].data, vec![8.5, 7.0, 5.0, 3.0, 1.0]);
    }

    #[test]
    fn test_parse_label_function() {
        // Test quoted string with comma
        let result = parse_label(r#""A,B""#);
        assert!(result.is_ok(), "Failed to parse quoted label: {:?}", result);
        let (_, label) = result.unwrap();
        assert_eq!(label, "A,B");

        // Test single quoted string with comma
        let result = parse_label(r#"'C,D'"#);
        assert!(
            result.is_ok(),
            "Failed to parse single quoted label: {:?}",
            result
        );
        let (_, label) = result.unwrap();
        assert_eq!(label, "C,D");
    }

    #[test]
    fn test_parse_labels_list() {
        // Test simple quoted labels with commas
        let result = parse_labels_list(r#""A,B", "C,D"]"#);
        assert!(result.is_ok(), "Failed to parse labels list: {:?}", result);
        let (remaining, labels) = result.unwrap();
        assert_eq!(remaining, "]");
        assert_eq!(labels.len(), 2);
        assert_eq!(labels[0], "A,B");
        assert_eq!(labels[1], "C,D");
    }

    #[test]
    fn test_parse_xychart_with_quoted_labels() {
        let input = r##"xychart-beta
  title "Test Chart"
  x-axis ["Label with, comma", 'Another, with comma', "Simple Label", UnquotedLabel]
  y-axis "Values" 0 --> 100
  bar [10, 20, 30, 40]
"##;

        let result = parse_xychart(input);
        assert!(result.is_ok(), "Failed to parse: {:?}", result);

        let (_, xychart) = result.unwrap();
        assert_eq!(xychart.x_axis.labels.len(), 4);
        assert_eq!(xychart.x_axis.labels[0], "Label with, comma");
        assert_eq!(xychart.x_axis.labels[1], "Another, with comma");
        assert_eq!(xychart.x_axis.labels[2], "Simple Label");
        assert_eq!(xychart.x_axis.labels[3], "UnquotedLabel");
    }
}
