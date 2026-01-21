//! Dense adjacency matrix stored as flat Vec<Option<EdgeId>> referencing edges Vec.
//! Edges Vec keeps edge records (including weight & meta). This way the matrix is a presence map of edges.
//! This matches the proposed design to avoid storing both weight and meta in the matrix cells.

use indexmap::IndexSet;

use crate::core::{EdgeId, NodeId};
use crate::interner::NodeInterner;
use crate::storage::graph_definition::{EdgeRecord as GEdgeRecord, GraphDefinition};
use crate::traits::{
    EdgeWeights, GraphBase, MutableStorage, StorageConvert, StorageRepresentation,
};
use std::fmt::Debug;
use std::hash::Hash;

pub type EdgeRecord<EdgeMeta, Weight> = GEdgeRecord<EdgeMeta, Weight>;

#[derive(Clone)]
pub struct AdjacencyMatrix<Key = String, Data = (), EdgeMeta = (), Weight = ()>
where
    Key: Debug + Clone + Eq + Hash,
    Data: Debug + Clone,
    EdgeMeta: Debug + Clone,
    Weight: Debug + Copy + PartialOrd,
{
    pub n: usize,
    pub nodes: NodeInterner<Key, Data>,
    pub edges: Vec<EdgeRecord<EdgeMeta, Weight>>,
    pub data: Vec<Option<EdgeId>>,
}

impl<Key, Data, EdgeMeta, Weight> StorageRepresentation
    for AdjacencyMatrix<Key, Data, EdgeMeta, Weight>
where
    Key: Debug + Clone + Eq + Hash + Default,
    Data: Debug + Clone + Default,
    EdgeMeta: Debug + Clone + Default,
    Weight: Debug + Copy + PartialOrd,
{
    fn with_node_capacity(capacity: usize) -> Self {
        Self::new(capacity)
    }
}

impl<Key, Data, EdgeMeta, Weight> AdjacencyMatrix<Key, Data, EdgeMeta, Weight>
where
    Key: Debug + Clone + Eq + Hash + Default,
    Data: Debug + Clone + Default,
    EdgeMeta: Debug + Clone + Default,
    Weight: Debug + Copy + PartialOrd,
{
    pub fn new(n: usize) -> Self {
        Self {
            n,
            nodes: NodeInterner::new(),
            edges: Vec::new(),
            data: vec![None; n * n],
        }
    }

    #[inline]
    fn idx(&self, r: usize, c: usize) -> usize {
        r * self.n + c
    }

    pub fn from_graphdef(def: GraphDefinition<Key, Data, EdgeMeta, Weight>) -> Self {
        let (records, index) = def.nodes.into_parts();
        let mut nodes = NodeInterner::new();
        nodes.records = records;
        nodes.index = index;
        let n = nodes.len();
        let mut mat = AdjacencyMatrix {
            n,
            nodes,
            edges: Vec::new(),
            data: vec![None; n * n],
        };
        for er in def.edges.into_iter() {
            let eid = EdgeId(mat.edges.len());
            let i = mat.idx(er.from.0, er.to.0);
            mat.edges.push(er);
            mat.data[i] = Some(eid);
        }
        mat
    }

    pub fn add_edge(
        &mut self,
        from: NodeId,
        to: NodeId,
        meta: EdgeMeta,
        weight: Option<Weight>,
    ) -> EdgeId {
        let eid = EdgeId(self.edges.len());
        self.edges
            .push(EdgeRecord::new(from, to, meta.clone(), weight));
        let i = self.idx(from.0, to.0);
        if self.data.len() <= i {
            let newn = self.nodes.len();
            self.data.resize(newn * newn, None);
            self.n = newn;
        }
        self.data[i] = Some(eid);
        eid
    }

    pub fn get_edge_id(&self, from: NodeId, to: NodeId) -> Option<EdgeId> {
        self.data[self.idx(from.0, to.0)]
    }

    pub fn row(&self, u: NodeId) -> &[Option<EdgeId>] {
        let start = u.0 * self.n;
        &self.data[start..start + self.n]
    }

    pub fn to_graph_def(&self) -> GraphDefinition<Key, Data, EdgeMeta, Weight> {
        let (records, index) = self.nodes.clone().into_parts();
        GraphDefinition {
            nodes: NodeInterner { records, index },
            edges: self.edges.clone(),
        }
    }
}

impl<Key, Data, EdgeMeta, Weight> From<GraphDefinition<Key, Data, EdgeMeta, Weight>>
    for AdjacencyMatrix<Key, Data, EdgeMeta, Weight>
where
    Key: Debug + Clone + Eq + Hash + Default,
    Data: Debug + Clone + Default,
    EdgeMeta: Debug + Clone + Default,
    Weight: Debug + Copy + PartialOrd,
{
    fn from(def: GraphDefinition<Key, Data, EdgeMeta, Weight>) -> Self {
        AdjacencyMatrix::from_graphdef(def)
    }
}

impl<Key, Data, EdgeMeta, Weight> GraphBase for AdjacencyMatrix<Key, Data, EdgeMeta, Weight>
where
    Key: Debug + Clone + Eq + Hash + Default,
    Data: Debug + Clone + Default,
    EdgeMeta: Debug + Clone + Default,
    Weight: Debug + Copy + PartialOrd,
{
    type Key = Key;
    type Data = Data;
    type EdgeMeta = EdgeMeta;
    type Weight = Weight;

    fn order(&self) -> usize {
        self.n
    }
    fn size(&self) -> usize {
        self.edges.len()
    }

    fn node_id(&self, key: &Self::Key) -> Option<NodeId> {
        self.nodes.get_id(key)
    }
    fn node_ids(&self) -> Box<dyn Iterator<Item = NodeId> + '_> {
        Box::new((0..self.n).map(NodeId))
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
        let i = self.idx(from.0, to.0);
        if let Some(eid) = self.data[i] {
            Box::new(std::iter::once(eid))
        } else {
            Box::new(std::iter::empty())
        }
    }

    fn neighborhood(&self, v: NodeId) -> Box<dyn Iterator<Item = NodeId> + '_> {
        if v.0 >= self.n {
            return Box::new(std::iter::empty());
        }

        let mut neighbors = IndexSet::new();
        for u in 0..self.n {
            let i_from = self.idx(v.0, u);
            if let Some(eid) = self.data[i_from] {
                neighbors.insert(self.edges[eid.0].to);
            }
        }
        for u in 0..self.n {
            let i_to = self.idx(u, v.0);
            if let Some(eid) = self.data[i_to] {
                neighbors.insert(self.edges[eid.0].from);
            }
        }

        Box::new(neighbors.into_iter())
    }

    fn predecessors(&self, v: NodeId) -> Box<dyn Iterator<Item = NodeId> + '_> {
        if v.0 >= self.n {
            return Box::new(std::iter::empty());
        }

        let predecessors = (0..self.n).filter_map(move |u| {
            let i = self.idx(u, v.0);
            if let Some(eid) = self.data[i] {
                Some(self.edges[eid.0].from)
            } else {
                None
            }
        });
        Box::new(predecessors)
    }
    fn successors(&self, v: NodeId) -> Box<dyn Iterator<Item = NodeId> + '_> {
        if v.0 >= self.n {
            return Box::new(std::iter::empty());
        }

        let successors = (0..self.n).filter_map(move |u| {
            let i = self.idx(v.0, u);
            if let Some(eid) = self.data[i] {
                Some(self.edges[eid.0].to)
            } else {
                None
            }
        });
        Box::new(successors)
    }
}

impl<Key, Data, EdgeMeta, Weight> EdgeWeights for AdjacencyMatrix<Key, Data, EdgeMeta, Weight>
where
    Key: Debug + Clone + Eq + Hash,
    Data: Debug + Clone,
    EdgeMeta: Debug + Clone,
    Weight: Debug + Copy + PartialOrd,
{
    type W = Weight;
    fn weight_of(&self, e: EdgeId) -> Option<Self::W> {
        self.edges[e.0].weight
    }
}

impl<Key, Data, EdgeMeta, Weight> MutableStorage for AdjacencyMatrix<Key, Data, EdgeMeta, Weight>
where
    Key: Debug + Clone + Eq + Hash + Default,
    Data: Debug + Clone + Default,
    EdgeMeta: Debug + Clone + Default,
    Weight: Debug + Copy + PartialOrd,
{
    fn add_node(&mut self, key: Self::Key, data: Self::Data) -> NodeId {
        let id = self.nodes.intern(key, data);
        let n_new = self.nodes.len();
        if n_new > self.n {
            self.data.resize(n_new * n_new, None);
            self.n = n_new;
        }
        id
    }

    fn add_edge_by_id(
        &mut self,
        from: NodeId,
        to: NodeId,
        meta: Self::EdgeMeta,
        weight: Option<Self::Weight>,
    ) -> EdgeId {
        let eid = EdgeId(self.edges.len());
        if self.n <= from.0 || self.n <= to.0 {
            let newn = self.nodes.len();
            self.data.resize(newn * newn, None);
            self.n = newn;
        }
        self.edges.push(EdgeRecord::new(from, to, meta, weight));
        let idx = self.idx(from.0, to.0);
        self.data[idx] = Some(eid);
        eid
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
        for i in 0..self.data.len() {
            self.data[i] = None;
        }
    }
}

impl<Key, Data, EdgeMeta, Weight, Target> StorageConvert<Target>
    for AdjacencyMatrix<Key, Data, EdgeMeta, Weight>
where
    Target: From<GraphDefinition<Key, Data, EdgeMeta, Weight>>,
    Key: Debug + Clone + Eq + Hash,
    Data: Debug + Clone,
    EdgeMeta: Debug + Clone,
    Weight: Debug + Copy + PartialOrd,
{
    fn convert(&self) -> Target {
        let mut def = GraphDefinition::new();
        for rec in self.nodes.records.iter() {
            def.nodes.intern(rec.key.clone(), rec.data.clone());
        }
        for er in self.edges.iter() {
            def.add_edge_by_id(er.from, er.to, er.meta.clone(), er.weight);
        }
        Target::from(def)
    }
}
