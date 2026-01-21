//! DirectedGraph and UndirectedGraph wrappers. They wrap a storage representation S.
//! They expose Graph trait to end users and delegate mutations to the underlying storage.
//! They also carry marker types (Simple / Pseudo / Multi) as type-level graph kind parameters
//! that select different behaviors at compile time.

use crate::core::{EdgeId, NodeId};
use crate::traits::*;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

pub trait Graph:
    GraphBase<
        Key = <Self::Storage as GraphBase>::Key,
        Data = <Self::Storage as GraphBase>::Data,
        EdgeMeta = <Self::Storage as GraphBase>::EdgeMeta,
        Weight = <Self::Storage as GraphBase>::Weight,
    >
where
    <Self::Storage as GraphBase>::Key: Eq + Hash,
{
    type Storage: StorageRepresentation;

    fn storage(&self) -> &Self::Storage;
    fn storage_mut(&mut self) -> &mut Self::Storage;
}

// Zero-sized marker types for graph kinds
#[derive(Clone, Copy, Debug)]
pub struct Simple;
#[derive(Clone, Copy, Debug)]
pub struct Pseudo;
#[derive(Clone, Copy, Debug)]
pub struct Multi;

impl GraphKindMarker for Simple {}
impl SimpleGraphKind for Simple {}
impl GraphKindMarker for Pseudo {}
impl PseudoGraphKind for Pseudo {}
impl GraphKindMarker for Multi {}
impl MultiGraphKind for Multi {}

#[derive(Clone)]
pub struct DirectedGraph<S, GK = Simple, K = String, D = (), E = (), W = ()>
where
    S: StorageRepresentation<Key = K, Data = D, EdgeMeta = E, Weight = W>,
    GK: GraphKindMarker,
    K: Clone + Eq + std::hash::Hash,
{
    pub storage: S,
    pub kind: PhantomData<GK>,
    _k: PhantomData<K>,
    _d: PhantomData<D>,
    _e: PhantomData<E>,
    _w: PhantomData<W>,
}

impl<S, GK, Key, Data, EdgeMeta, Weight> DirectedGraph<S, GK, Key, Data, EdgeMeta, Weight>
where
    S: StorageRepresentation<Key = Key, Data = Data, EdgeMeta = EdgeMeta, Weight = Weight>,
    GK: GraphKindMarker,
    Key: Clone + Eq + std::hash::Hash,
{
    pub fn converted_storage<SourceS>(
        other: &DirectedGraph<SourceS, GK, Key, Data, EdgeMeta, Weight>,
    ) -> DirectedGraph<S, GK, Key, Data, EdgeMeta, Weight>
    where
        SourceS: StorageConvert<S>
            + StorageRepresentation<Key = Key, Data = Data, EdgeMeta = EdgeMeta, Weight = Weight>,
    {
        let new = other.convert_storage();
        DirectedGraph::new(new)
    }

    pub fn new(storage: S) -> Self {
        Self {
            storage,
            kind: PhantomData,
            _k: PhantomData,
            _d: PhantomData,
            _e: PhantomData,
            _w: PhantomData,
        }
    }

    /// Convert storage representation to another storage type.
    pub fn convert_storage<TargetS>(&self) -> TargetS
    where
        S: StorageConvert<TargetS>,
    {
        self.storage.convert()
    }

    /// Convert into DirectedGraph of another storage (explicit via into_storage)
    pub fn into_storage<TargetS>(self) -> DirectedGraph<TargetS, GK, Key, Data, EdgeMeta, Weight>
    where
        S: StorageConvert<TargetS>,
        TargetS:
            StorageRepresentation<Key = Key, Data = Data, EdgeMeta = EdgeMeta, Weight = Weight>,
    {
        let new = self.storage.convert();
        DirectedGraph::new(new)
    }
}

/// Implement Graph trait for DirectedGraph
impl<S, GK, K, D, E, W> Graph for DirectedGraph<S, GK, K, D, E, W>
where
    S: StorageRepresentation<Key = K, Data = D, EdgeMeta = E, Weight = W>,
    GK: GraphKindMarker,
    K: Debug + Clone + Eq + Hash,
    D: Debug + Clone,
    E: Debug + Clone,
    W: Debug + Copy + PartialOrd,
{
    type Storage = S;
    fn storage(&self) -> &Self::Storage {
        &self.storage
    }
    fn storage_mut(&mut self) -> &mut Self::Storage {
        &mut self.storage
    }
}

/// Implement GraphBase by delegating to storage
impl<S, GK, K, D, E, W> GraphBase for DirectedGraph<S, GK, K, D, E, W>
where
    S: GraphBase<Key = K, Data = D, EdgeMeta = E, Weight = W>
        + StorageRepresentation<Key = K, Data = D, EdgeMeta = E, Weight = W>,
    GK: GraphKindMarker,
    K: Debug + Clone + Eq + Hash,
    D: Clone + Debug,
    E: Clone + Debug,
    W: Debug + Copy + PartialOrd,
{
    type Key = K;
    type Data = D;
    type EdgeMeta = E;
    type Weight = W;

    fn order(&self) -> usize {
        self.storage.order()
    }
    fn size(&self) -> usize {
        self.storage.size()
    }

    fn node_id(&self, key: &Self::Key) -> Option<NodeId> {
        self.storage.node_id(key)
    }
    fn node_ids(&self) -> Box<dyn Iterator<Item = NodeId> + '_> {
        self.storage.node_ids()
    }
    fn node_key(&self, id: NodeId) -> &Self::Key {
        self.storage.node_key(id)
    }
    fn node_data(&self, id: NodeId) -> &Self::Data {
        self.storage.node_data(id)
    }

    fn edge_ids(&self) -> Box<dyn Iterator<Item = EdgeId> + '_> {
        self.storage.edge_ids()
    }
    fn endpoints(&self, e: EdgeId) -> (NodeId, NodeId) {
        self.storage.endpoints(e)
    }
    fn edge_meta(&self, e: EdgeId) -> &Self::EdgeMeta {
        self.storage.edge_meta(e)
    }
    fn edges_between(&self, from: NodeId, to: NodeId) -> Box<dyn Iterator<Item = EdgeId> + '_> {
        self.storage.edges_between(from, to)
    }

    fn neighborhood(&self, v: NodeId) -> Box<dyn Iterator<Item = NodeId> + '_> {
        self.storage.neighborhood(v)
    }

    fn successors(&self, v: NodeId) -> Box<dyn Iterator<Item = NodeId> + '_> {
        self.storage.successors(v)
    }
    fn predecessors(&self, v: NodeId) -> Box<dyn Iterator<Item = NodeId> + '_> {
        self.storage.predecessors(v)
    }
}

/// Provide EdgeWeights delegating to storage; only available when W is NotUnit and storage supports weights
impl<S, GK, K, D, E, W> EdgeWeights for DirectedGraph<S, GK, K, D, E, W>
where
    S: EdgeWeights<W = W>
        + GraphBase<Key = K, Data = D, EdgeMeta = E, Weight = W>
        + StorageRepresentation<Key = K, Data = D, EdgeMeta = E, Weight = W>,
    GK: GraphKindMarker,
    K: Debug + Clone + Eq + Hash,
    D: Clone + Debug,
    E: Clone + Debug,
    W: Debug + Copy + PartialOrd,
{
    type W = W;
    fn weight_of(&self, e: EdgeId) -> Option<Self::W> {
        self.storage.weight_of(e)
    }
}

/// === Mutating behavior for DirectedGraph depending on GraphKind ===
/// We provide different impl blocks conditioned on GK marker trait:
/// - For Simple (default) => disallow self-loops and parallel edges
/// - For Pseudo => allow both self-loops and parallel edges
/// - For Multi => allow parallel edges, disallow self-loops
///
/// Storage must implement MutableStorage. Wrapper methods return Result to report constraint violations.

/// Generic helper: scan for any existing edge from->to
fn has_edge_between<S>(storage: &S, from: NodeId, to: NodeId) -> bool
where
    S: GraphBase,
{
    for e in storage.edge_ids() {
        let (f, t) = storage.endpoints(e);
        if f == from && t == to {
            return true;
        }
    }
    false
}

/// Impl for Simple graphs (no self-loops, no parallel edges)
impl<S, K, D, E, W> DirectedGraph<S, Simple, K, D, E, W>
where
    S: MutableStorage<Key = K, Data = D, EdgeMeta = E, Weight = W>
        + GraphBase<Key = K, Data = D, EdgeMeta = E, Weight = W>
        + StorageRepresentation<Key = K, Data = D, EdgeMeta = E, Weight = W>,
    K: Debug + Clone + Eq + Hash + Default,
    D: Debug + Clone + Default,
    E: Debug + Clone + Default,
    W: Debug + Copy + PartialOrd,
{
    /// checked add that accepts Option<weight>
    pub fn add_arc_checked(
        &mut self,
        from: NodeId,
        to: NodeId,
        meta: E,
        weight: Option<W>,
    ) -> Result<EdgeId, String> {
        if from == to {
            return Err("Simple graph: self-loops are not allowed".to_string());
        }
        if has_edge_between(&self.storage, from, to) {
            return Err("Simple graph: parallel edges are not allowed".to_string());
        }
        Ok(self.storage.add_edge_by_id(from, to, meta, weight))
    }

    /// convenience API when weight type is unit: no weight parameter
    pub fn add_arc(&mut self, from: NodeId, to: NodeId, meta: E) -> Result<EdgeId, String>
    where
        W: IsUnit,
    {
        self.add_arc_checked(from, to, meta, None)
    }

    /// convenience API when weight type is non-unit
    pub fn add_arc_with_weight(
        &mut self,
        from: NodeId,
        to: NodeId,
        meta: E,
        weight: W,
    ) -> Result<EdgeId, String>
    where
        W: NotUnit,
    {
        self.add_arc_checked(from, to, meta, Some(weight))
    }

    pub fn add_arc_by_key_checked(
        &mut self,
        from_key: K,
        to_key: K,
        from_data: D,
        to_data: D,
        meta: E,
        weight: Option<W>,
    ) -> Result<EdgeId, String> {
        let from = self.storage.add_node(from_key, from_data);
        let to = self.storage.add_node(to_key, to_data);
        self.add_arc_checked(from, to, meta, weight)
    }
}

/// Impl for Pseudo graphs (allow self-loops and parallel edges)
impl<S, K, D, E, W> DirectedGraph<S, Pseudo, K, D, E, W>
where
    S: MutableStorage<Key = K, Data = D, EdgeMeta = E, Weight = W>
        + GraphBase<Key = K, Data = D, EdgeMeta = E, Weight = W>
        + StorageRepresentation<Key = K, Data = D, EdgeMeta = E, Weight = W>,
    K: Debug + Clone + Eq + Hash + Default,
    D: Debug + Clone + Default,
    E: Debug + Clone + Default,
    W: Debug + Copy + PartialOrd,
{
    /// In pseudographs, we allow both self-loops and parallel edges; just delegate
    pub fn add_arc_checked(
        &mut self,
        from: NodeId,
        to: NodeId,
        meta: E,
        weight: Option<W>,
    ) -> Result<EdgeId, String> {
        Ok(self.storage.add_edge_by_id(from, to, meta, weight))
    }

    pub fn add_arc(&mut self, from: NodeId, to: NodeId, meta: E) -> Result<EdgeId, String>
    where
        W: IsUnit,
    {
        self.add_arc_checked(from, to, meta, None)
    }

    pub fn add_arc_with_weight(
        &mut self,
        from: NodeId,
        to: NodeId,
        meta: E,
        weight: W,
    ) -> Result<EdgeId, String>
    where
        W: NotUnit,
    {
        self.add_arc_checked(from, to, meta, Some(weight))
    }

    pub fn add_arc_by_key_checked(
        &mut self,
        from_key: K,
        to_key: K,
        from_data: D,
        to_data: D,
        meta: E,
        weight: Option<W>,
    ) -> Result<EdgeId, String> {
        let from = self.storage.add_node(from_key, from_data);
        let to = self.storage.add_node(to_key, to_data);
        self.add_arc_checked(from, to, meta, weight)
    }
}

/// Impl for Multi graphs (allow parallel edges, disallow self-loops)
impl<S, K, D, E, W> DirectedGraph<S, Multi, K, D, E, W>
where
    S: MutableStorage<Key = K, Data = D, EdgeMeta = E, Weight = W>
        + GraphBase<Key = K, Data = D, EdgeMeta = E, Weight = W>
        + StorageRepresentation<Key = K, Data = D, EdgeMeta = E, Weight = W>,
    K: Debug + Clone + Eq + Hash + Default,
    D: Debug + Clone + Default,
    E: Debug + Clone + Default,
    W: Debug + Copy + PartialOrd,
{
    pub fn add_arc_checked(
        &mut self,
        from: NodeId,
        to: NodeId,
        meta: E,
        weight: Option<W>,
    ) -> Result<EdgeId, String> {
        if from == to {
            return Err("Multi graph: self-loops are not allowed".to_string());
        }
        Ok(self.storage.add_edge_by_id(from, to, meta, weight))
    }

    pub fn add_arc(&mut self, from: NodeId, to: NodeId, meta: E) -> Result<EdgeId, String>
    where
        W: IsUnit,
    {
        self.add_arc_checked(from, to, meta, None)
    }

    pub fn add_arc_with_weight(
        &mut self,
        from: NodeId,
        to: NodeId,
        meta: E,
        weight: W,
    ) -> Result<EdgeId, String>
    where
        W: NotUnit,
    {
        self.add_arc_checked(from, to, meta, Some(weight))
    }

    pub fn add_arc_by_key_checked(
        &mut self,
        from_key: K,
        to_key: K,
        from_data: D,
        to_data: D,
        meta: E,
        weight: Option<W>,
    ) -> Result<EdgeId, String> {
        let from = self.storage.add_node(from_key, from_data);
        let to = self.storage.add_node(to_key, to_data);
        self.add_arc_checked(from, to, meta, weight)
    }
}

impl<S, K> DirectedGraph<S, Simple, K, (), (), ()>
where
    S: MutableStorage<Key = K, Data = (), EdgeMeta = (), Weight = ()>
        + GraphBase<Key = K, Data = (), EdgeMeta = (), Weight = ()>
        + StorageRepresentation<Key = K, Data = (), EdgeMeta = (), Weight = ()>,
    K: Debug + Clone + Eq + Hash + Default,
{
    pub fn from_isolated_nodes_and_edges<UK, NI, EI>(nodes_iter: NI, edges_iter: EI) -> Self
    where
        UK: Into<K>,
        NI: IntoIterator<Item = UK>,
        EI: IntoIterator<Item = (UK, UK)>,
    {
        let nodes = Vec::from_iter(nodes_iter);

        let mut storage = S::with_node_capacity(nodes.len());
        for nk in nodes {
            storage.add_node(nk.into(), ());
        }

        let mut graph = Self::new(storage);

        for (from_key, to_key) in edges_iter {
            graph
                .add_arc_by_key_checked(from_key.into(), to_key.into(), (), (), (), Some(()))
                .unwrap();
        }

        graph
    }

    pub fn from_edges<UK, EI>(edges_iter: EI) -> Self
    where
        UK: Into<K>,
        EI: IntoIterator<Item = (UK, UK)>,
    {
        let storage = S::with_node_capacity(0);
        let mut graph = Self::new(storage);

        for (from_key, to_key) in edges_iter {
            graph
                .add_arc_by_key_checked(from_key.into(), to_key.into(), (), (), (), Some(()))
                .unwrap();
        }

        graph
    }
}

impl<S, K, W> DirectedGraph<S, Simple, K, (), (), W>
where
    S: MutableStorage<Key = K, Data = (), EdgeMeta = (), Weight = W>
        + GraphBase<Key = K, Data = (), EdgeMeta = (), Weight = W>
        + StorageRepresentation<Key = K, Data = (), EdgeMeta = (), Weight = W>,
    K: Debug + Clone + Eq + Hash + Default,
    W: Debug + Copy + PartialOrd + NotUnit,
{
    pub fn from_isolated_nodes_and_edges<UK, NI, EI>(nodes_iter: NI, edges_iter: EI) -> Self
    where
        UK: Into<K>,
        NI: IntoIterator<Item = UK>,
        EI: IntoIterator<Item = (UK, UK, W)>,
    {
        let nodes = Vec::from_iter(nodes_iter);

        let mut storage = S::with_node_capacity(nodes.len());
        for nk in nodes {
            storage.add_node(nk.into(), ());
        }

        let mut graph = Self::new(storage);

        for (from_key, to_key, weight) in edges_iter {
            graph
                .add_arc_by_key_checked(from_key.into(), to_key.into(), (), (), (), Some(weight))
                .unwrap();
        }

        graph
    }

    pub fn from_edges<UK, EI>(edges_iter: EI) -> Self
    where
        UK: Into<K>,
        EI: IntoIterator<Item = (UK, UK, W)>,
    {
        let storage = S::with_node_capacity(0);
        let mut graph = Self::new(storage);

        for (from_key, to_key, weight) in edges_iter {
            graph
                .add_arc_by_key_checked(from_key.into(), to_key.into(), (), (), (), Some(weight))
                .unwrap();
        }

        graph
    }
}

/// UNDIRECTED WRAPPER
#[derive(Clone)]
pub struct UndirectedGraph<S, GK = Simple, K = String, D = (), E = (), W = ()>
where
    S: StorageRepresentation<Key = K, Data = D, EdgeMeta = E, Weight = W>,
    GK: GraphKindMarker,
    K: Clone + Eq + std::hash::Hash,
{
    pub storage: S,
    pub kind: PhantomData<GK>,
    _k: PhantomData<K>,
    _d: PhantomData<D>,
    _e: PhantomData<E>,
    _w: PhantomData<W>,
}

impl<S, GK, K, D, E, W> UndirectedGraph<S, GK, K, D, E, W>
where
    S: StorageRepresentation<Key = K, Data = D, EdgeMeta = E, Weight = W>,
    GK: GraphKindMarker,
    K: Clone + Eq + std::hash::Hash,
{
    pub fn new(storage: S) -> Self {
        Self {
            storage,
            kind: PhantomData,
            _k: PhantomData,
            _d: PhantomData,
            _e: PhantomData,
            _w: PhantomData,
        }
    }

    /// Convert storage similarly
    pub fn into_storage<TargetS>(self) -> UndirectedGraph<TargetS, GK, K, D, E, W>
    where
        S: StorageConvert<TargetS>,
        TargetS: StorageRepresentation<Key = K, Data = D, EdgeMeta = E, Weight = W>,
    {
        let new = self.storage.convert();
        UndirectedGraph::new(new)
    }

    /// Convert undirected to directed explicitly (user must request)
    pub fn into_directed<TargetS>(self) -> DirectedGraph<TargetS, GK, K, D, E, W>
    where
        S: StorageConvert<TargetS>,
        TargetS: StorageRepresentation<Key = K, Data = D, EdgeMeta = E, Weight = W>,
    {
        let new = self.storage.convert();
        DirectedGraph::new(new)
    }
}

impl<S, GK, K, D, E, W> Graph for UndirectedGraph<S, GK, K, D, E, W>
where
    S: StorageRepresentation<Key = K, Data = D, EdgeMeta = E, Weight = W>,
    GK: GraphKindMarker,
    K: Debug + Clone + Eq + Hash,
    D: Debug + Clone,
    E: Debug + Clone,
    W: Debug + Copy + PartialOrd,
{
    type Storage = S;
    fn storage(&self) -> &Self::Storage {
        &self.storage
    }
    fn storage_mut(&mut self) -> &mut Self::Storage {
        &mut self.storage
    }
}

impl<S, GK, K, D, E, W> GraphBase for UndirectedGraph<S, GK, K, D, E, W>
where
    S: GraphBase<Key = K, Data = D, EdgeMeta = E, Weight = W>
        + StorageRepresentation<Key = K, Data = D, EdgeMeta = E, Weight = W>,
    GK: GraphKindMarker,
    K: Clone + Eq + std::hash::Hash + Debug,
    D: Clone + Debug,
    E: Clone + Debug,
    W: Copy + PartialOrd + Debug,
{
    type Key = K;
    type Data = D;
    type EdgeMeta = E;
    type Weight = W;

    fn order(&self) -> usize {
        self.storage.order()
    }
    fn size(&self) -> usize {
        self.storage.size()
    }

    fn node_id(&self, key: &Self::Key) -> Option<NodeId> {
        self.storage.node_id(key)
    }
    fn node_ids(&self) -> Box<dyn Iterator<Item = NodeId> + '_> {
        self.storage.node_ids()
    }
    fn node_key(&self, id: NodeId) -> &Self::Key {
        self.storage.node_key(id)
    }
    fn node_data(&self, id: NodeId) -> &Self::Data {
        self.storage.node_data(id)
    }

    fn edge_ids(&self) -> Box<dyn Iterator<Item = EdgeId> + '_> {
        self.storage.edge_ids()
    }
    fn endpoints(&self, e: EdgeId) -> (NodeId, NodeId) {
        self.storage.endpoints(e)
    }
    fn edge_meta(&self, e: EdgeId) -> &Self::EdgeMeta {
        self.storage.edge_meta(e)
    }
    fn edges_between(&self, from: NodeId, to: NodeId) -> Box<dyn Iterator<Item = EdgeId> + '_> {
        self.storage.edges_between(from, to)
    }

    fn neighborhood(&self, v: NodeId) -> Box<dyn Iterator<Item = NodeId> + '_> {
        self.storage.neighborhood(v)
    }

    fn successors(&self, v: NodeId) -> Box<dyn Iterator<Item = NodeId> + '_> {
        self.storage.neighborhood(v)
    }
    fn predecessors(&self, v: NodeId) -> Box<dyn Iterator<Item = NodeId> + '_> {
        self.storage.neighborhood(v)
    }
}

/// Mutating operations for undirected graph add symmetric edges into the underlying storage.
/// We provide different behavior depending on GraphKind similarly to DirectedGraph:
/// - Simple: no self-loops, no parallel edges
/// - Pseudo: allow self-loops and parallel edges
/// - Multi: allow parallel edges, disallow self-loops

/// Simple undirected graph impl
impl<S, K, D, E, W> UndirectedGraph<S, Simple, K, D, E, W>
where
    S: MutableStorage
        + GraphBase<Key = K, Data = D, EdgeMeta = E, Weight = W>
        + StorageRepresentation<Key = K, Data = D, EdgeMeta = E, Weight = W>,
    K: Debug + Clone + Eq + Hash + Default,
    D: Debug + Clone + Default,
    E: Debug + Clone + Default,
    W: Debug + Copy + PartialOrd,
{
    pub fn add_edge_checked(
        &mut self,
        a: NodeId,
        b: NodeId,
        meta: E,
        weight: Option<W>,
    ) -> Result<(EdgeId, EdgeId), String> {
        if a == b {
            return Err("Simple undirected graph: self-loops not allowed".to_string());
        }
        // scan for existing a->b or b->a edge
        for e in self.storage.edge_ids() {
            let (f, t) = self.storage.endpoints(e);
            if (f == a && t == b) || (f == b && t == a) {
                return Err("Simple undirected graph: parallel edges not allowed".to_string());
            }
        }
        let e1 = self.storage.add_edge_by_id(a, b, meta.clone(), weight);
        let e2 = self.storage.add_edge_by_id(b, a, meta, weight);
        Ok((e1, e2))
    }

    pub fn add_edge(&mut self, a: NodeId, b: NodeId, meta: E) -> Result<(EdgeId, EdgeId), String>
    where
        W: IsUnit,
    {
        self.add_edge_checked(a, b, meta, None)
    }

    pub fn add_edge_with_weight(
        &mut self,
        a: NodeId,
        b: NodeId,
        meta: E,
        weight: W,
    ) -> Result<(EdgeId, EdgeId), String>
    where
        W: NotUnit,
    {
        self.add_edge_checked(a, b, meta, Some(weight))
    }

    pub fn add_edge_by_key_checked(
        &mut self,
        a_key: K,
        b_key: K,
        a_data: D,
        b_data: D,
        meta: E,
        weight: Option<W>,
    ) -> Result<(EdgeId, EdgeId), String> {
        let a = self.storage.add_node(a_key, a_data);
        let b = self.storage.add_node(b_key, b_data);
        self.add_edge_checked(a, b, meta, weight)
    }
}

/// Pseudo undirected graph impl (allow self-loops and parallel edges)
impl<S, K, D, E, W> UndirectedGraph<S, Pseudo, K, D, E, W>
where
    S: MutableStorage
        + GraphBase<Key = K, Data = D, EdgeMeta = E, Weight = W>
        + StorageRepresentation<Key = K, Data = D, EdgeMeta = E, Weight = W>,
    K: Debug + Clone + Eq + Hash + Default,
    D: Debug + Clone + Default,
    E: Debug + Clone + Default,
    W: Debug + Copy + PartialOrd,
{
    pub fn add_edge_checked(
        &mut self,
        a: NodeId,
        b: NodeId,
        meta: E,
        weight: Option<W>,
    ) -> Result<(EdgeId, EdgeId), String> {
        // allow everything: self-loops and parallel edges permitted
        let e1 = self.storage.add_edge_by_id(a, b, meta.clone(), weight);
        let e2 = self.storage.add_edge_by_id(b, a, meta, weight);
        Ok((e1, e2))
    }

    pub fn add_edge(&mut self, a: NodeId, b: NodeId, meta: E) -> Result<(EdgeId, EdgeId), String>
    where
        W: IsUnit,
    {
        self.add_edge_checked(a, b, meta, None)
    }

    pub fn add_edge_with_weight(
        &mut self,
        a: NodeId,
        b: NodeId,
        meta: E,
        weight: W,
    ) -> Result<(EdgeId, EdgeId), String>
    where
        W: NotUnit,
    {
        self.add_edge_checked(a, b, meta, Some(weight))
    }

    pub fn add_edge_by_key_checked(
        &mut self,
        a_key: K,
        b_key: K,
        a_data: D,
        b_data: D,
        meta: E,
        weight: Option<W>,
    ) -> Result<(EdgeId, EdgeId), String> {
        let a = self.storage.add_node(a_key, a_data);
        let b = self.storage.add_node(b_key, b_data);
        self.add_edge_checked(a, b, meta, weight)
    }
}

/// Multi undirected graph impl (allow parallel edges, disallow self-loops)
impl<S, K, D, E, W> UndirectedGraph<S, Multi, K, D, E, W>
where
    S: MutableStorage
        + GraphBase<Key = K, Data = D, EdgeMeta = E, Weight = W>
        + StorageRepresentation<Key = K, Data = D, EdgeMeta = E, Weight = W>,
    K: Debug + Clone + Eq + Hash + Default,
    D: Debug + Clone + Default,
    E: Debug + Clone + Default,
    W: Debug + Copy + PartialOrd,
{
    pub fn add_edge_checked(
        &mut self,
        a: NodeId,
        b: NodeId,
        meta: E,
        weight: Option<W>,
    ) -> Result<(EdgeId, EdgeId), String> {
        if a == b {
            return Err("Multi undirected graph: self-loops not allowed".to_string());
        }
        let e1 = self.storage.add_edge_by_id(a, b, meta.clone(), weight);
        let e2 = self.storage.add_edge_by_id(b, a, meta, weight);
        Ok((e1, e2))
    }

    pub fn add_edge(&mut self, a: NodeId, b: NodeId, meta: E) -> Result<(EdgeId, EdgeId), String>
    where
        W: IsUnit,
    {
        self.add_edge_checked(a, b, meta, None)
    }

    pub fn add_edge_with_weight(
        &mut self,
        a: NodeId,
        b: NodeId,
        meta: E,
        weight: W,
    ) -> Result<(EdgeId, EdgeId), String>
    where
        W: NotUnit,
    {
        self.add_edge_checked(a, b, meta, Some(weight))
    }

    pub fn add_edge_by_key_checked(
        &mut self,
        a_key: K,
        b_key: K,
        a_data: D,
        b_data: D,
        meta: E,
        weight: Option<W>,
    ) -> Result<(EdgeId, EdgeId), String> {
        let a = self.storage.add_node(a_key, a_data);
        let b = self.storage.add_node(b_key, b_data);
        self.add_edge_checked(a, b, meta, weight)
    }
}

impl<S, K> UndirectedGraph<S, Simple, K, (), (), ()>
where
    S: MutableStorage<Key = K, Data = (), EdgeMeta = (), Weight = ()>
        + GraphBase<Key = K, Data = (), EdgeMeta = (), Weight = ()>
        + StorageRepresentation<Key = K, Data = (), EdgeMeta = (), Weight = ()>,
    K: Debug + Clone + Eq + Hash + Default,
{
    pub fn from_isolated_nodes_and_edges<UK, NI, EI>(nodes_iter: NI, edges_iter: EI) -> Self
    where
        UK: Into<K>,
        NI: IntoIterator<Item = UK>,
        EI: IntoIterator<Item = (UK, UK)>,
    {
        let nodes = Vec::from_iter(nodes_iter);

        let mut storage = S::with_node_capacity(nodes.len());
        for nk in nodes {
            storage.add_node(nk.into(), ());
        }

        let mut graph = Self::new(storage);

        for (from_key, to_key) in edges_iter {
            graph
                .add_edge_by_key_checked(from_key.into(), to_key.into(), (), (), (), Some(()))
                .unwrap();
        }

        graph
    }

    pub fn from_edges<UK, EI>(edges_iter: EI) -> Self
    where
        UK: Into<K>,
        EI: IntoIterator<Item = (UK, UK)>,
    {
        let storage = S::with_node_capacity(0);
        let mut graph = Self::new(storage);

        for (from_key, to_key) in edges_iter {
            graph
                .add_edge_by_key_checked(from_key.into(), to_key.into(), (), (), (), Some(()))
                .unwrap();
        }

        graph
    }
}

impl<S, K, W> UndirectedGraph<S, Simple, K, (), (), W>
where
    S: MutableStorage<Key = K, Data = (), EdgeMeta = (), Weight = W>
        + GraphBase<Key = K, Data = (), EdgeMeta = (), Weight = W>
        + StorageRepresentation<Key = K, Data = (), EdgeMeta = (), Weight = W>,
    K: Debug + Clone + Eq + Hash + Default,
    W: Debug + Copy + PartialOrd + NotUnit,
{
    pub fn from_isolated_nodes_and_edges<UK, NI, EI>(nodes_iter: NI, edges_iter: EI) -> Self
    where
        UK: Into<K>,
        NI: IntoIterator<Item = UK>,
        EI: IntoIterator<Item = (UK, UK, W)>,
    {
        let nodes = Vec::from_iter(nodes_iter);

        let mut storage = S::with_node_capacity(nodes.len());
        for nk in nodes {
            storage.add_node(nk.into(), ());
        }

        let mut graph = Self::new(storage);

        for (from_key, to_key, weight) in edges_iter {
            graph
                .add_edge_by_key_checked(from_key.into(), to_key.into(), (), (), (), Some(weight))
                .unwrap();
        }

        graph
    }

    pub fn from_edges<UK, EI>(edges_iter: EI) -> Self
    where
        UK: Into<K>,
        EI: IntoIterator<Item = (UK, UK, W)>,
    {
        let storage = S::with_node_capacity(0);
        let mut graph = Self::new(storage);

        for (from_key, to_key, weight) in edges_iter {
            graph
                .add_edge_by_key_checked(from_key.into(), to_key.into(), (), (), (), Some(weight))
                .unwrap();
        }

        graph
    }
}

impl<S, GK, K, D, E, W> EdgeWeights for UndirectedGraph<S, GK, K, D, E, W>
where
    S: EdgeWeights<W = W>
        + GraphBase<Key = K, Data = D, EdgeMeta = E, Weight = W>
        + StorageRepresentation<Key = K, Data = D, EdgeMeta = E, Weight = W>,
    GK: GraphKindMarker,
    K: Debug + Clone + Eq + Hash,
    D: Clone + Debug,
    E: Clone + Debug,
    W: Debug + Copy + PartialOrd,
{
    type W = W;
    fn weight_of(&self, e: EdgeId) -> Option<Self::W> {
        self.storage.weight_of(e)
    }
}

// /// Blanket impl: if A can convert to B, then DirectedGraph<A> -> DirectedGraph<B> via From (implicit)
// impl<A, B, GK, K, D, E, W> From<DirectedGraph<A, GK, K, D, E, W>>
//     for DirectedGraph<B, GK, K, D, E, W>
// where
//     A: StorageConvert<B> + StorageRepresentation<Key = K, Data = D, EdgeMeta = E, Weight = W>,
//     B: StorageRepresentation<Key = K, Data = D, EdgeMeta = E, Weight = W>,
//     GK: GraphKindMarker,
//     K: Clone + Eq + std::hash::Hash,
// {
//     fn from(src: DirectedGraph<A, GK, K, D, E, W>) -> Self {
//         let new_storage: B = src.storage.convert();
//         DirectedGraph::new(new_storage)
//     }
// }
