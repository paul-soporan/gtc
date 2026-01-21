use std::fmt::Display;
use std::ops::{Add, Mul, Sub};

use crate::{Graph, LatexDisplay, NodeId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Polynomial {
    // Coefficients of powers of x, starting from x^0.
    // coeffs[i] is the coefficient of x^i
    pub coeffs: Vec<i64>,
}

impl Polynomial {
    pub fn zero() -> Self {
        Self { coeffs: vec![0] }
    }

    pub fn one() -> Self {
        Self { coeffs: vec![1] }
    }

    pub fn x() -> Self {
        Self { coeffs: vec![0, 1] }
    }

    pub fn from_monomial(power: usize, coeff: i64) -> Self {
        let mut coeffs = vec![0; power + 1];
        coeffs[power] = coeff;
        Self { coeffs }
    }

    /// Evaluates the polynomial at a given value x.
    pub fn eval(&self, x: i64) -> i64 {
        let mut result = 0;
        let mut power_of_x = 1;
        for &c in &self.coeffs {
            result += c * power_of_x;
            power_of_x *= x;
        }
        result
    }

    /// Normalizes vector (removes trailing zeros).
    fn normalize(&mut self) {
        while self.coeffs.len() > 1 && self.coeffs.last() == Some(&0) {
            self.coeffs.pop();
        }
    }
}

impl Add for Polynomial {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let len = std::cmp::max(self.coeffs.len(), other.coeffs.len());
        let mut new_coeffs = vec![0; len];

        for (i, c) in self.coeffs.iter().enumerate() {
            new_coeffs[i] += c;
        }
        for (i, c) in other.coeffs.iter().enumerate() {
            new_coeffs[i] += c;
        }

        let mut p = Polynomial { coeffs: new_coeffs };
        p.normalize();
        p
    }
}

impl Sub for Polynomial {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let len = std::cmp::max(self.coeffs.len(), other.coeffs.len());
        let mut new_coeffs = vec![0; len];

        for (i, c) in self.coeffs.iter().enumerate() {
            new_coeffs[i] += c;
        }
        for (i, c) in other.coeffs.iter().enumerate() {
            new_coeffs[i] -= c;
        }

        let mut p = Polynomial { coeffs: new_coeffs };
        p.normalize();
        p
    }
}

impl Mul for Polynomial {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        if self.coeffs.is_empty() || other.coeffs.is_empty() {
            return Polynomial::zero();
        }
        let new_len = self.coeffs.len() + other.coeffs.len() - 1;
        let mut new_coeffs = vec![0; new_len];

        for (i, c1) in self.coeffs.iter().enumerate() {
            if *c1 == 0 {
                continue;
            }
            for (j, c2) in other.coeffs.iter().enumerate() {
                new_coeffs[i + j] += c1 * c2;
            }
        }

        let mut p = Polynomial { coeffs: new_coeffs };
        p.normalize();
        p
    }
}

impl Display for Polynomial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.coeffs.len() == 1 && self.coeffs[0] == 0 {
            return write!(f, "0");
        }

        let mut first = true;
        for i in (0..self.coeffs.len()).rev() {
            let coeff = self.coeffs[i];
            if coeff == 0 {
                continue;
            }

            if !first {
                if coeff > 0 {
                    write!(f, " + ")?;
                } else {
                    write!(f, " - ")?;
                }
            } else if coeff < 0 {
                write!(f, "-")?;
            }

            let abs_coeff = coeff.abs();
            if abs_coeff != 1 || i == 0 {
                write!(f, "{}", abs_coeff)?;
            }

            if i > 0 {
                write!(f, "x")?;
                if i > 1 {
                    write!(f, "^{{{}}}", i)?;
                }
            }
            first = false;
        }
        Ok(())
    }
}

impl LatexDisplay for Polynomial {
    fn to_latex(&self) -> String {
        format!("P_G(x) = {}", self)
    }
}

#[derive(Clone, Debug)]
struct WorkingGraph {
    adj: Vec<Vec<bool>>,
    n: usize,
}

impl WorkingGraph {
    fn from_graph<G>(graph: &G) -> Self
    where
        G: Graph,
    {
        let n = graph.order();
        let mut adj = vec![vec![false; n]; n];

        let nodes: Vec<NodeId> = graph.node_ids().collect();
        for (i, &u_id) in nodes.iter().enumerate() {
            for neighbor_id in graph.neighborhood(u_id) {
                if let Some(j) = nodes.iter().position(|&id| id == neighbor_id) {
                    if i != j {
                        adj[i][j] = true;
                        adj[j][i] = true;
                    }
                }
            }
        }

        Self { adj, n }
    }

    fn edge_count(&self) -> usize {
        let mut count = 0;
        for i in 0..self.n {
            for j in (i + 1)..self.n {
                if self.adj[i][j] {
                    count += 1;
                }
            }
        }
        count
    }

    /// Returns first edge found (u, v) with u < v
    fn find_edge(&self) -> Option<(usize, usize)> {
        for i in 0..self.n {
            for j in (i + 1)..self.n {
                if self.adj[i][j] {
                    return Some((i, j));
                }
            }
        }
        None
    }

    /// Returns first non-edge found (u, v) with u < v
    fn find_non_edge(&self) -> Option<(usize, usize)> {
        for i in 0..self.n {
            for j in (i + 1)..self.n {
                if !self.adj[i][j] {
                    return Some((i, j));
                }
            }
        }
        None
    }

    fn remove_edge(&mut self, u: usize, v: usize) {
        self.adj[u][v] = false;
        self.adj[v][u] = false;
    }

    fn add_edge(&mut self, u: usize, v: usize) {
        self.adj[u][v] = true;
        self.adj[v][u] = true;
    }

    /// Contract edge (u, v). Merges v into u.
    /// Removes vertex v.
    fn contract(&self, u: usize, v: usize) -> Self {
        // Assume u < v to keep indices stable for the first part
        let mut new_adj = Vec::with_capacity(self.n - 1);
        let n = self.n;

        // Map old indices to new indices:
        // 0..v-1 -> same
        // v -> u (merged)
        // v+1..n -> index-1

        // Node `k` in new matrix corresponds to `k` in old if k < v, or `k+1` in old if k >= v.

        for i in 0..n {
            if i == v {
                continue;
            }
            let mut row = Vec::with_capacity(n - 1);
            for j in 0..n {
                if j == v {
                    continue;
                }

                let mut connected = self.adj[i][j];

                // If i is u, check if j was connected to v
                if i == u {
                    if self.adj[v][j] {
                        connected = true;
                    }
                }
                // If j is u, check if i was connected to v
                if j == u {
                    if self.adj[i][v] {
                        connected = true;
                    }
                }

                // Remove self-loops formed by contraction
                if i == u && j == u {
                    connected = false;
                }

                row.push(connected);
            }
            new_adj.push(row);
        }

        Self {
            adj: new_adj,
            n: n - 1,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ChromaticPolynomialMethod {
    /// P(G) = P(G-e) - P(G/e). Best for sparse graphs.
    RemoveEdges,
    /// P(G) = P(G+e) + P(G/e). Best for dense graphs.
    AddEdges,
    /// Automatically choose based on density.
    Auto,
}

pub fn chromatic_polynomial<G>(graph: &G, method: ChromaticPolynomialMethod) -> Polynomial
where
    G: Graph,
{
    let wg = WorkingGraph::from_graph(graph);
    let density = if wg.n > 1 {
        wg.edge_count() as f64 / (wg.n * (wg.n - 1) / 2) as f64
    } else {
        1.0
    };

    let resolved_method = match method {
        ChromaticPolynomialMethod::Auto => {
            if density > 0.6 {
                ChromaticPolynomialMethod::AddEdges
            } else {
                ChromaticPolynomialMethod::RemoveEdges
            }
        }
        m => m,
    };

    match resolved_method {
        ChromaticPolynomialMethod::RemoveEdges => compute_poly_remove(wg),
        ChromaticPolynomialMethod::AddEdges => compute_poly_add(wg),
        _ => unreachable!(),
    }
}

/// Recursive implementation for P(G) = P(G-e) - P(G/e)
fn compute_poly_remove(g: WorkingGraph) -> Polynomial {
    // Base case: Empty graph (no edges)
    // P(E_n) = x^n
    if let Some((u, v)) = g.find_edge() {
        // G_minus: G with edge removed
        let mut g_minus = g.clone();
        g_minus.remove_edge(u, v);

        // G_contract: G with edge contracted
        let g_contract = g.contract(u, v);

        // P(G) = P(G-e) - P(G/e)
        compute_poly_remove(g_minus) - compute_poly_remove(g_contract)
    } else {
        // No edges, return x^n
        Polynomial::from_monomial(g.n, 1)
    }
}

/// Recursive implementation for P(G) = P(G+e) + P(G/e)
fn compute_poly_add(g: WorkingGraph) -> Polynomial {
    // Base case: Complete graph
    // P(K_n) = x(x-1)...(x-n+1)
    if let Some((u, v)) = g.find_non_edge() {
        // G_plus: G with edge added
        let mut g_plus = g.clone();
        g_plus.add_edge(u, v);

        // G_contract: G with (non-edge) contracted
        let g_contract = g.contract(u, v);

        // P(G) = P(G+e) + P(G/e)
        compute_poly_add(g_plus) + compute_poly_add(g_contract)
    } else {
        // Complete graph K_n
        // Result is x(x-1)...(x-n+1)
        let mut poly = Polynomial::one();
        for i in 0..g.n {
            // multiply by (x - i)
            let term = Polynomial {
                coeffs: vec![-(i as i64), 1], // -i + 1*x
            };
            poly = poly * term;
        }
        poly
    }
}

pub fn num_k_colorings<G>(graph: &G, k: i64) -> i64
where
    G: Graph,
{
    let poly = chromatic_polynomial(graph, ChromaticPolynomialMethod::Auto);
    poly.eval(k)
}

pub fn chromatic_number<G>(graph: &G) -> usize
where
    G: Graph,
{
    let poly = chromatic_polynomial(graph, ChromaticPolynomialMethod::Auto);
    let n = graph.order();

    for k in 1..=n {
        if poly.eval(k as i64) > 0 {
            return k;
        }
    }

    if n == 0 { 0 } else { 1 }
}
