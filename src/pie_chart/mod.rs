use crate::common::ChartConfig;

// Type alias for backward compatibility
pub type PieChartConfig = ChartConfig;

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

pub mod content_parser;
pub mod parser;
pub mod renderer;

pub use content_parser::*;
pub use parser::*;
pub use renderer::*;
