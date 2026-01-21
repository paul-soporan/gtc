//! core small types

use std::fmt::Debug;

/// Typed node/edge identifiers
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct EdgeId(pub usize);

/// Representation hint for algorithms
#[derive(Copy, Clone, Debug)]
pub enum RepresentationHint {
    PreferAdjList,
    PreferAdjMatrix,
    Auto,
    ForceAdjList,
    ForceAdjMatrix,
}

/// Minimal numeric weight trait
pub trait Weight:
    Copy + PartialOrd + std::ops::Add<Output = Self> + Debug + Send + Sync + 'static
{
    fn zero() -> Self;
}

impl Weight for f32 {
    fn zero() -> Self {
        0.0
    }
}
impl Weight for f64 {
    fn zero() -> Self {
        0.0
    }
}

impl Weight for i32 {
    fn zero() -> Self {
        0
    }
}
impl Weight for i64 {
    fn zero() -> Self {
        0
    }
}
