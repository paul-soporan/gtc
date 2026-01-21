use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display};
use std::hash::Hash;

use crate::{
    EdgeId, EdgeWeights, Graph, LatexDisplay, LatexVisualDisplay, VisualEdge, VisualGraphData,
    generate_latex_graph,
};

pub struct KruskalResult<K, W> {
    pub edges: Vec<(K, K, W)>,
    pub total_weight: W,
}

impl<K, W> LatexDisplay for KruskalResult<K, W>
where
    K: Display,
    W: Display,
{
    fn to_latex(&self) -> String {
        let mut s = String::new();
        s.push_str("\\begin{itemize}\n");
        for (u, v, w) in &self.edges {
            s.push_str(&format!("  \\item ({}, {}) : {}\n", u, v, w));
        }
        s.push_str(&format!(
            "  \\item \\textbf{{Total Weight:}} {}\n",
            self.total_weight
        ));
        s.push_str("\\end{itemize}");
        s
    }
}

impl<K, W> LatexVisualDisplay for KruskalResult<K, W>
where
    K: Clone + Eq + Hash + Display,
    W: Display + Clone,
{
    fn to_latex_visual(&self) -> String {
        let mut unique_nodes = HashSet::new();
        for (u, v, _) in &self.edges {
            unique_nodes.insert(u);
            unique_nodes.insert(v);
        }

        let mut sorted_nodes: Vec<&K> = unique_nodes.into_iter().collect();
        sorted_nodes.sort_by_key(|k| k.to_string());

        let mut node_to_idx = HashMap::new();
        let mut labels = Vec::new();
        for (i, node) in sorted_nodes.iter().enumerate() {
            node_to_idx.insert(*node, i);
            labels.push(node.to_string());
        }

        let mut visual_edges = Vec::new();
        for (u, v, w) in &self.edges {
            let u_idx = *node_to_idx.get(u).expect("Node should exist in map");
            let v_idx = *node_to_idx.get(v).expect("Node should exist in map");
            visual_edges.push(VisualEdge {
                u: u_idx,
                v: v_idx,
                label: Some(w.to_string()),
            });
        }

        let data = VisualGraphData {
            labels,
            edges: visual_edges,
            is_directed: false,
        };

        generate_latex_graph(data)
    }
}

/// Helper Disjoint Set Union (DSU) / Union-Find data structure.
struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }

    fn find(&mut self, i: usize) -> usize {
        if self.parent[i] != i {
            // Path compression: point directly to root
            self.parent[i] = self.find(self.parent[i]);
        }
        self.parent[i]
    }

    fn union(&mut self, i: usize, j: usize) -> bool {
        let root_i = self.find(i);
        let root_j = self.find(j);

        if root_i != root_j {
            // Union by rank: attach smaller tree to larger tree
            match self.rank[root_i].cmp(&self.rank[root_j]) {
                Ordering::Less => self.parent[root_i] = root_j,
                Ordering::Greater => self.parent[root_j] = root_i,
                Ordering::Equal => {
                    self.parent[root_j] = root_i;
                    self.rank[root_i] += 1;
                }
            }
            true
        } else {
            false
        }
    }
}

pub fn kruskal_mst<G, W>(graph: &G) -> KruskalResult<G::Key, W>
where
    G: Graph,
    G::Key: Eq + Hash + Clone + Debug,
    G: EdgeWeights<W = W>,
    W: Copy + PartialOrd + std::ops::Add<Output = W> + Default + Debug,
{
    let mut edges: Vec<(EdgeId, W)> = Vec::new();
    for eid in graph.edge_ids() {
        if let Some(w) = graph.weight_of(eid) {
            edges.push((eid, w));
        }
    }

    edges.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));

    let mut uf = UnionFind::new(graph.order());
    let mut mst_edges = Vec::new();
    let mut total_weight = W::default();

    for (eid, w) in edges {
        let (u, v) = graph.endpoints(eid);

        if uf.union(u.0, v.0) {
            mst_edges.push((graph.node_key(u).clone(), graph.node_key(v).clone(), w));
            total_weight = total_weight + w;
        }
    }

    KruskalResult {
        edges: mst_edges,
        total_weight,
    }
}
