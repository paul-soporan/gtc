use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display};
use std::hash::Hash;

use crate::{
    EdgeId, GraphBase, GraphKindMarker, LatexDisplay, NodeId, StorageRepresentation,
    UndirectedGraph,
};

/// Result of Hierholzer's algorithm containing the Eulerian circuit path.
pub struct HierholzerResult<K> {
    pub path: Vec<K>,
}

impl<K: Display> LatexDisplay for HierholzerResult<K> {
    fn to_latex(&self) -> String {
        if self.path.is_empty() {
            return "\\text{No Eulerian Circuit found}".to_string();
        }
        self.path
            .iter()
            .map(|k| k.to_string())
            .collect::<Vec<_>>()
            .join(" \\to ")
    }
}

pub fn hierholzer_undirected<S, GK, K, D, E, W>(
    graph: &UndirectedGraph<S, GK, K, D, E, W>,
) -> Result<HierholzerResult<K>, String>
where
    S: StorageRepresentation<Key = K, Data = D, EdgeMeta = E, Weight = W>,
    GK: GraphKindMarker,
    K: Clone + Eq + Hash + Debug,
    D: Clone + Debug,
    E: Clone + Debug,
    W: Copy + PartialOrd + Debug,
{
    if graph.size() == 0 {
        if graph.order() > 0 {
            return Ok(HierholzerResult {
                path: vec![graph.node_key(NodeId(0)).clone()],
            });
        }
        return Ok(HierholzerResult { path: vec![] });
    }

    let mut adjacency_list: HashMap<NodeId, Vec<EdgeId>> = HashMap::new();
    let mut degree: HashMap<NodeId, usize> = HashMap::new();

    for id in graph.node_ids() {
        adjacency_list.insert(id, Vec::new());
        degree.insert(id, 0);
    }

    for eid in graph.edge_ids() {
        let (u, v) = graph.endpoints(eid);
        adjacency_list.get_mut(&u).unwrap().push(eid);
        adjacency_list.get_mut(&v).unwrap().push(eid);
        *degree.get_mut(&u).unwrap() += 1;
        *degree.get_mut(&v).unwrap() += 1;
    }

    let mut start_node = None;
    for id in graph.node_ids() {
        if degree[&id] % 2 != 0 {
            return Err(format!(
                "Graph is not Eulerian: Node {:?} has odd degree {}",
                graph.node_key(id),
                degree[&id]
            ));
        }
        if degree[&id] > 0 && start_node.is_none() {
            start_node = Some(id);
        }
    }

    let start_node = match start_node {
        Some(node) => node,
        None => return Ok(HierholzerResult { path: vec![] }),
    };

    let mut used_edges = HashSet::new();
    let mut circuit = Vec::new();
    let mut curr_path = vec![start_node];

    while let Some(&u) = curr_path.last() {
        let mut next_edge = None;
        let edges = adjacency_list.get_mut(&u).unwrap();

        while let Some(eid) = edges.pop() {
            if !used_edges.contains(&eid) {
                next_edge = Some(eid);
                break;
            }
        }

        if let Some(eid) = next_edge {
            used_edges.insert(eid);
            let (v1, v2) = graph.endpoints(eid);
            let v = if v1 == u { v2 } else { v1 };
            curr_path.push(v);
        } else {
            circuit.push(curr_path.pop().unwrap());
        }
    }

    if circuit.len() != graph.size() + 1 {
        return Err("Graph has disconnected components with edges.".to_string());
    }

    circuit.reverse();
    let path_keys = circuit
        .into_iter()
        .map(|id| graph.node_key(id).clone())
        .collect();

    Ok(HierholzerResult { path: path_keys })
}
