use std::hash::{Hash, RandomState};

use indexmap::IndexSet;

use crate::{EdgeWeights, Graph, LatexDisplay, NodeId, StorageRepresentation};

pub struct DijkstraResult<K>
where
    K: Clone + Eq + Hash,
{
    pub nodes: Vec<K>,
    pub tentative_weights: Vec<Option<i32>>,
    pub predecessors: Vec<Option<NodeId>>,
    _marker: std::marker::PhantomData<K>,
}

impl<K> DijkstraResult<K>
where
    K: Clone + Eq + Hash,
{
    pub fn lightest_path_to(&self, target: &K) -> Option<(i32, Vec<K>)> {
        let target_index = self
            .nodes
            .iter()
            .position(|k| k == target)
            .expect("Target node not found in DijkstraResult");

        let tentative_weight = self.tentative_weights[target_index]?;

        let mut path = Vec::new();
        let mut current_index = target_index;

        while let Some(pred) = &self.predecessors[current_index] {
            path.push(self.nodes[current_index].clone());
            current_index = pred.0;
        }
        path.push(self.nodes[current_index].clone());
        path.reverse();

        Some((tentative_weight, path))
    }
}

impl LatexDisplay for DijkstraResult<String> {
    fn to_latex(&self) -> String {
        let mut result = String::new();
        result.push_str("\\begin{tabular}{|c|c|c|}\n\\hline\n");
        result.push_str("Node & Tentative Weight & Predecessor \\\\\n\\hline\n");
        for (i, (weight, pred)) in self
            .tentative_weights
            .iter()
            .zip(self.predecessors.iter())
            .enumerate()
        {
            let node = &self.nodes[i];
            let weight_str = match weight {
                Some(w) => w.to_string(),
                None => "\\infty".to_string(),
            };
            let pred_str = match pred {
                Some(p) => self.nodes[p.0].to_string(),
                None => "undef".to_string(),
            };
            result.push_str(&format!("{} & {} & {} \\\\\n", node, weight_str, pred_str));
        }
        result.push_str("\\hline\n\\end{tabular}\n");
        result
    }
}

pub fn dijkstra<G, S, K>(graph: &G, start: K) -> DijkstraResult<K>
where
    G: Graph<Storage = S> + EdgeWeights<W = i32>,
    S: StorageRepresentation<Key = K, Weight = i32>,
    K: Clone + Eq + Hash,
{
    let source_id = graph
        .node_id(&start)
        .expect("Start node not found in graph");

    let mut tentative_weights = Vec::with_capacity(graph.order());
    let mut predecessors = Vec::with_capacity(graph.order());

    for _ in 0..graph.order() {
        tentative_weights.push(None);
        predecessors.push(None);
    }

    tentative_weights[source_id.0] = Some(0);
    predecessors[source_id.0] = None;

    let mut unvisited: IndexSet<NodeId, RandomState> =
        IndexSet::from_iter((0..graph.order()).map(|i| NodeId(i)));

    while !unvisited.is_empty() {
        let current = unvisited
            .iter()
            .min_by(|&&a, &&b| {
                let wa = tentative_weights[a.0];
                let wb = tentative_weights[b.0];
                match (wa, wb) {
                    (Some(wa), Some(wb)) => wa.partial_cmp(&wb).unwrap(),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                }
            })
            .cloned()
            .expect("No reachable unvisited nodes remaining");

        unvisited.shift_remove(&current);

        for neighbor in graph.successors(current) {
            if !unvisited.contains(&neighbor) {
                continue;
            }

            let edges = graph.edges_between(current, neighbor);
            let min_edge_weight = edges.filter_map(|eid| graph.weight_of(eid)).min().expect(
                "There should be at least one edge between current and neighbor in successors",
            );

            let alt_weight = tentative_weights[current.0]
                .map(|w| w + min_edge_weight)
                .expect("Current node should have a tentative weight");

            if tentative_weights[neighbor.0].map_or(true, |w| alt_weight < w) {
                tentative_weights[neighbor.0] = Some(alt_weight);
                predecessors[neighbor.0] = Some(current);
            }
        }
    }

    DijkstraResult {
        nodes: (0..graph.order())
            .map(|i| graph.node_key(NodeId(i)).clone())
            .collect(),
        tentative_weights,
        predecessors,
        _marker: std::marker::PhantomData,
    }
}
