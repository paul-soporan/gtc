use std::{
    collections::{HashMap, VecDeque},
    fmt::{Debug, Display},
    hash::Hash,
    ops::Add,
};

use crate::{
    DirectedGraph, EdgeId, GraphBase, GraphKindMarker, LatexDisplay, LatexVisualDisplay,
    MutableStorage, NodeId, StorageRepresentation, VisualEdge, VisualGraphData,
    generate_latex_graph,
};

#[derive(Clone, Debug)]
pub struct Flow {
    map: HashMap<(NodeId, NodeId), i32>,
}

impl Flow {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}

impl Add<&Flow> for Flow {
    type Output = Flow;

    fn add(self, other: &Flow) -> Flow {
        let mut result = self.map.clone();
        for (&(src, dst), &f) in &other.map {
            *result.entry((src, dst)).or_insert(0) += f;
        }
        Flow { map: result }
    }
}

#[derive(Clone)]
pub struct FlowNetwork<S, GK, K, D, E, W>
where
    S: StorageRepresentation<Key = K, Data = D, EdgeMeta = E, Weight = W> + Clone,
    GK: crate::traits::GraphKindMarker + Clone,
    K: Clone + Eq + std::hash::Hash,
{
    pub graph: DirectedGraph<S, GK, K, D, E, W>,
    pub capacity: Vec<u32>,
    pub source: NodeId,
    pub sink: NodeId,
    pub flow: Flow,
}

impl<S, GK, K, D, E, W> LatexVisualDisplay for FlowNetwork<S, GK, K, D, E, W>
where
    S: StorageRepresentation<Key = K, Data = D, EdgeMeta = E, Weight = W>
        + MutableStorage<Key = K, Data = D, EdgeMeta = E, Weight = W>
        + Clone,
    GK: GraphKindMarker + Clone,
    K: Debug + Clone + Eq + Hash + Display,
    D: Debug + Clone,
    E: Debug + Clone + Default,
    W: Debug + Copy + PartialOrd,
{
    fn to_latex_visual(&self) -> String {
        let mut network = self.clone();
        for ((from, to), value) in network.flow.map.iter() {
            if *value > 0 && network.graph.edges_between(*from, *to).next().is_none() {
                let c = network.capacity[network.graph.edges_between(*to, *from).next().unwrap().0]
                    as i32
                    + *network.flow.map.get(&(*to, *from)).unwrap_or(&0);

                if c > 0 {
                    network
                        .graph
                        .storage
                        .add_edge_by_id(*from, *to, E::default(), None);
                    network.capacity.push(c as u32);
                }
            }
        }

        let n = network.graph.order();
        let labels: Vec<String> = (0..n)
            .map(|i| network.graph.node_key(NodeId(i)).to_string())
            .collect();

        let mut edges = Vec::new();
        for eid in network.graph.storage.edge_ids() {
            let (u, v) = network.graph.endpoints(eid);
            let label = network
                .flow
                .map
                .get(&(u, v))
                .and_then(|f| {
                    if *f > 0 {
                        Some(format!("{}/{}", f, network.capacity[eid.0]))
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| format!("{}", network.capacity[eid.0]));
            edges.push(VisualEdge {
                u: u.0,
                v: v.0,
                label: Some(label),
            });
        }

        let data = VisualGraphData {
            labels,
            edges,
            is_directed: true,
        };

        generate_latex_graph(data)
    }
}

impl<S, GK, K, D, E, W> FlowNetwork<S, GK, K, D, E, W>
where
    S: StorageRepresentation<Key = K, Data = D, EdgeMeta = E, Weight = W> + Clone,
    GK: crate::traits::GraphKindMarker + Clone,
    K: Clone + Eq + std::hash::Hash,
{
    pub fn new(
        graph: DirectedGraph<S, GK, K, D, E, W>,
        capacity: Vec<u32>,
        source: NodeId,
        sink: NodeId,
    ) -> Self {
        Self {
            graph,
            capacity,
            source,
            sink,
            flow: Flow::new(),
        }
    }
}

impl<S, GK, K> FlowNetwork<S, GK, K, (), (), ()>
where
    S: StorageRepresentation<Key = K, Data = (), EdgeMeta = (), Weight = ()>
        + MutableStorage<Key = K, Data = (), EdgeMeta = (), Weight = ()>
        + Clone,
    GK: crate::traits::GraphKindMarker + Clone,
    K: Clone + Eq + std::hash::Hash,
{
    pub fn from_edges<UK>(edges: Vec<(UK, UK, i32, u32)>, source_key: UK, sink_key: UK) -> Self
    where
        UK: Into<K> + Clone,
    {
        let storage = S::with_node_capacity(edges.len() * 2);
        let mut graph = DirectedGraph::<S, GK, K, (), (), ()>::new(storage);
        let mut capacity: Vec<u32> = Vec::new();

        let mut flow_map: HashMap<(NodeId, NodeId), i32> = HashMap::new();

        for (from_key, to_key, flow, cap) in edges {
            let from_data = ();
            let to_data = ();
            let edge_meta = ();
            graph.storage.add_edge_by_key(
                from_key.clone().into(),
                to_key.clone().into(),
                from_data,
                to_data,
                edge_meta,
                None,
            );
            capacity.push(cap);

            flow_map.insert(
                (
                    graph
                        .storage
                        .node_id(&from_key.clone().into())
                        .expect("From node key not found in graph"),
                    graph
                        .storage
                        .node_id(&to_key.clone().into())
                        .expect("To node key not found in graph"),
                ),
                flow,
            );
        }

        let source_id = graph
            .storage
            .node_id(&source_key.clone().into())
            .expect("Source node key not found in graph");
        let sink_id = graph
            .storage
            .node_id(&sink_key.clone().into())
            .expect("Sink node key not found in graph");

        Self {
            graph,
            capacity,
            source: source_id,
            sink: sink_id,
            flow: Flow { map: flow_map },
        }
    }
}

fn residual_network<S, GK, K, D, E, W>(
    flow_network: &FlowNetwork<S, GK, K, D, E, W>,
) -> FlowNetwork<S, GK, K, D, E, W>
where
    S: StorageRepresentation<Key = K, Data = D, EdgeMeta = E, Weight = W>
        + MutableStorage<Key = K, Data = D, EdgeMeta = E, Weight = W>
        + Clone,
    GK: crate::traits::GraphKindMarker + Clone,
    K: Debug + Clone + Eq + std::hash::Hash,
    D: Debug + Clone,
    E: Debug + Clone,
    W: Debug + Copy + PartialOrd,
{
    let mut residual_graph = flow_network.graph.clone();
    residual_graph.storage.clear_edges();

    let mut residual_capacities: Vec<u32> = Vec::new();

    for edge_id in flow_network.graph.edge_ids() {
        let (src, dst) = flow_network.graph.endpoints(edge_id);
        let cap = flow_network.capacity[edge_id.0];

        let fwd_flow = *flow_network.flow.map.get(&(src, dst)).unwrap_or(&0);

        let new_capacity = cap as i32 - fwd_flow;

        if new_capacity > 0 {
            residual_graph.storage.add_edge_by_id(
                src,
                dst,
                flow_network.graph.edge_meta(edge_id).clone(),
                None,
            );
            residual_capacities.push(new_capacity as u32);
        }

        if flow_network.graph.edges_between(dst, src).next().is_none() {
            if fwd_flow > 0 {
                residual_graph.storage.add_edge_by_id(
                    dst,
                    src,
                    flow_network.graph.edge_meta(edge_id).clone(),
                    None,
                );
                residual_capacities.push(fwd_flow as u32);
            }
        }
    }

    FlowNetwork {
        graph: residual_graph,
        capacity: residual_capacities,
        source: flow_network.source,
        sink: flow_network.sink,
        flow: Flow::new(),
    }
}

pub struct FordFulkersonResult<S, GK, K, D, E, W>
where
    S: StorageRepresentation<Key = K, Data = D, EdgeMeta = E, Weight = W>
        + MutableStorage<Key = K, Data = D, EdgeMeta = E, Weight = W>
        + Clone,
    GK: crate::traits::GraphKindMarker + Clone,
    K: Debug + Clone + Eq + std::hash::Hash + Display,
    D: Debug + Clone,
    E: Debug + Clone,
    W: Debug + Copy + PartialOrd,
{
    pub max_flow: u32,
    pub flow: Flow,
    pub steps: Vec<(
        FlowNetwork<S, GK, K, D, E, W>,
        Option<FlowNetwork<S, GK, K, D, E, W>>,
        Vec<K>,
        u32,
    )>,
    phantom: std::marker::PhantomData<K>,
}

impl<S, GK, K, D, E, W> LatexDisplay for FordFulkersonResult<S, GK, K, D, E, W>
where
    S: StorageRepresentation<Key = K, Data = D, EdgeMeta = E, Weight = W>
        + MutableStorage<Key = K, Data = D, EdgeMeta = E, Weight = W>
        + Clone,
    GK: crate::traits::GraphKindMarker + Clone,
    K: Debug + Clone + Eq + std::hash::Hash + Display,
    D: Debug + Clone,
    E: Debug + Clone + Default,
    W: Debug + Copy + PartialOrd,
{
    fn to_latex(&self) -> String {
        let mut result = String::new();
        result.push_str(&format!(
            "\\textbf{{Maximum Flow:}} {}\\\\\n",
            self.max_flow
        ));
        result.push_str("\\textbf{Flow Assignments:}\\\\\n");
        for (&(src, dst), &f) in &self.flow.map {
            if f > 0 {
                result.push_str(&format!("Flow from {} to {}: {}\\\\\n", src.0, dst.0, f));
            }
        }
        result.push_str("\\textbf{Residual Networks at Each Step:}\\\\\n");
        for (i, (residual_network, network, path, flow)) in self.steps.iter().enumerate() {
            result.push_str(&format!(
                "\\textbf{{Step {}}}: Path = [{}], Flow = {}\\\\\n",
                i + 1,
                path.iter()
                    .map(|k| k.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                flow
            ));
            result.push_str(&residual_network.to_latex_visual());
            if let Some(network) = network {
                result.push_str("\\\\\n\\textbf{Augmented Flow Network:}\\\\\n");
                result.push_str(&network.to_latex_visual());
            }
            result.push_str("\\\\\n");
        }
        result
    }
}

pub fn ford_fulkerson<S, GK, K, D, E, W>(
    mut flow_network: FlowNetwork<S, GK, K, D, E, W>,
) -> FordFulkersonResult<S, GK, K, D, E, W>
where
    S: StorageRepresentation<Key = K, Data = D, EdgeMeta = E, Weight = W>
        + MutableStorage<Key = K, Data = D, EdgeMeta = E, Weight = W>
        + Clone,
    GK: crate::traits::GraphKindMarker + Clone,
    K: Debug + Clone + Eq + std::hash::Hash + Display,
    D: Debug + Clone,
    E: Debug + Clone,
    W: Debug + Copy + PartialOrd,
{
    let mut networks = Vec::new();

    let original_flow = flow_network.flow.clone();
    let mut flow: Flow = Flow::new();

    for edge_id in flow_network.graph.edge_ids() {
        let (src, dst) = flow_network.graph.endpoints(edge_id);
        flow.map.insert((src, dst), 0);
        flow.map.insert((dst, src), 0);
    }

    loop {
        let residual_flow_network = residual_network(&flow_network);

        let mut parent: HashMap<NodeId, Option<NodeId>> = HashMap::new();
        let mut visited: HashMap<NodeId, bool> = HashMap::new();

        let mut queue: VecDeque<NodeId> = VecDeque::new();

        let source_id = flow_network.source;
        let sink_id = flow_network.sink;

        queue.push_back(source_id);
        visited.insert(source_id, true);
        parent.insert(source_id, None);

        let mut found_augmenting_path = false;

        while let Some(current) = queue.pop_front() {
            if current == sink_id {
                found_augmenting_path = true;
                break;
            }

            for neighbor in residual_flow_network.graph.successors(current) {
                if !visited.get(&neighbor).unwrap_or(&false) {
                    visited.insert(neighbor, true);
                    parent.insert(neighbor, Some(current));
                    queue.push_back(neighbor);
                }
            }
        }

        if !found_augmenting_path {
            networks.push((residual_flow_network, None, Vec::new(), 0));

            break;
        }

        let mut path_capacity = u32::MAX;
        let mut v = sink_id;
        while let Some(u) = parent[&v] {
            let edge_ids: Vec<EdgeId> = residual_flow_network.graph.edges_between(u, v).collect();
            if let Some(edge_id) = edge_ids.first() {
                let cap_index = edge_id.0;
                let cap = residual_flow_network.capacity[cap_index];
                path_capacity = path_capacity.min(cap);
            }
            v = u;
        }

        let mut path_keys: Vec<K> = Vec::from_iter([flow_network.graph.node_key(sink_id).clone()]);

        v = sink_id;
        while let Some(u) = parent[&v] {
            path_keys.push(flow_network.graph.node_key(u).clone());
            *flow.map.entry((u, v)).or_insert(0) += path_capacity as i32;
            *flow.map.entry((v, u)).or_insert(0) -= path_capacity as i32;
            v = u;
        }

        flow_network.flow = original_flow.clone() + &flow;
        let mut augmented_flow_network = flow_network.clone();
        augmented_flow_network.flow = flow.clone();

        path_keys.reverse();

        networks.push((
            residual_flow_network,
            Some(augmented_flow_network),
            path_keys,
            path_capacity,
        ));
    }

    let max_flow: u32 = flow
        .map
        .iter()
        .filter_map(|(&(src, _), &f)| {
            if src == flow_network.source {
                Some(f)
            } else {
                None
            }
        })
        .sum::<i32>() as u32;

    FordFulkersonResult {
        max_flow,
        flow,
        phantom: std::marker::PhantomData,
        steps: networks,
    }
}
