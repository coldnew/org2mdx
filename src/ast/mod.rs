use serde::Serialize;

pub mod node;

pub use node::Node;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Position {
    pub start: Point,
    pub end: Point,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Point {
    pub line: u32,
    pub column: u32,
    pub offset: u32,
}
