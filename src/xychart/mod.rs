use crate::common::ChartConfig;

#[derive(Debug, Clone, PartialEq)]
pub struct XYChart {
    pub config: Option<ChartConfig>,
    pub title: Option<String>,
    pub x_axis: XAxis,
    pub y_axis: YAxis,
    pub series: Vec<Series>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct XAxis {
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct YAxis {
    pub title: String,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SeriesType {
    Bar,
    Line,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Series {
    pub series_type: SeriesType,
    pub data: Vec<f64>,
}

pub mod content_parser;
pub mod parser;
pub mod renderer;

pub use content_parser::*;
pub use parser::*;
pub use renderer::*;
