use nom::{
    bytes::complete::{tag, take_until},
    character::complete::{char, multispace0, space0},
    combinator::{map, opt},
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

fn x_axis_line(input: &str) -> IResult<&str, XAxis> {
    let (input, _) = tag("x-axis")(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('[')(input)?;
    let (input, labels) = separated_list0(
        tuple((space0, char(','), space0)),
        map(take_until_any(&[',', ']']), |s: &str| s.trim().to_string()),
    )(input)?;
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
