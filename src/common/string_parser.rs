use nom::{
    bytes::complete::take_until,
    character::complete::{char, multispace0},
    sequence::delimited,
    IResult,
};

/// Parse a double-quoted string
pub fn quoted_string(input: &str) -> IResult<&str, &str> {
    delimited(char('"'), take_until("\""), char('"'))(input)
}

/// Parse a single-quoted string
pub fn quoted_string_single(input: &str) -> IResult<&str, &str> {
    delimited(char('\''), take_until("'"), char('\''))(input)
}

/// Parse a label that can be either quoted (single or double quotes) or unquoted
/// When quoted, the label can contain commas
pub fn parse_label(input: &str) -> IResult<&str, String> {
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

/// Parse a list of labels enclosed in brackets
/// Labels can be quoted or unquoted, separated by commas
pub fn parse_labels_list(input: &str) -> IResult<&str, Vec<String>> {
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

/// Take until any of the specified characters is found
pub fn take_until_any(chars: &[char]) -> impl Fn(&str) -> IResult<&str, &str> + '_ {
    move |input: &str| {
        let mut end = 0;
        for (i, ch) in input.char_indices() {
            if chars.contains(&ch) {
                break;
            }
            end = i + ch.len_utf8();
        }

        if end == 0 && !chars.contains(&input.chars().next().unwrap_or('\0')) {
            end = input.len();
        }

        Ok((&input[end..], &input[..end]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_label() {
        // Test double-quoted string with comma
        let result = parse_label(r#""A,B""#);
        assert!(result.is_ok());
        let (_, label) = result.unwrap();
        assert_eq!(label, "A,B");

        // Test single-quoted string with comma
        let result = parse_label(r#"'C,D'"#);
        assert!(result.is_ok());
        let (_, label) = result.unwrap();
        assert_eq!(label, "C,D");

        // Test unquoted string
        let result = parse_label("SimpleLabel");
        assert!(result.is_ok());
        let (_, label) = result.unwrap();
        assert_eq!(label, "SimpleLabel");
    }

    #[test]
    fn test_parse_labels_list() {
        // Test mixed quoted and unquoted labels
        let result = parse_labels_list(r#""A,B", 'C,D', SimpleLabel, "Another, Label"]"#);
        assert!(result.is_ok());
        let (remaining, labels) = result.unwrap();
        assert_eq!(remaining, "]");
        assert_eq!(labels.len(), 4);
        assert_eq!(labels[0], "A,B");
        assert_eq!(labels[1], "C,D");
        assert_eq!(labels[2], "SimpleLabel");
        assert_eq!(labels[3], "Another, Label");
    }

    #[test]
    fn test_take_until_any() {
        let parser = take_until_any(&[',', ']']);

        let result = parser("hello,world");
        assert!(result.is_ok());
        let (remaining, parsed) = result.unwrap();
        assert_eq!(parsed, "hello");
        assert_eq!(remaining, ",world");

        let result = parser("hello]world");
        assert!(result.is_ok());
        let (remaining, parsed) = result.unwrap();
        assert_eq!(parsed, "hello");
        assert_eq!(remaining, "]world");
    }
}
