use nom::{
    bytes::complete::tag,
    character::complete::{char, multispace0, space0},
    combinator::opt,
    multi::separated_list0,
    sequence::{preceded, tuple},
    IResult,
};

use super::{Series, SeriesType, XAxis, XYChart, YAxis};
use crate::common::{
    number,
    string_parser::{parse_labels_list, quoted_string, take_until_any},
    ChartConfig,
};

fn xy_header(input: &str) -> IResult<&str, Option<String>> {
    let (input, _) = tag("xychart-beta")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, title) = opt(preceded(tag("title "), quoted_string))(input)?;
    Ok((input, title.map(|s| s.to_string())))
}

fn x_axis_line(input: &str) -> IResult<&str, XAxis> {
    let (input, _) = tag("x-axis")(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('[')(input)?;
    let (input, labels) = parse_labels_list(input)?;
    let (input, _) = char(']')(input)?;

    Ok((input, XAxis { labels }))
}

fn legend_line(input: &str) -> IResult<&str, Vec<String>> {
    let (input, _) = char('[')(input)?;
    let (input, labels) = parse_labels_list(input)?;
    let (input, _) = char(']')(input)?;

    Ok((input, labels))
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
    let (input, legend) = opt(preceded(tuple((tag("legend"), space0)), legend_line))(input)?;
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
            legend,
            x_axis,
            y_axis,
            series,
        },
    ))
}
