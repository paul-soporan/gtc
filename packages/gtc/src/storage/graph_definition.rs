//! GraphDefinition: mathematical definition (nodes + edges).
//! Implements GraphBase and StorageRepresentation/MutableStorage so it can act as a storage representation.

use crate::core::{EdgeId, NodeId};
use crate::interner::NodeInterner;
use crate::traits::{GraphBase, MutableStorage, StorageRepresentation};
use crate::{EdgeWeights, StorageConvert};
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Clone, Debug)]
pub struct EdgeRecord<EdgeMeta, Weight> {
    pub from: NodeId,
    pub to: NodeId,
    pub meta: EdgeMeta,
    pub weight: Option<Weight>,
}

impl<EdgeMeta, Weight> EdgeRecord<EdgeMeta, Weight> {
    pub fn new(from: NodeId, to: NodeId, meta: EdgeMeta, weight: Option<Weight>) -> Self {
        EdgeRecord {
            from,
            to,
            meta,
            weight,
        }
    }
}

#[derive(Clone)]
pub struct GraphDefinition<Key = String, Data = (), EdgeMeta = (), Weight = ()>
where
    Key: Debug + Clone + Eq + Hash,
    Data: Debug + Clone,
    EdgeMeta: Debug + Clone,
    Weight: Debug + Copy + PartialOrd,
{
    pub nodes: NodeInterner<Key, Data>,
    pub edges: Vec<EdgeRecord<EdgeMeta, Weight>>,
}

impl<Key, Data, EdgeMeta, Weight> GraphDefinition<Key, Data, EdgeMeta, Weight>
where
    Key: Debug + Clone + Eq + Hash,
    Data: Debug + Clone,
    EdgeMeta: Debug + Clone,
    Weight: Debug + Copy + PartialOrd,
{
    pub fn new() -> Self {
        Self {
            nodes: NodeInterner::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, key: Key, data: Data) -> NodeId {
        self.nodes.intern(key, data)
    }

    pub fn add_edge_by_id(
        &mut self,
        from: NodeId,
        to: NodeId,
        meta: EdgeMeta,
        weight: Option<Weight>,
    ) -> EdgeId {
        let id = EdgeId(self.edges.len());
        self.edges.push(EdgeRecord::new(from, to, meta, weight));
        id
    }

    pub fn add_edge_by_key(
        &mut self,
        from_key: Key,
        to_key: Key,
        from_data: Data,
        to_data: Data,
        meta: EdgeMeta,
        weight: Option<Weight>,
    ) -> EdgeId {
        let from = self.nodes.intern(from_key, from_data);
        let to = self.nodes.intern(to_key, to_data);
        self.add_edge_by_id(from, to, meta, weight)
    }

    pub fn order(&self) -> usize {
        self.nodes.len()
    }

    pub fn size(&self) -> usize {
        self.edges.len()
    }
}

impl<Key, Data, EdgeMeta, Weight> StorageRepresentation
    for GraphDefinition<Key, Data, EdgeMeta, Weight>
where
    Key: Debug + Clone + Eq + Hash,
    Data: Debug + Clone,
    EdgeMeta: Debug + Clone,
    Weight: Debug + Copy + PartialOrd,
{
    fn with_node_capacity(_capacity: usize) -> Self {
        Self::new()
    }
}

impl<Key, Data, EdgeMeta, Weight> MutableStorage for GraphDefinition<Key, Data, EdgeMeta, Weight>
where
    Key: Debug + Clone + Eq + Hash,
    Data: Debug + Clone,
    EdgeMeta: Debug + Clone,
    Weight: Debug + Copy + PartialOrd,
{
    fn add_node(&mut self, key: Self::Key, data: Self::Data) -> NodeId {
        self.nodes.intern(key, data)
    }

    fn add_edge_by_id(
        &mut self,
        from: NodeId,
        to: NodeId,
        meta: Self::EdgeMeta,
        weight: Option<Self::Weight>,
    ) -> EdgeId {
        let id = EdgeId(self.edges.len());
        self.edges.push(EdgeRecord::new(from, to, meta, weight));
        id
    }

    fn add_edge_by_key(
        &mut self,
        from_key: Self::Key,
        to_key: Self::Key,
        from_data: Self::Data,
        to_data: Self::Data,
        meta: Self::EdgeMeta,
        weight: Option<Self::Weight>,
    ) -> EdgeId {
        let from = self.add_node(from_key, from_data);
        let to = self.add_node(to_key, to_data);
        self.add_edge_by_id(from, to, meta, weight)
    }

    fn clear_edges(&mut self) {
        self.edges.clear();
    }
}

impl<K, D, E, W> GraphBase for GraphDefinition<K, D, E, W>
where
    K: Eq + std::hash::Hash + Clone + std::fmt::Debug,
    D: Clone + std::fmt::Debug,
    E: Clone + std::fmt::Debug,
    W: Copy + PartialOrd + Debug,
{
    type Key = K;
    type Data = D;
    type EdgeMeta = E;
    type Weight = W;

    fn order(&self) -> usize {
        self.order()
    }
    fn size(&self) -> usize {
        self.size()
    }

    fn node_id(&self, key: &Self::Key) -> Option<NodeId> {
        self.nodes.get_id(key)
    }
    fn node_ids(&self) -> Box<dyn Iterator<Item = NodeId> + '_> {
        Box::new((0..self.nodes.len()).map(NodeId))
    }
    fn node_key(&self, id: NodeId) -> &Self::Key {
        &self.nodes.get(id).key
    }
    fn node_data(&self, id: NodeId) -> &Self::Data {
        &self.nodes.get(id).data
    }

    fn edge_ids(&self) -> Box<dyn Iterator<Item = EdgeId> + '_> {
        Box::new((0..self.edges.len()).map(EdgeId))
    }

    fn endpoints(&self, e: EdgeId) -> (NodeId, NodeId) {
        let r = &self.edges[e.0];
        (r.from, r.to)
    }

    fn edge_meta(&self, e: EdgeId) -> &Self::EdgeMeta {
        &self.edges[e.0].meta
    }
    fn edges_between(&self, from: NodeId, to: NodeId) -> Box<dyn Iterator<Item = EdgeId> + '_> {
        let mut edge_ids = Vec::new();
        for (i, edge) in self.edges.iter().enumerate() {
            if edge.from == from && edge.to == to {
                edge_ids.push(EdgeId(i));
            }
        }
        Box::new(edge_ids.into_iter())
    }

    fn neighborhood(&self, v: NodeId) -> Box<dyn Iterator<Item = NodeId> + '_> {
        let mut neighbors = Vec::new();
        for edge in &self.edges {
            if edge.from == v {
                neighbors.push(edge.to);
            } else if edge.to == v {
                neighbors.push(edge.from);
            }
        }
        Box::new(neighbors.into_iter())
    }

    fn predecessors(&self, v: NodeId) -> Box<dyn Iterator<Item = NodeId> + '_> {
        let mut predecessors = Vec::new();
        for edge in &self.edges {
            if edge.to == v {
                predecessors.push(edge.from);
            }
        }
        Box::new(predecessors.into_iter())
    }
    fn successors(&self, v: NodeId) -> Box<dyn Iterator<Item = NodeId> + '_> {
        let mut successors = Vec::new();
        for edge in &self.edges {
            if edge.from == v {
                successors.push(edge.to);
            }
        }
        Box::new(successors.into_iter())
    }
}

impl<Key, Data, EdgeMeta, Weight> EdgeWeights for GraphDefinition<Key, Data, EdgeMeta, Weight>
where
    Key: Debug + Clone + Eq + Hash,
    Data: Debug + Clone,
    EdgeMeta: Debug + Clone,
    Weight: Debug + Copy + PartialOrd,
{
    type W = Weight;

    fn weight_of(&self, e: EdgeId) -> Option<Self::W> {
        self.edges.get(e.0).and_then(|er| er.weight)
    }
}

impl<Key, Data, EdgeMeta, Weight, Target> StorageConvert<Target>
    for GraphDefinition<Key, Data, EdgeMeta, Weight>
where
    Key: Debug + Clone + Eq + Hash,
    Data: Debug + Clone,
    EdgeMeta: Debug + Clone,
    Weight: Debug + Copy + PartialOrd,
    Target: StorageRepresentation<Key = Key, Data = Data, EdgeMeta = EdgeMeta, Weight = Weight>
        + MutableStorage<Key = Key, Data = Data, EdgeMeta = EdgeMeta, Weight = Weight>,
{
    fn convert(&self) -> Target {
        let mut target = Target::with_node_capacity(self.nodes.len());
        for node_id in self.node_ids() {
            let key = self.node_key(node_id).clone();
            let data = self.node_data(node_id).clone();
            target.add_node(key, data);
        }
        for edge_id in self.edge_ids() {
            let (from, to) = self.endpoints(edge_id);
            let meta = self.edge_meta(edge_id).clone();
            let weight = self.edges.get(edge_id.0).and_then(|er| er.weight).clone();
            target.add_edge_by_id(from, to, meta, weight);
        }
        target
    }
}
