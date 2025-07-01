use nom::{
    bytes::complete::{tag, take_until},
    character::complete::{multispace0, space0},
    combinator::opt,
    multi::separated_list0,
    sequence::preceded,
    IResult,
};

use crate::common::{config_line, number, quoted_string};
use crate::{PieChart, PieChartData};

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

pub fn parse_pie_chart(input: &str) -> IResult<&str, PieChart> {
    let (input, config) = opt(preceded(multispace0, config_line))(input)?;
    let (input, _) = multispace0(input)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pie_chart() {
        let input = r#"%%{init: {'theme': 'dark', 'themeVariables': {'pieStrokeColor': 'white', 'pie1': 'blue'}}}%%
pie showData title Story points by status
  "Done": 262
  "To Do": 129
  "In test": 87
  "Ready for QA": 46
  "Blocked": 20
  "In Progress": 20
"#;

        let result = parse_pie_chart(input);
        assert!(result.is_ok());

        let (_, pie_chart) = result.unwrap();
        assert!(pie_chart.config.is_some());
        let config = pie_chart.config.as_ref().unwrap();
        assert_eq!(config.theme, "dark");
        assert_eq!(
            config.theme_variables.get("pieStrokeColor"),
            Some(&"white".to_string())
        );
        assert_eq!(
            config.theme_variables.get("pie1"),
            Some(&"blue".to_string())
        );
        assert_eq!(pie_chart.show_data, true);
        assert_eq!(pie_chart.title, Some("Story points by status".to_string()));
        assert_eq!(pie_chart.data.len(), 6);
        assert_eq!(pie_chart.data[0].label, "Done");
        assert_eq!(pie_chart.data[0].value, 262.0);
    }
}
