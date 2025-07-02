use nom::{
    bytes::complete::{tag, take_until},
    character::complete::{char, multispace0, space0},
    combinator::opt,
    multi::separated_list0,
    sequence::{delimited, preceded, tuple},
    IResult,
};

use super::{Series, SeriesType, XAxis, XYChart, YAxis};
use crate::common::{number, ChartConfig};

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

pub fn parse_xychart_content(input: &str, config: Option<ChartConfig>) -> IResult<&str, XYChart> {
    let (input, title) = xy_header(input)?;
    let (input, _) = multispace0(input)?;
    let (input, x_axis) = x_axis_line(input)?;
    let (input, _) = multispace0(input)?;
    let (input, y_axis) = y_axis_line(input)?;
    let (input, _) = multispace0(input)?;
    let (input, series) = separated_list0(multispace0, series_line)(input)?;
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
