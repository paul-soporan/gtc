use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

use crate::{Graph, GraphDefinition, NodeId};

pub fn tree_to_prufer<G>(graph: &G) -> Vec<G::Key>
where
    G: Graph,
    G::Key: Clone + Eq + Hash + Ord + Debug,
{
    let n = graph.order();
    if n < 2 {
        return Vec::new();
    }

    let mut degrees: HashMap<NodeId, usize> = HashMap::new();
    let mut removed: HashSet<NodeId> = HashSet::new();
    let mut min_heap: BinaryHeap<Reverse<G::Key>> = BinaryHeap::new();

    for i in 0..n {
        let nid = NodeId(i);

        let unique_neighbors: HashSet<NodeId> = graph.neighborhood(nid).collect();
        let degree = unique_neighbors.len();

        degrees.insert(nid, degree);

        if degree == 1 {
            min_heap.push(Reverse(graph.node_key(nid).clone()));
        }
    }

    let mut prufer_sequence = Vec::with_capacity(n - 2);

    for _ in 0..(n - 2) {
        let leaf_key = min_heap
            .pop()
            .expect("Graph is not a tree or disconnected")
            .0;
        let leaf_id = graph.node_id(&leaf_key).expect("Node key not found");

        removed.insert(leaf_id);

        let mut neighbor_id = None;
        for neighbor in graph.neighborhood(leaf_id) {
            if !removed.contains(&neighbor) {
                neighbor_id = Some(neighbor);
                break;
            }
        }

        let neighbor_id = neighbor_id.expect("Leaf must have a neighbor");
        let neighbor_key = graph.node_key(neighbor_id).clone();

        prufer_sequence.push(neighbor_key.clone());

        if let Some(d) = degrees.get_mut(&neighbor_id) {
            *d = d.saturating_sub(1);

            if *d == 1 {
                min_heap.push(Reverse(neighbor_key));
            }
        }
    }

    prufer_sequence
}

pub fn prufer_to_tree(sequence: &[usize]) -> GraphDefinition<usize, (), (), ()> {
    let n = sequence.len() + 2;
    let mut def = GraphDefinition::new();

    for i in 1..=n {
        def.add_node(i, ());
    }

    if n == 2 {
        def.add_edge_by_key(1, 2, (), (), (), None);
        return def;
    }

    let mut degrees = vec![1; n + 1];
    for &node in sequence {
        if node < 1 || node > n {
            panic!(
                "Invalid Prüfer sequence: node index {} out of bounds for range 1..={}",
                node, n
            );
        }
        degrees[node] += 1;
    }

    let mut min_heap: BinaryHeap<Reverse<usize>> = BinaryHeap::new();
    for i in 1..=n {
        if degrees[i] == 1 {
            min_heap.push(Reverse(i));
        }
    }

    for &v in sequence {
        let u = min_heap.pop().expect("Heap should not be empty").0;

        def.add_edge_by_key(u, v, (), (), (), None);

        degrees[v] -= 1;
        if degrees[v] == 1 {
            min_heap.push(Reverse(v));
        }
    }

    let u = min_heap.pop().unwrap().0;
    let v = min_heap.pop().unwrap().0;
    def.add_edge_by_key(u, v, (), (), (), None);

    def
}
