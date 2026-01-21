//! Capability traits and markers used across storage and wrappers.

use crate::{
    Weight,
    core::{EdgeId, NodeId},
};
use std::{fmt::Debug, hash::Hash};

/// Minimal read-only graph trait for storage and wrappers.
pub trait GraphBase {
    type Key: Debug + Clone;
    type Data: Debug + Clone;
    type EdgeMeta: Debug + Clone;
    type Weight: Debug + Copy + PartialOrd;

    fn order(&self) -> usize;
    fn size(&self) -> usize;

    fn node_id(&self, key: &Self::Key) -> Option<NodeId>;
    fn node_ids(&self) -> Box<dyn Iterator<Item = NodeId> + '_>;
    fn node_key(&self, id: NodeId) -> &Self::Key;
    fn node_data(&self, id: NodeId) -> &Self::Data;

    fn edge_ids(&self) -> Box<dyn Iterator<Item = EdgeId> + '_>;
    fn endpoints(&self, e: EdgeId) -> (NodeId, NodeId);
    fn edge_meta(&self, e: EdgeId) -> &Self::EdgeMeta;
    fn edges_between(&self, from: NodeId, to: NodeId) -> Box<dyn Iterator<Item = EdgeId> + '_>;

    fn neighborhood(&self, v: NodeId) -> Box<dyn Iterator<Item = NodeId> + '_>;

    fn successors(&self, v: NodeId) -> Box<dyn Iterator<Item = NodeId> + '_>;
    fn predecessors(&self, v: NodeId) -> Box<dyn Iterator<Item = NodeId> + '_>;
}

/// Edge weight lookup
pub trait EdgeWeights {
    type W: Debug + Copy + PartialOrd;
    fn weight_of(&self, e: EdgeId) -> Option<Self::W>;
}

/// Marker trait: storage types implement this to mark they are a storage representation.
/// Associated types define node key/data and edge meta/weight types to propagate through wrappers.
/// StorageRepresentation now requires GraphBase so storage types must also implement GraphBase.
pub trait StorageRepresentation: GraphBase
where
    <Self as GraphBase>::Key: Eq + Hash,
{
    fn with_node_capacity(capacity: usize) -> Self;
}

/// Mutable storage operations (add/remove nodes & edges). Implemented by storage structs that are mutable.
/// Uses associated types from StorageRepresentation.
pub trait MutableStorage: StorageRepresentation
where
    <Self as GraphBase>::Key: Eq + Hash,
{
    fn add_node(&mut self, key: Self::Key, data: Self::Data) -> NodeId;
    fn add_edge_by_id(
        &mut self,
        from: NodeId,
        to: NodeId,
        meta: Self::EdgeMeta,
        weight: Option<Self::Weight>,
    ) -> EdgeId;
    fn add_edge_by_key(
        &mut self,
        from_key: Self::Key,
        to_key: Self::Key,
        from_data: Self::Data,
        to_data: Self::Data,
        meta: Self::EdgeMeta,
        weight: Option<Self::Weight>,
    ) -> EdgeId;
    fn clear_edges(&mut self);
}

/// Trait for converting between storage representations (expensive, may allocate).
/// Implementations should convert `Self` into `Target` storage type.
pub trait StorageConvert<Target> {
    fn convert(&self) -> Target;
}

/// Graph kind marker traits (type-level markers)
/// These are empty marker traits implemented by zero-sized types you can pass to wrappers.
/// Use these marker types as the GK generic parameter on wrappers.
pub trait GraphKindMarker {}
pub trait SimpleGraphKind: GraphKindMarker {}
pub trait PseudoGraphKind: GraphKindMarker {}
pub trait MultiGraphKind: GraphKindMarker {}

/// Compile-time weight markers to allow different APIs for unit vs non-unit weight types.
///
/// - `IsUnit` is implemented only for `()` (the unit weight).
/// - `NotUnit` is implemented for numeric weight types (the `Weight` trait).
///
/// This enables conditional method availability:
/// - `add_edge` (no weight parameter) when `W = ()` (IsUnit)
/// - `add_edge_with_weight` when `W` is non-unit (NotUnit)
pub trait IsUnit {}
pub trait NotUnit {}

impl IsUnit for () {}
// NotUnit implemented for any type that implements the Weight trait (numeric-like)
impl<T> NotUnit for T where T: Weight {}

/// Marker for nodes that have total ordering (placeholder)
pub trait OrderedNodes {}

/// Merge strategies (placeholder)
#[derive(Clone, Debug)]
pub enum MergeStrategy {
    /// Relabel nodes (default)
    Relabel,
    /// Merge by key equality
    MergeByKey,
    /// Merge by globally provided UID (requires UID in node data)
    MergeByUid,
}
