use std::{collections::HashMap, f64::consts::PI};

pub struct VisualGraphData {
    pub labels: Vec<String>,
    pub edges: Vec<VisualEdge>,
    pub is_directed: bool,
}

pub struct VisualEdge {
    pub u: usize,
    pub v: usize,
    pub label: Option<String>,
}

// --- Core Function ---

pub fn generate_latex_graph(data: VisualGraphData) -> String {
    let n = data.labels.len();
    if n == 0 {
        return "\\begin{figure}[htbp]\\begin{tikzpicture}\n% empty graph\n\\end{tikzpicture}\\end{figure}".to_string();
    }

    // --- Helper: Latex Escaping ---
    let escape_latex = |s: &str| -> String {
        s.replace('\\', "\\textbackslash{}")
            .replace('%', "\\%")
            .replace('&', "\\&")
            .replace('#', "\\#")
            .replace('_', "\\_")
            .replace('{', "\\{")
            .replace('}', "\\}")
            .replace('$', "\\$")
            .replace('^', "\\^")
    };

    // --- Physics Simulation (Force-Directed Layout) ---
    #[derive(Clone, Copy)]
    struct Point {
        x: f64,
        y: f64,
    }

    // 1. Initialize positions in a circle
    let radius = (n as f64).sqrt() * 2.0;
    let mut pos: Vec<Point> = (0..n)
        .map(|i| {
            let angle = 2.0 * PI * (i as f64) / (n as f64);
            Point {
                x: radius * angle.cos(),
                y: radius * angle.sin(),
            }
        })
        .collect();

    // 2. Build adjacency for physics (treat everything as undirected attraction)
    let mut adj: Vec<Vec<usize>> = vec![vec![]; n];
    for edge in &data.edges {
        if edge.u != edge.v {
            adj[edge.u].push(edge.v);
            adj[edge.v].push(edge.u);
        }
    }

    // 3. Physics Constants
    let width = (n as f64).sqrt() * 5.0;
    let k_opt = (width * width / (n as f64)).sqrt();
    let iterations = 100;
    let mut temp = width / 10.0;

    // 4. Simulation Loop
    for _ in 0..iterations {
        let mut disp = vec![Point { x: 0.0, y: 0.0 }; n];

        // Repulsive forces
        for v in 0..n {
            for u in 0..n {
                if u != v {
                    let dx = pos[v].x - pos[u].x;
                    let dy = pos[v].y - pos[u].y;
                    let dist = (dx * dx + dy * dy).sqrt().max(0.01);
                    let force = (k_opt * k_opt) / dist;
                    disp[v].x += (dx / dist) * force;
                    disp[v].y += (dy / dist) * force;
                }
            }
        }

        // Attractive forces
        for v in 0..n {
            for &u in &adj[v] {
                let dx = pos[v].x - pos[u].x;
                let dy = pos[v].y - pos[u].y;
                let dist = (dx * dx + dy * dy).sqrt().max(0.01);
                let force = (dist * dist) / k_opt;
                disp[v].x -= (dx / dist) * force;
                disp[v].y -= (dy / dist) * force;
            }
        }

        // Apply displacement
        for v in 0..n {
            let dx = disp[v].x;
            let dy = disp[v].y;
            let dist = (dx * dx + dy * dy).sqrt().max(0.01);
            let move_dist = dist.min(temp);
            pos[v].x += (dx / dist) * move_dist;
            pos[v].y += (dy / dist) * move_dist;
        }
        temp *= 0.95;
    }

    // --- Generate Nodes ---
    let mut nodes_tex = String::new();
    for i in 0..n {
        let label = escape_latex(&data.labels[i]);
        nodes_tex.push_str(&format!(
            "  \\node[main node] (n{}) at ({:.3},{:.3}) {{{}}};\n",
            i, pos[i].x, pos[i].y, label
        ));
    }

    // --- Generate Edges ---
    let mut edges_tex = String::new();
    let base_style = "draw opacity=1, line width=0.8pt";
    let arrow_style = if data.is_directed { "->" } else { "-" };

    // Grouping logic
    let mut pair_groups: HashMap<(usize, usize), (Vec<&VisualEdge>, Vec<&VisualEdge>)> =
        HashMap::new();

    for edge in &data.edges {
        let u = edge.u;
        let v = edge.v;
        if data.is_directed {
            if u <= v {
                pair_groups.entry((u, v)).or_default().0.push(edge);
            } else {
                pair_groups.entry((v, u)).or_default().1.push(edge);
            }
        } else {
            // Undirected: always normalize to min->max
            let (min, max) = if u < v { (u, v) } else { (v, u) };
            pair_groups.entry((min, max)).or_default().0.push(edge);
        }
    }

    for ((u, v), (mut forward, backward)) in pair_groups {
        // Case: Self Loops
        if u == v {
            forward.extend(backward);
            for (i, edge) in forward.iter().enumerate() {
                let angle_step = 30;
                let out_angle = 45
                    + (i as isize
                        * if i % 2 == 0 { 1 } else { -1 }
                        * (i / 2 + 1) as isize
                        * angle_step);
                let in_angle = out_angle + 90;
                let w_lbl = edge
                    .label
                    .as_ref()
                    .map(|l| format!("node[midway, above, font=\\tiny] {{{}}}", escape_latex(l)))
                    .unwrap_or_default();

                edges_tex.push_str(&format!(
                    "  \\draw[{}, {}, looseness=10] (n{}) to[out={}, in={}] {} (n{});\n",
                    arrow_style, base_style, u, out_angle, in_angle, w_lbl, v
                ));
            }
            continue;
        }

        // Determine flow flags BEFORE moving vectors or calling closures
        let has_forward = !forward.is_empty();
        let has_backward = !backward.is_empty();

        // Helper to draw a batch (takes reference to avoid move)
        let mut draw_batch =
            |batch: &Vec<&VisualEdge>, from: usize, to: usize, is_opposed_flow: bool| {
                let count = batch.len();
                for (i, edge) in batch.iter().enumerate() {
                    let bend_str = if data.is_directed && is_opposed_flow {
                        // Bidirectional traffic: strict left bend (eye shape)
                        let base = 15.0;
                        let spread = 15.0;
                        format!("bend left={:.1}", base + (i as f64 * spread))
                    } else {
                        // One-way traffic (or undirected): Fan out symmetrically
                        if count == 1 {
                            "bend left=0".to_string()
                        } else {
                            let spread = 30.0;
                            let step = spread / (count as f64 - 1.0);
                            let angle = -spread / 2.0 + (i as f64 * step);
                            format!("bend left={:.1}", angle)
                        }
                    };

                    let lbl_pos = if bend_str.contains("-") && !is_opposed_flow {
                        "below"
                    } else {
                        "above"
                    };
                    let w_lbl = edge
                        .label
                        .as_ref()
                        .map(|l| {
                            format!(
                                "node[midway, sloped, {}, font=\\small] {{{}}}",
                                lbl_pos,
                                escape_latex(l)
                            )
                        })
                        .unwrap_or_default();

                    edges_tex.push_str(&format!(
                        "  \\draw[{}, {}, {}] (n{}) to {} (n{});\n",
                        arrow_style, base_style, bend_str, from, w_lbl, to
                    ));
                }
            };

        // Draw forward (u -> v)
        // We pass `has_backward` to know if we need to bend to avoid the other direction
        draw_batch(&forward, u, v, has_backward);

        if data.is_directed {
            // Draw backward (v -> u)
            // We pass `has_forward` (calculated earlier) to check for opposition
            draw_batch(&backward, v, u, has_forward);
        }
    }

    format!(
        "\\begin{{figure}}[htbp]\\begin{{tikzpicture}}[>=latex, auto]\n\
         \\tikzstyle{{main node}}=[circle, draw, fill=white, font=\\sffamily\\bfseries, minimum size=20pt, inner sep=2pt]\n\
         % Nodes\n\
         {}\n\
         % Edges\n\
         {}\n\
         \\end{{tikzpicture}}\\end{{figure}}",
        nodes_tex, edges_tex
    )
}
