use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct PieChartConfig {
    pub theme: String,
    pub theme_variables: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PieChartData {
    pub label: String,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PieChart {
    pub config: Option<PieChartConfig>,
    pub show_data: bool,
    pub title: Option<String>,
    pub data: Vec<PieChartData>,
}

pub mod font;
pub mod parser;
pub mod png;
pub mod renderer;
