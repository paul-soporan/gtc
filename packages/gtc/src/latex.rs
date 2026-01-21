use core::panic;
use std::{
    fmt::{Debug, Display},
    hash::Hash,
};

use crate::{
    AdjacencyMatrix, DirectedGraph, EdgeWeights, GraphBase, GraphDefinition, GraphKindMarker,
    NodeId, StorageRepresentation, UndirectedGraph, VisualEdge, VisualGraphData,
    generate_latex_graph,
};

pub trait LatexDisplay {
    fn to_latex(&self) -> String;
}

pub trait LatexVisualDisplay {
    fn to_latex_visual(&self) -> String;
}

pub struct LatexMatrix<'a, T> {
    pub row_labels: Vec<String>,
    pub col_labels: Vec<String>,
    pub data: &'a Vec<Vec<T>>,
    pub format_cell: &'a dyn Fn(&T) -> String,
}

impl<T> LatexDisplay for LatexMatrix<'_, T> {
    fn to_latex(&self) -> String {
        let mut row_label_indices: Vec<usize> = (0..self.row_labels.len()).collect();
        row_label_indices.sort_by_key(|&i| &self.row_labels[i]);

        let mut col_label_indices: Vec<usize> = (0..self.col_labels.len()).collect();
        col_label_indices.sort_by_key(|&i| &self.col_labels[i]);

        let mut latex_string = String::new();

        // 1. Determine which options to enable based on available data
        let has_row_labels = !self.row_labels.is_empty();
        let has_col_labels = !self.col_labels.is_empty();

        let mut options = Vec::new();
        if has_col_labels {
            options.push("first-row");
        }
        if has_row_labels {
            options.push("first-col");
        }

        // 2. Open Environment with options
        latex_string.push_str("$\\begin{pNiceMatrix}");
        if !options.is_empty() {
            latex_string.push('[');
            latex_string.push_str(&options.join(","));
            latex_string.push(']');
        }
        latex_string.push('\n');

        // 3. Generate Header Row (Top Column Labels)
        if has_col_labels {
            // Empty cell for the top-left corner (above first L_1)
            if has_row_labels {
                latex_string.push_str("    & ");
            }

            for (idx, &i) in col_label_indices.iter().enumerate() {
                let label = &self.col_labels[i];
                latex_string.push_str(label);

                // Add separator unless it's the last column label
                if idx < self.col_labels.len() - 1 {
                    latex_string.push_str(" & ");
                }
            }

            // Empty cell for the top-right corner (above last L_1)
            if has_row_labels {
                latex_string.push_str(" & ");
            }

            latex_string.push_str(" \\\\\n");
        }

        // 4. Generate Body Rows (Left Label -> Data)
        for i in row_label_indices {
            let row = &self.data[i];

            // Left Row Label
            if has_row_labels {
                if let Some(label) = self.row_labels.get(i) {
                    latex_string.push_str(label);
                    latex_string.push_str(" & ");
                }
            }

            // Data Cells
            for (idx, &j) in col_label_indices.iter().enumerate() {
                let cell = &row[j];
                latex_string.push_str(&(self.format_cell)(cell));
                if idx < row.len() - 1 {
                    latex_string.push_str(" & ");
                }
            }

            latex_string.push_str(" \\\\\n");
        }

        latex_string.push_str("\\end{pNiceMatrix}$");
        latex_string
    }
}

impl LatexDisplay for () {
    fn to_latex(&self) -> String {
        String::from("1")
    }
}

impl LatexDisplay for i64 {
    fn to_latex(&self) -> String {
        format!("{}", self)
    }
}

impl LatexDisplay for f64 {
    fn to_latex(&self) -> String {
        format!("{:.2}", self)
    }
}

impl<S, GK, K, D, E, W> LatexDisplay for DirectedGraph<S, GK, K, D, E, W>
where
    S: StorageRepresentation<Key = K, Data = D, EdgeMeta = E, Weight = W> + LatexDisplay,
    GK: GraphKindMarker,
    K: Clone + Eq + std::hash::Hash,
{
    fn to_latex(&self) -> String {
        self.storage.to_latex()
    }
}

impl<S, GK, K, D, E, W> LatexDisplay for UndirectedGraph<S, GK, K, D, E, W>
where
    S: StorageRepresentation<Key = K, Data = D, EdgeMeta = E, Weight = W> + LatexDisplay,
    GK: GraphKindMarker,
    K: Clone + Eq + std::hash::Hash,
{
    fn to_latex(&self) -> String {
        self.storage.to_latex()
    }
}

impl<K, D, E, W> LatexDisplay for GraphDefinition<K, D, E, W>
where
    K: Debug + Clone + Eq + Hash + Display,
    D: Debug + Clone,
    E: Debug + Clone,
    W: Debug + Copy + PartialOrd,
{
    fn to_latex(&self) -> String {
        let definition_string = format!(
            "G = (V, E) with |V| = {} and |E| = {}",
            self.order(),
            self.size()
        );

        let mut nodes = self
            .nodes
            .iter()
            .map(|(_, record)| record.key.to_string())
            .collect::<Vec<_>>();

        nodes.sort();

        let nodes_string = format!("V = {{ {} }}", nodes.join(", "));

        let mut edges = self
            .edges
            .iter()
            .map(|edge| {
                let from_key = &self.nodes.get(edge.from).key;
                let to_key = &self.nodes.get(edge.to).key;
                format!("({}, {})", from_key, to_key)
            })
            .collect::<Vec<_>>();

        let edges_string = format!("E = {{ {} }}", edges.join(", "));

        edges.sort();

        definition_string + "\n" + &nodes_string + "\n" + &edges_string
    }
}

impl<K, D, E, W> LatexDisplay for AdjacencyMatrix<K, D, E, W>
where
    K: Debug + Clone + Eq + Hash + Default,
    D: Debug + Clone + Default,
    E: Debug + Clone + Default,
    W: Debug + Copy + PartialOrd + LatexDisplay,
{
    fn to_latex(&self) -> String {
        let mut latex_string = String::new();

        latex_string.push_str("\\begin{pmatrix}\n");
        for i in 0..self.n {
            let mut row_entries = Vec::new();
            for j in 0..self.n {
                let edge = self.get_edge_id(NodeId(i), NodeId(j));

                let weight = match edge {
                    Some(eid) => match self.weight_of(eid) {
                        Some(w) => &w.to_latex(),
                        None => panic!("Edge weight should be defined for existing edges"),
                    },
                    None => "∞",
                };
                row_entries.push(format!("{}", weight));
            }
            latex_string.push_str(&row_entries.join(" & "));
            if i < self.n - 1 {
                latex_string.push_str(" \\\\\n");
            } else {
                latex_string.push_str("\n");
            }
        }
        latex_string.push_str("\\end{pmatrix}");

        latex_string
    }
}

impl<S, GK, K, D, E, W> LatexVisualDisplay for DirectedGraph<S, GK, K, D, E, W>
where
    DirectedGraph<S, GK, K, D, E, W>: EdgeWeights<W = W>,
    S: StorageRepresentation<Key = K, Data = D, EdgeMeta = E, Weight = W>,
    GK: GraphKindMarker,
    K: Debug + Clone + Eq + Hash + Display,
    D: Debug + Clone,
    E: Debug + Clone,
    W: Debug + Copy + PartialOrd,
{
    fn to_latex_visual(&self) -> String {
        let n = self.order();
        let labels: Vec<String> = (0..n)
            .map(|i| self.node_key(NodeId(i)).to_string())
            .collect();

        let mut edges = Vec::new();
        for eid in self.storage.edge_ids() {
            let (u, v) = self.endpoints(eid);
            let label = self.weight_of(eid).map(|w| format!("{:?}", w));
            edges.push(VisualEdge {
                u: u.0,
                v: v.0,
                label,
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

impl<S, GK, K, D, E, W> LatexVisualDisplay for UndirectedGraph<S, GK, K, D, E, W>
where
    UndirectedGraph<S, GK, K, D, E, W>: EdgeWeights<W = W>,
    S: StorageRepresentation<Key = K, Data = D, EdgeMeta = E, Weight = W>,
    GK: GraphKindMarker,
    K: Debug + Clone + Eq + Hash + Display,
    D: Debug + Clone,
    E: Debug + Clone,
    W: Debug + Copy + PartialOrd,
{
    fn to_latex_visual(&self) -> String {
        let n = self.order();
        let labels: Vec<String> = (0..n)
            .map(|i| self.node_key(NodeId(i)).to_string())
            .collect();

        let mut edges = Vec::new();
        for eid in self.storage.edge_ids() {
            let (u, v) = self.endpoints(eid);
            let label = self.weight_of(eid).map(|w| format!("{:?}", w));
            edges.push(VisualEdge {
                u: u.0,
                v: v.0,
                label,
            });
        }

        let data = VisualGraphData {
            labels,
            edges,
            is_directed: false,
        };

        generate_latex_graph(data)
    }
}
