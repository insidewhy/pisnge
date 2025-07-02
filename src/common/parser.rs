use nom::{
    bytes::complete::tag, character::complete::multispace0, combinator::opt, sequence::preceded,
    IResult,
};

use super::{config_line, ChartConfig};

#[derive(Debug, Clone, PartialEq)]
pub enum ChartType {
    Pie,
    XY,
    WorkItemMovement,
}

pub fn detect_chart_type(input: &str) -> IResult<&str, ChartType> {
    let (input, _) = multispace0(input)?;

    // Try to match work-item-movement
    if let Ok((input, _)) = tag::<&str, &str, nom::error::Error<&str>>("work-item-movement")(input)
    {
        return Ok((input, ChartType::WorkItemMovement));
    }

    // Try to match xychart-beta
    if let Ok((input, _)) = tag::<&str, &str, nom::error::Error<&str>>("xychart-beta")(input) {
        return Ok((input, ChartType::XY));
    }

    // Try to match pie
    if let Ok((input, _)) = tag::<&str, &str, nom::error::Error<&str>>("pie")(input) {
        return Ok((input, ChartType::Pie));
    }

    Err(nom::Err::Error(nom::error::Error::new(
        input,
        nom::error::ErrorKind::Tag,
    )))
}

pub fn parse_config_and_detect_type(
    input: &str,
) -> IResult<&str, (Option<ChartConfig>, ChartType, &str)> {
    let (input, config) = opt(preceded(multispace0, config_line))(input)?;
    let (remaining, _) = multispace0(input)?;
    let (_, chart_type) = detect_chart_type(remaining)?;

    Ok(("", (config, chart_type, remaining)))
}
