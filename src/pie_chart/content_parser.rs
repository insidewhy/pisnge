use nom::{
    bytes::complete::{tag, take_until},
    character::complete::{multispace0, space0},
    combinator::opt,
    multi::separated_list0,
    sequence::preceded,
    IResult,
};

use super::{PieChart, PieChartData};
use crate::common::{number, quoted_string, ChartConfig};

fn pie_header(input: &str) -> IResult<&str, (bool, Option<String>)> {
    let (input, _) = tag("pie")(input)?;
    let (input, _) = space0(input)?;

    let (input, show_data) = opt(tag("showData"))(input)?;
    let (input, _) = space0(input)?;

    let (input, title) = opt(preceded(tag("title "), take_until("\n")))(input)?;

    Ok((input, (show_data.is_some(), title.map(|s| s.to_string()))))
}

fn pie_data_entry(input: &str) -> IResult<&str, PieChartData> {
    let (input, _) = multispace0(input)?;
    let (input, label) = quoted_string(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, _) = space0(input)?;
    let (input, value) = number(input)?;

    Ok((
        input,
        PieChartData {
            label: label.to_string(),
            value,
        },
    ))
}

pub fn parse_pie_chart_content(
    input: &str,
    config: Option<ChartConfig>,
) -> IResult<&str, PieChart> {
    let (input, (show_data, title)) = pie_header(input)?;
    let (input, _) = multispace0(input)?;
    let (input, data) = separated_list0(multispace0, pie_data_entry)(input)?;
    let (input, _) = multispace0(input)?;

    Ok((
        input,
        PieChart {
            config,
            show_data,
            title,
            data,
        },
    ))
}
