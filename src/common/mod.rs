use nom::{
    bytes::complete::{tag, take_until},
    character::complete::{char, digit1},
    combinator::{map, opt, recognize},
    sequence::{delimited, tuple},
    IResult,
};
use std::collections::HashMap;

pub mod parser;

#[derive(Debug, Clone, PartialEq)]
pub struct ChartConfig {
    pub theme: String,
    pub theme_variables: HashMap<String, String>,
    pub width: Option<u32>,
}

pub fn quoted_string(input: &str) -> IResult<&str, &str> {
    delimited(char('"'), take_until("\""), char('"'))(input)
}

pub fn number(input: &str) -> IResult<&str, f64> {
    map(
        recognize(tuple((
            opt(char('-')),
            digit1,
            opt(tuple((char('.'), digit1))),
        ))),
        |s: &str| s.parse().unwrap(),
    )(input)
}

pub fn config_line(input: &str) -> IResult<&str, ChartConfig> {
    let (input, _) = tag("%%{init: ")(input)?;
    let (input, config_content) = take_until("}%%")(input)?;
    let (input, _) = tag("}%%")(input)?;

    let mut theme = "base".to_string();
    let mut theme_variables = HashMap::new();
    let mut width = None;

    // Parse theme
    if let Some(theme_start) = config_content.find("'theme': '") {
        let theme_content = &config_content[theme_start + 10..];
        if let Some(theme_end) = theme_content.find("'") {
            theme = theme_content[..theme_end].to_string();
        }
    }

    // Parse width
    if let Some(width_start) = config_content.find("'width': ") {
        let width_content = &config_content[width_start + 9..];
        // Find the end of the number (either comma or closing brace)
        let width_end = width_content
            .find(',')
            .or(width_content.find('}'))
            .unwrap_or(width_content.len());
        if let Ok(w) = width_content[..width_end].trim().parse::<u32>() {
            width = Some(w);
        }
    }

    // Parse themeVariables - handle nested objects
    if let Some(vars_start) = config_content.find("'themeVariables': {") {
        let vars_content = &config_content[vars_start + 19..];
        parse_theme_variables(vars_content, &mut theme_variables);
    }

    Ok((
        input,
        ChartConfig {
            theme,
            theme_variables,
            width,
        },
    ))
}

fn parse_theme_variables(content: &str, theme_variables: &mut HashMap<String, String>) {
    let mut brace_count = 0;
    let mut current_key = String::new();
    let mut current_value = String::new();
    let mut in_key = false;
    let mut in_value = false;
    let mut in_quotes = false;
    let mut chars = content.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '{' => {
                brace_count += 1;
                if brace_count == 1 && !in_quotes {
                    // Start of nested object
                    if !current_key.is_empty() {
                        // Parse nested object
                        let mut nested_content = String::new();
                        let mut nested_brace_count = 1;
                        while let Some(nested_ch) = chars.next() {
                            if nested_ch == '{' && !in_quotes {
                                nested_brace_count += 1;
                            } else if nested_ch == '}' && !in_quotes {
                                nested_brace_count -= 1;
                                if nested_brace_count == 0 {
                                    break;
                                }
                            } else if nested_ch == '"' {
                                in_quotes = !in_quotes;
                            }
                            nested_content.push(nested_ch);
                        }

                        // Parse the nested content as key-value pairs
                        let key_prefix = current_key.trim().trim_matches('\'').trim_matches('"');
                        parse_nested_theme_variables(&nested_content, key_prefix, theme_variables);
                        current_key.clear();
                        brace_count = 0;
                        in_key = false;
                        in_value = false;
                    }
                } else if !in_quotes {
                    current_value.push(ch);
                }
            }
            '}' => {
                if brace_count > 0 {
                    brace_count -= 1;
                }
                if brace_count == 0 && !in_quotes {
                    // End of current level
                    if !current_key.is_empty() && !current_value.is_empty() {
                        let key = current_key.trim().trim_matches('\'').trim_matches('"');
                        let value = current_value.trim().trim_matches('\'').trim_matches('"');
                        theme_variables.insert(key.to_string(), value.to_string());
                    }
                    break;
                } else if !in_quotes {
                    current_value.push(ch);
                }
            }
            '"' | '\'' => {
                in_quotes = !in_quotes;
                if in_value {
                    current_value.push(ch);
                } else if in_key {
                    current_key.push(ch);
                }
            }
            ':' if !in_quotes => {
                in_key = false;
                in_value = true;
            }
            ',' if !in_quotes && brace_count == 0 => {
                if !current_key.is_empty() && !current_value.is_empty() {
                    let key = current_key.trim().trim_matches('\'').trim_matches('"');
                    let value = current_value.trim().trim_matches('\'').trim_matches('"');
                    theme_variables.insert(key.to_string(), value.to_string());
                }
                current_key.clear();
                current_value.clear();
                in_key = true;
                in_value = false;
            }
            _ => {
                if !in_key && !in_value && ch.is_alphabetic() {
                    in_key = true;
                }

                if in_value {
                    current_value.push(ch);
                } else if in_key {
                    current_key.push(ch);
                }
            }
        }
    }

    // Handle final key-value pair
    if !current_key.is_empty() && !current_value.is_empty() {
        let key = current_key.trim().trim_matches('\'').trim_matches('"');
        let value = current_value.trim().trim_matches('\'').trim_matches('"');
        theme_variables.insert(key.to_string(), value.to_string());
    }
}

fn parse_nested_theme_variables(
    content: &str,
    prefix: &str,
    theme_variables: &mut HashMap<String, String>,
) {
    // Parse nested key-value pairs, handling quoted strings with commas
    let mut chars = content.chars().peekable();
    let mut current_key = String::new();
    let mut current_value = String::new();
    let mut in_key = false;
    let mut in_value = false;
    let mut in_quotes = false;
    let mut quote_char = '"';

    while let Some(ch) = chars.next() {
        match ch {
            '"' | '\'' => {
                if !in_quotes {
                    in_quotes = true;
                    quote_char = ch;
                } else if ch == quote_char {
                    in_quotes = false;
                }
                // Don't include quotes in the final values
            }
            ':' if !in_quotes => {
                in_key = false;
                in_value = true;
            }
            ',' if !in_quotes => {
                if !current_key.is_empty() && !current_value.is_empty() {
                    let key = current_key.trim();
                    let value = current_value.trim();
                    let full_key = format!("{}.{}", prefix, key);
                    theme_variables.insert(full_key, value.to_string());
                }
                current_key.clear();
                current_value.clear();
                in_key = false;
                in_value = false;
            }
            _ => {
                if !in_key && !in_value && (ch.is_alphabetic() || ch == '"' || ch == '\'') {
                    in_key = true;
                }

                if in_value {
                    current_value.push(ch);
                } else if in_key && ch != '"' && ch != '\'' {
                    current_key.push(ch);
                }
            }
        }
    }

    // Handle final key-value pair
    if !current_key.is_empty() && !current_value.is_empty() {
        let key = current_key.trim();
        let value = current_value.trim();
        let full_key = format!("{}.{}", prefix, key);
        theme_variables.insert(full_key, value.to_string());
    }
}
