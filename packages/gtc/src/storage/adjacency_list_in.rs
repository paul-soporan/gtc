use indexmap::IndexSet;

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
pub struct AdjacencyListIn<Key = String, Data = (), EdgeMeta = (), Weight = ()>
where
    Key: Clone + Eq + Hash,
    Data: Clone,
    EdgeMeta: Clone,
    Weight: Copy + PartialOrd + Debug,
{
    pub nodes: NodeInterner<Key, Data>,
    pub edges: Vec<EdgeRecord<EdgeMeta, Weight>>,
    pub out_adj: Vec<Vec<EdgeId>>,
    pub in_adj: Vec<Vec<EdgeId>>,
}

impl<Key, Data, EdgeMeta, Weight> StorageRepresentation
    for AdjacencyListIn<Key, Data, EdgeMeta, Weight>
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
            in_adj: Vec::with_capacity(capacity),
        }
    }
}

impl<Key, Data, EdgeMeta, Weight> AdjacencyListIn<Key, Data, EdgeMeta, Weight>
where
    Key: Debug + Clone + Eq + Hash + Default,
    Data: Debug + Clone + Default,
    EdgeMeta: Debug + Clone + Default,
    Weight: Debug + Copy + PartialOrd,
{
    pub fn new() -> Self {
        Self {
            nodes: NodeInterner::new(),
            edges: Vec::new(),
            out_adj: Vec::new(),
            in_adj: Vec::new(),
        }
    }

    pub fn from_edge_list<NI, EI>(nodes_iter: NI, edges_iter: EI) -> Self
    where
        NI: IntoIterator<Item = (Key, Data)>,
        EI: IntoIterator<Item = (Key, Key, EdgeMeta, Option<Weight>)>,
    {
        let mut st = Self::new();
        for (k, d) in nodes_iter {
            st.nodes.intern(k, d);
        }
        for (a, b, meta, weight) in edges_iter {
            let from = st.nodes.intern(a, Default::default());
            let to = st.nodes.intern(b, Default::default());
            if st.out_adj.len() <= from.0 {
                st.out_adj.resize(from.0 + 1, Vec::new());
            }
            if st.in_adj.len() <= to.0 {
                st.in_adj.resize(to.0 + 1, Vec::new());
            }
            let eid = EdgeId(st.edges.len());
            st.out_adj[from.0].push(eid);
            st.in_adj[to.0].push(eid);
            st.edges.push(EdgeRecord::new(from, to, meta, weight));
        }
        st
    }

    pub fn from_graphdef(def: GraphDefinition<Key, Data, EdgeMeta, Weight>) -> Self {
        let (records, index) = def.nodes.into_parts();
        let mut nodes = NodeInterner::new();
        nodes.records = records;
        nodes.index = index;
        let n = nodes.len();
        let mut g = Self {
            nodes,
            edges: Vec::new(),
            out_adj: vec![Vec::new(); n],
            in_adj: vec![Vec::new(); n],
        };
        for er in def.edges.into_iter() {
            let eid = EdgeId(g.edges.len());
            let from = er.from.0;
            let to = er.to.0;
            if g.out_adj.len() <= from {
                g.out_adj.resize(from + 1, Vec::new());
            }
            if g.in_adj.len() <= to {
                g.in_adj.resize(to + 1, Vec::new());
            }
            g.out_adj[from].push(eid);
            g.in_adj[to].push(eid);
            g.edges.push(er);
        }
        g
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
    for AdjacencyListIn<Key, Data, EdgeMeta, Weight>
where
    Key: Debug + Clone + Eq + Hash + Default,
    Data: Debug + Clone + Default,
    EdgeMeta: Debug + Clone + Default,
    Weight: Debug + Copy + PartialOrd,
{
    fn from(def: GraphDefinition<Key, Data, EdgeMeta, Weight>) -> Self {
        Self::from_graphdef(def)
    }
}

impl<Key, Data, EdgeMeta, Weight> GraphBase for AdjacencyListIn<Key, Data, EdgeMeta, Weight>
where
    Key: Eq + std::hash::Hash + Clone + std::fmt::Debug,
    Data: Clone + std::fmt::Debug,
    EdgeMeta: Clone + std::fmt::Debug,
    Weight: Copy + PartialOrd + Debug,
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
        for &eid in &self.out_adj.get(from.0).cloned().unwrap_or_default() {
            if self.edges[eid.0].to == to {
                edge_ids.push(eid);
            }
        }
        Box::new(edge_ids.into_iter())
    }

    fn neighborhood(&self, v: NodeId) -> Box<dyn Iterator<Item = NodeId> + '_> {
        let mut set = IndexSet::new();
        if v.0 < self.out_adj.len() {
            for &eid in &self.out_adj[v.0] {
                set.insert(self.edges[eid.0].to);
            }
        }
        if v.0 < self.in_adj.len() {
            for &eid in &self.in_adj[v.0] {
                set.insert(self.edges[eid.0].from);
            }
        }
        Box::new(set.into_iter())
    }

    fn predecessors(&self, v: NodeId) -> Box<dyn Iterator<Item = NodeId> + '_> {
        if v.0 >= self.in_adj.len() {
            return Box::new(std::iter::empty());
        }
        let vec = self.in_adj[v.0]
            .iter()
            .map(move |&eid| self.edges[eid.0].from)
            .collect::<Vec<_>>();
        Box::new(vec.into_iter())
    }
    fn successors(&self, v: NodeId) -> Box<dyn Iterator<Item = NodeId> + '_> {
        if v.0 >= self.out_adj.len() {
            return Box::new(std::iter::empty());
        }
        let vec = self.out_adj[v.0]
            .iter()
            .map(move |&eid| self.edges[eid.0].to)
            .collect::<Vec<_>>();
        Box::new(vec.into_iter())
    }
}

impl<K, D, E, W> EdgeWeights for AdjacencyListIn<K, D, E, W>
where
    K: Eq + std::hash::Hash + Clone + std::fmt::Debug,
    D: Clone + std::fmt::Debug,
    E: Clone + std::fmt::Debug,
    W: Copy + PartialOrd + Debug,
{
    type W = W;
    fn weight_of(&self, e: EdgeId) -> Option<Self::W> {
        self.edges[e.0].weight
    }
}

impl<Key, Data, EdgeMeta, Weight> MutableStorage for AdjacencyListIn<Key, Data, EdgeMeta, Weight>
where
    Key: Debug + Clone + Eq + Hash + Default,
    Data: Debug + Clone + Default,
    EdgeMeta: Debug + Clone + Default,
    Weight: Debug + Copy + PartialOrd,
{
    fn add_node(&mut self, key: Self::Key, data: Self::Data) -> NodeId {
        let id = self.nodes.intern(key, data);
        if self.out_adj.len() <= id.0 {
            self.out_adj.resize(id.0 + 1, Vec::new());
        }
        if self.in_adj.len() <= id.0 {
            self.in_adj.resize(id.0 + 1, Vec::new());
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
        if self.in_adj.len() <= to.0 {
            self.in_adj.resize(to.0 + 1, Vec::new());
        }
        let eid = EdgeId(self.edges.len());
        self.out_adj[from.0].push(eid);
        self.in_adj[to.0].push(eid);
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
        for i in 0..self.in_adj.len() {
            self.in_adj[i].clear();
        }
    }
}

impl<K, D, E, W, Target> StorageConvert<Target> for AdjacencyListIn<K, D, E, W>
where
    Target: From<GraphDefinition<K, D, E, W>>,
    K: Debug + Eq + std::hash::Hash + Clone,
    D: Debug + Clone,
    E: Debug + Clone,
    W: Debug + Copy + PartialOrd,
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
