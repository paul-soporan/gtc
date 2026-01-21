//! AdjacencyList: out-edges only, implements GraphBase, Neighborhood (neighborhood == successors here), EdgeWeights,
//! StorageRepresentation, MutableStorage, and StorageConvert into other storage types via GraphDefinition.

use crate::core::{EdgeId, NodeId};
use crate::interner::NodeInterner;
use crate::storage::graph_definition::{EdgeRecord as GEdgeRecord, GraphDefinition};
use crate::traits::{
    EdgeWeights, GraphBase, MutableStorage, StorageConvert, StorageRepresentation,
};
use std::fmt::Debug;
use std::hash::Hash;

pub type EdgeRecord<E, W> = GEdgeRecord<E, W>;

#[derive(Clone)]
pub struct AdjacencyList<Key = String, Data = (), EdgeMeta = (), Weight = ()>
where
    Key: Debug + Clone + Eq + Hash,
    Data: Debug + Clone,
    EdgeMeta: Debug + Clone,
    Weight: Debug + Copy + PartialOrd,
{
    pub nodes: NodeInterner<Key, Data>,
    pub edges: Vec<EdgeRecord<EdgeMeta, Weight>>,
    pub out_adj: Vec<Vec<EdgeId>>,
}

impl<Key, Data, EdgeMeta, Weight> StorageRepresentation
    for AdjacencyList<Key, Data, EdgeMeta, Weight>
where
    Key: Debug + Clone + Eq + Hash,
    Data: Debug + Clone,
    EdgeMeta: Debug + Clone,
    Weight: Debug + Copy + PartialOrd,
{
    fn with_node_capacity(capacity: usize) -> Self {
        Self {
            nodes: NodeInterner::new(),
            edges: Vec::new(),
            out_adj: Vec::with_capacity(capacity),
        }
    }
}

impl<Key, Data, EdgeMeta, Weight> AdjacencyList<Key, Data, EdgeMeta, Weight>
where
    Key: Debug + Clone + Eq + Hash + Default,
    Data: Debug + Clone + Default,
    EdgeMeta: Debug + Clone + Default,
    Weight: Debug + Copy + PartialOrd + Default,
{
    pub fn new() -> Self {
        Self {
            nodes: NodeInterner::new(),
            edges: Vec::new(),
            out_adj: Vec::new(),
        }
    }

    pub fn from_edge_list<NI, EI>(nodes_iter: NI, edges_iter: EI) -> Self
    where
        NI: IntoIterator<Item = (Key, Data)>,
        EI: IntoIterator<Item = (Key, Key, EdgeMeta, Option<Weight>)>,
    {
        let mut interner = NodeInterner::new();
        for (k, d) in nodes_iter {
            interner.intern(k, d);
        }
        let n = interner.len();
        let out_adj = vec![Vec::new(); n];
        let edges = Vec::new();

        let mut al = Self {
            nodes: interner,
            edges,
            out_adj,
        };

        for (a, b, meta, weight) in edges_iter {
            let from = al.nodes.intern(a, Default::default());
            let to = al.nodes.intern(b, Default::default());
            if al.out_adj.len() <= from.0 {
                al.out_adj.resize(from.0 + 1, Vec::new());
            }
            let eid = EdgeId(al.edges.len());
            al.out_adj[from.0].push(eid);
            al.edges.push(EdgeRecord::new(from, to, meta, weight));
        }
        al
    }

    pub fn from_graphdef(def: GraphDefinition<Key, Data, EdgeMeta, Weight>) -> Self {
        let (node_records, index) = def.nodes.into_parts();
        let mut nodes = NodeInterner::new();
        nodes.records = node_records;
        nodes.index = index;

        let n = nodes.len();
        let mut al = Self {
            nodes,
            edges: Vec::new(),
            out_adj: vec![Vec::new(); n],
        };

        for er in def.edges.into_iter() {
            let eid = EdgeId(al.edges.len());
            let from = er.from.0;
            if al.out_adj.len() <= from {
                al.out_adj.resize(from + 1, Vec::new());
            }
            al.out_adj[from].push(eid);
            al.edges.push(er);
        }
        al
    }

    pub fn to_graph_def(&self) -> GraphDefinition<Key, Data, EdgeMeta, Weight> {
        let (records, index) = self.nodes.clone().into_parts();
        GraphDefinition {
            nodes: crate::interner::NodeInterner { records, index },
            edges: self.edges.clone(),
        }
    }
}

impl<Key, Data, EdgeMeta, Weight> From<GraphDefinition<Key, Data, EdgeMeta, Weight>>
    for AdjacencyList<Key, Data, EdgeMeta, Weight>
where
    Key: Debug + Clone + Eq + Hash + Default,
    Data: Debug + Clone + Default,
    EdgeMeta: Debug + Clone + Default,
    Weight: Debug + Copy + PartialOrd + Default,
{
    fn from(def: GraphDefinition<Key, Data, EdgeMeta, Weight>) -> Self {
        Self::from_graphdef(def)
    }
}

impl<Key, Data, EdgeMeta, Weight> GraphBase for AdjacencyList<Key, Data, EdgeMeta, Weight>
where
    Key: Debug + Clone + Eq + Hash,
    Data: Debug + Clone,
    EdgeMeta: Debug + Clone,
    Weight: Debug + Copy + PartialOrd,
{
    type Key = Key;
    type Data = Data;
    type EdgeMeta = EdgeMeta;
    type Weight = Weight;

    fn order(&self) -> usize {
        self.nodes.len()
    }
    fn size(&self) -> usize {
        self.edges.len()
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
        if v.0 >= self.nodes.len() {
            return Box::new(std::iter::empty());
        }

        let mut neighbors = Vec::new();
        for er in &self.edges {
            if er.from == v {
                neighbors.push(er.to);
            } else if er.to == v {
                neighbors.push(er.from);
            }
        }
        Box::new(neighbors.into_iter())
    }

    fn predecessors(&self, v: NodeId) -> Box<dyn Iterator<Item = NodeId> + '_> {
        if v.0 >= self.nodes.len() {
            return Box::new(std::iter::empty());
        }

        let mut predecessors = Vec::new();
        for er in &self.edges {
            if er.to == v {
                predecessors.push(er.from);
            }
        }
        Box::new(predecessors.into_iter())
    }
    fn successors(&self, v: NodeId) -> Box<dyn Iterator<Item = NodeId> + '_> {
        if v.0 >= self.nodes.len() {
            return Box::new(std::iter::empty());
        }

        let mut successors = Vec::new();
        if v.0 < self.out_adj.len() {
            for &eid in &self.out_adj[v.0] {
                successors.push(self.edges[eid.0].to);
            }
        }
        Box::new(successors.into_iter())
    }
}

impl<Key, Data, EdgeMeta, Weight> EdgeWeights for AdjacencyList<Key, Data, EdgeMeta, Weight>
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

impl<Key, Data, EdgeMeta, Weight> MutableStorage for AdjacencyList<Key, Data, EdgeMeta, Weight>
where
    Key: Debug + Clone + Eq + Hash + Default,
    Data: Debug + Clone + Default,
    EdgeMeta: Debug + Clone + Default,
    Weight: Debug + Copy + PartialOrd + Default,
{
    fn add_node(&mut self, key: Self::Key, data: Self::Data) -> NodeId {
        let id = self.nodes.intern(key, data);
        if self.out_adj.len() <= id.0 {
            self.out_adj.resize(id.0 + 1, Vec::new());
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
        if self.out_adj.len() <= from.0 {
            self.out_adj.resize(from.0 + 1, Vec::new());
        }
        let eid = EdgeId(self.edges.len());
        self.out_adj[from.0].push(eid);
        self.edges.push(EdgeRecord::new(from, to, meta, weight));
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
        for i in 0..self.out_adj.len() {
            self.out_adj[i].clear();
        }
    }
}

impl<Key, Data, EdgeMeta, Weight, Target> StorageConvert<Target>
    for AdjacencyList<Key, Data, EdgeMeta, Weight>
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
