use std::hash::Hash;

use crate::{Graph, LatexDisplay, LatexMatrix};

pub struct WarshallClosureResult<K> {
    pub nodes: Vec<K>,
    pub closure: Vec<Vec<bool>>,
}

impl<K: std::fmt::Display> LatexDisplay for WarshallClosureResult<K> {
    fn to_latex(&self) -> String {
        let labels = self.nodes.iter().map(|k| k.to_string()).collect::<Vec<_>>();

        LatexMatrix {
            data: &self.closure,
            col_labels: labels.clone(),
            row_labels: labels,
            format_cell: &|cell| {
                if *cell {
                    "1".to_string()
                } else {
                    "0".to_string()
                }
            },
        }
        .to_latex()
    }
}

pub fn warshall_closure<G>(graph: &G) -> WarshallClosureResult<G::Key>
where
    G: Graph,
    G::Key: Eq + Hash,
{
    let n = graph.order();
    let mut closure = vec![vec![false; n]; n];

    for i in 0..n {
        closure[i][i] = true;
    }

    for edge_id in graph.edge_ids() {
        let (src, dst) = graph.endpoints(edge_id);
        closure[src.0][dst.0] = true;
    }

    for k in 0..n {
        for i in 0..n {
            for j in 0..n {
                if closure[i][k] && closure[k][j] {
                    closure[i][j] = true;
                }
            }
        }
    }

    WarshallClosureResult {
        nodes: graph
            .node_ids()
            .map(|nid| graph.node_key(nid).clone())
            .collect(),
        closure,
    }
}

#[derive(Clone)]
pub struct WarshallPathMatrix<K, W> {
    pub nodes: Vec<K>,
    pub paths: Vec<Vec<Option<(Vec<usize>, W)>>>,
}

impl<K, W> LatexDisplay for WarshallPathMatrix<K, W>
where
    K: std::fmt::Display,
    W: Copy + std::fmt::Display,
{
    fn to_latex(&self) -> String {
        let labels = self.nodes.iter().map(|k| k.to_string()).collect::<Vec<_>>();

        LatexMatrix {
            data: &self.paths,
            col_labels: labels.clone(),
            row_labels: labels,
            format_cell: &|cell| match cell {
                None => "\\emptyset".to_string(),
                Some((path, weight)) => {
                    let path_str = path
                        .iter()
                        .map(|&idx| self.nodes[idx].to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("[{}] _{{{}}}", path_str, weight)
                }
            },
        }
        .to_latex()
    }
}

pub struct WarshallLightestPathResult<K, W> {
    pub nodes: Vec<K>,
    pub matrices: Vec<WarshallPathMatrix<K, W>>,
}

impl<K, W> LatexDisplay for WarshallLightestPathResult<K, W>
where
    K: std::fmt::Display,
    W: Copy + std::fmt::Display,
{
    fn to_latex(&self) -> String {
        let mut result = String::new();
        for (i, matrix) in self.matrices.iter().enumerate() {
            result.push_str(&format!("\\\\\\textbf{{Iteration {}}}\\\\\n", i));
            result.push_str(&matrix.to_latex());
            result.push_str("\n\n");
        }
        result
    }
}

pub fn warshall_lightest_path_matrix<G, W>(graph: &G) -> WarshallLightestPathResult<G::Key, W>
where
    G: Graph,
    G::Key: Eq + Hash,
    G: crate::EdgeWeights<W = W>,
    W: Copy + PartialOrd + std::ops::Add<Output = W>,
{
    let n = graph.order();
    let paths = vec![vec![None; n]; n];
    let mut warshall_path_matrix = WarshallPathMatrix {
        nodes: graph
            .node_ids()
            .map(|nid| graph.node_key(nid).clone())
            .collect(),
        paths,
    };

    for edge_id in graph.edge_ids() {
        let (src, dst) = graph.endpoints(edge_id);
        if let Some(weight) = graph.weight_of(edge_id) {
            warshall_path_matrix.paths[src.0][dst.0] = Some((vec![src.0, dst.0], weight));
        }
    }

    let mut matrices = Vec::new();
    matrices.push(warshall_path_matrix.clone());

    for k in 0..n {
        for i in 0..n {
            for j in 0..n {
                if let (Some((path_ik, weight_ik)), Some((path_kj, weight_kj))) = (
                    &warshall_path_matrix.paths[i][k],
                    &warshall_path_matrix.paths[k][j],
                ) {
                    let new_weight = *weight_ik + *weight_kj;
                    match &warshall_path_matrix.paths[i][j] {
                        Some((_, existing_weight)) => {
                            if new_weight < *existing_weight {
                                let mut new_path = path_ik.clone();
                                new_path.pop();
                                new_path.extend(path_kj.iter());
                                warshall_path_matrix.paths[i][j] = Some((new_path, new_weight));
                            }
                        }
                        None => {
                            let mut new_path = path_ik.clone();
                            new_path.pop();
                            new_path.extend(path_kj.iter());
                            warshall_path_matrix.paths[i][j] = Some((new_path, new_weight));
                        }
                    }
                }
            }
        }

        matrices.push(warshall_path_matrix.clone());
    }

    WarshallLightestPathResult {
        nodes: graph
            .node_ids()
            .map(|nid| graph.node_key(nid).clone())
            .collect(),
        matrices,
    }
}

pub struct GraphDistances<K> {
    pub nodes: Vec<K>,
    pub eccentricities: Vec<Option<usize>>,
    pub radius: Option<usize>,
    pub diameter: Option<usize>,
}

impl<K> LatexDisplay for GraphDistances<K>
where
    K: std::fmt::Display,
{
    // radius, diameter, center nodes, periphery nodes, table of eccentricities
    fn to_latex(&self) -> String {
        let mut result = String::new();

        if let Some(radius) = self.radius {
            result.push_str(&format!("\\\\\\textbf{{Radius}}: {}\\\\\n", radius));
        } else {
            result.push_str("\\\\\\textbf{Radius}: \\text{Undefined}\\\\\n");
        }

        if let Some(diameter) = self.diameter {
            result.push_str(&format!("\\\\\\textbf{{Diameter}}: {}\\\\\n", diameter));
        } else {
            result.push_str("\\\\\\textbf{Diameter}: \\text{Undefined}\\\\\n");
        }

        let center_nodes = self.center_nodes();
        let center_keys = center_nodes
            .iter()
            .map(|&idx| self.nodes[idx].to_string())
            .collect::<Vec<_>>()
            .join(", ");
        result.push_str(&format!(
            "\\\\\\textbf{{Center Nodes}}: {{{}}}\\\\\n",
            center_keys
        ));

        let periphery_nodes = self.periphery_nodes();
        let periphery_keys = periphery_nodes
            .iter()
            .map(|&idx| self.nodes[idx].to_string())
            .collect::<Vec<_>>()
            .join(", ");
        result.push_str(&format!(
            "\\\\\\textbf{{Periphery Nodes}}: {{{}}}\\\\\n",
            periphery_keys
        ));

        result.push_str("\\\\\\textbf{Eccentricities}:\\\\\n");
        result.push_str("\\begin{tabular}{|c|c|}\\hline\n");
        result.push_str("Node & Eccentricity \\\\ \\hline\n");
        for (i, ecc) in self.eccentricities.iter().enumerate() {
            let ecc_str = match ecc {
                Some(e) => e.to_string(),
                None => "\\infty".to_string(),
            };
            result.push_str(&format!("{} & {} \\\\ \\hline\n", self.nodes[i], ecc_str));
        }
        result.push_str("\\end{tabular}\n");

        result
    }
}

impl<K> GraphDistances<K> {
    pub fn center_nodes(&self) -> Vec<usize> {
        let mut centers = Vec::new();
        if let Some(radius) = self.radius {
            for (i, ecc) in self.eccentricities.iter().enumerate() {
                if let Some(e) = ecc {
                    if *e == radius {
                        centers.push(i);
                    }
                }
            }
        }
        centers
    }

    pub fn periphery_nodes(&self) -> Vec<usize> {
        let mut periphery = Vec::new();
        if let Some(diameter) = self.diameter {
            for (i, ecc) in self.eccentricities.iter().enumerate() {
                if let Some(e) = ecc {
                    if *e == diameter {
                        periphery.push(i);
                    }
                }
            }
        }
        periphery
    }
}

pub fn compute_graph_distances<K>(matrix: &WarshallLightestPathResult<K, i32>) -> GraphDistances<K>
where
    K: Clone,
{
    let n = matrix.matrices[0].nodes.len();
    let mut eccentricities = vec![None; n];

    for i in 0..n {
        let mut max_distance: Option<usize> = None;
        for j in 0..n {
            if i != j {
                if let Some((_, weight)) = &matrix.matrices.last().unwrap().paths[i][j] {
                    let dist = *weight as usize;
                    max_distance = match max_distance {
                        Some(current_max) => Some(current_max.max(dist)),
                        None => Some(dist),
                    };
                }
            }
        }
        eccentricities[i] = max_distance;
    }

    let radius = eccentricities.iter().filter_map(|&e| e).min();

    let diameter = eccentricities.iter().filter_map(|&e| e).max();

    GraphDistances {
        nodes: matrix.nodes.clone(),
        eccentricities,
        radius,
        diameter,
    }
}
