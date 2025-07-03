use nom::{
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{char, digit1, multispace0, space0},
    combinator::{map, opt, recognize},
    multi::separated_list0,
    sequence::{delimited, tuple},
    IResult,
};
use std::fmt;

use super::{WorkItem, WorkItemMovement};
use crate::common::ChartConfig;

#[derive(Debug)]
pub struct ValidationError {
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ValidationError {}

fn header(input: &str) -> IResult<&str, ()> {
    let (input, _) = tag("work-item-movement")(input)?;
    Ok((input, ()))
}

fn quoted_string(input: &str) -> IResult<&str, &str> {
    delimited(char('\''), take_until("'"), char('\''))(input)
}

fn title_line(input: &str) -> IResult<&str, Option<String>> {
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("title")(input)?;
    let (input, _) = space0(input)?;
    let (input, title) = quoted_string(input)?;
    Ok((input, Some(title.to_string())))
}

fn columns_line(input: &str) -> IResult<&str, Vec<String>> {
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("columns")(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('[')(input)?;
    let (input, columns) = separated_list0(
        tuple((space0, char(','), space0)),
        map(take_while1(|c: char| c != ',' && c != ']'), |s: &str| {
            s.trim().to_string()
        }),
    )(input)?;
    let (input, _) = char(']')(input)?;
    Ok((input, columns))
}

fn number(input: &str) -> IResult<&str, i32> {
    map(digit1, |s: &str| s.parse().unwrap())(input)
}

fn work_item_id(input: &str) -> IResult<&str, &str> {
    recognize(tuple((
        take_while1(|c: char| c.is_alphabetic()),
        char('-'),
        digit1,
    )))(input)
}

fn state_with_points(input: &str) -> IResult<&str, (&str, i32)> {
    let (input, state) = take_until(":")(input)?;
    let (input, _) = char(':')(input)?;
    let (input, _) = space0(input)?;
    let (input, points) = number(input)?;
    Ok((input, (state.trim(), points)))
}

fn work_item_line(input: &str) -> IResult<&str, WorkItem> {
    let (input, _) = multispace0(input)?;
    let (input, id) = work_item_id(input)?;
    let (input, _) = space0(input)?;
    let (input, (from_state, from_points)) = state_with_points(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = tag("->")(input)?;
    let (input, _) = space0(input)?;
    let (input, (to_state, to_points)) = state_with_points(input)?;

    Ok((
        input,
        WorkItem {
            id: id.to_string(),
            from_state: from_state.to_string(),
            from_points,
            to_state: to_state.to_string(),
            to_points,
        },
    ))
}

pub fn parse_work_item_movement(
    input: &str,
    config: Option<ChartConfig>,
) -> IResult<&str, WorkItemMovement> {
    let (input, _) = header(input)?;
    let (input, _) = multispace0(input)?;
    let (input, title) = opt(title_line)(input)?;
    let (input, _) = multispace0(input)?;
    let (input, columns) = columns_line(input)?;
    let (input, _) = multispace0(input)?;
    let (input, items) = separated_list0(multispace0, work_item_line)(input)?;
    let (input, _) = multispace0(input)?;

    // Don't validate here - we'll validate in a separate function

    Ok((
        input,
        WorkItemMovement {
            config,
            title: title.flatten(),
            columns,
            items,
        },
    ))
}

/// Validates that all referenced states in work items exist in the columns list
pub fn validate_work_item_movement(chart: &WorkItemMovement) -> Result<(), ValidationError> {
    for item in &chart.items {
        // Case-insensitive check for from_state
        if !chart.columns.iter().any(|col| col.to_lowercase() == item.from_state.to_lowercase()) {
            return Err(ValidationError {
                message: format!(
                    "Work item '{}' references column '{}' which does not exist. Available columns are: {:?}",
                    item.id, item.from_state, chart.columns
                ),
            });
        }
        // Case-insensitive check for to_state
        if !chart.columns.iter().any(|col| col.to_lowercase() == item.to_state.to_lowercase()) {
            return Err(ValidationError {
                message: format!(
                    "Work item '{}' references column '{}' which does not exist. Available columns are: {:?}",
                    item.id, item.to_state, chart.columns
                ),
            });
        }
    }
    Ok(())
}

/// Parse and validate work item movement chart
pub fn parse_and_validate_work_item_movement(
    input: &str,
    config: Option<ChartConfig>,
) -> Result<WorkItemMovement, Box<dyn std::error::Error>> {
    let (_, chart) = parse_work_item_movement(input, config)
        .map_err(|e| format!("Failed to parse work item movement chart: {:?}", e))?;

    validate_work_item_movement(&chart)?;

    Ok(chart)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_work_item_movement() {
        let input = r#"work-item-movement
  title 'Work Item Changes'
  columns [Not Existing, Draft, To Do, In Progress, In Review, In Test, Done]
  PJ-633 Not Existing: 0 -> Draft: 1
  PJ-491 In Review: 3 -> Done: 3
  PJ-1 In Progress: 5 -> Draft: 8
"#;

        let result = parse_work_item_movement(input, None);
        assert!(result.is_ok(), "Failed to parse: {:?}", result);

        let (_, chart) = result.unwrap();
        assert_eq!(chart.title, Some("Work Item Changes".to_string()));
        assert_eq!(chart.columns.len(), 7);
        assert_eq!(chart.columns[0], "Not Existing");
        assert_eq!(chart.columns[6], "Done");
        assert_eq!(chart.items.len(), 3);

        let item = &chart.items[0];
        assert_eq!(item.id, "PJ-633");
        assert_eq!(item.from_state, "Not Existing");
        assert_eq!(item.from_points, 0);
        assert_eq!(item.to_state, "Draft");
        assert_eq!(item.to_points, 1);
        assert_eq!(item.points_change(), 1);

        let item = &chart.items[2];
        assert_eq!(item.id, "PJ-1");
        assert_eq!(item.from_state, "In Progress");
        assert_eq!(item.from_points, 5);
        assert_eq!(item.to_state, "Draft");
        assert_eq!(item.to_points, 8);
        assert_eq!(item.points_change(), 3);
    }
}
