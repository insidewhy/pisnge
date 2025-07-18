use crate::common::ChartConfig;

#[derive(Debug, Clone, PartialEq)]
pub struct WorkItemMovement {
    pub config: Option<ChartConfig>,
    pub title: Option<String>,
    pub columns: Vec<String>,
    pub items: Vec<WorkItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkItem {
    pub id: String,
    pub from_state: String,
    pub from_points: f64,
    pub to_state: String,
    pub to_points: f64,
}

impl WorkItem {
    pub fn points_change(&self) -> f64 {
        self.to_points - self.from_points
    }
}

pub mod parser;
pub mod renderer;

pub use parser::*;
pub use renderer::*;
