use gtc::{
    AdjacencyList, AdjacencyMatrix, DirectedGraph, FlowNetwork, GraphDefinition, LatexDisplay,
    LatexVisualDisplay, Simple, UndirectedGraph, dijkstra, warshall_closure,
    warshall_lightest_path_matrix,
};

fn dijkstra_example() {
    let graph =
        DirectedGraph::<GraphDefinition<_, _, _, i32>, _, String, (), (), i32>::from_edges([
            ("s", "u", 10),
            ("s", "x", 5),
            ("u", "x", 2),
            ("x", "u", 3),
            ("u", "v", 1),
            ("x", "v", 9),
            ("x", "y", 2),
            ("v", "y", 4),
            ("y", "v", 6),
            ("y", "s", 7),
        ]);

    println!("Graph:\n{}", graph.to_latex_visual());

    let result = dijkstra(&graph, "s".to_string());
    println!("Dijkstra Result:\n{}", result.to_latex());

    if let Some((weight, path)) = result.lightest_path_to(&"v".to_string()) {
        println!(
            "Lightest path from s to v has weight {}: {:?}",
            weight, path
        );
    } else {
        println!("No path from s to v found.");
    }
}

fn warshall_closure_example() {
    let graph = DirectedGraph::<GraphDefinition<_, _, _, i32>, _, String, _, _, i32>::from_edges([
        ("v1", "v3", 1),
        ("v2", "v1", 1),
        ("v3", "v5", 1),
        ("v4", "v3", 1),
        ("v5", "v1", 1),
        ("v5", "v4", 1),
    ]);

    println!("Graph:\n{}", graph.to_latex_visual());

    let closure = warshall_closure(&graph);
    println!("Reflexive and Transitive Closure:\n{}", closure.to_latex());

    let warshall_path_matrix = warshall_lightest_path_matrix(&graph);
    println!(
        "Warshall Lightest Path Matrix:\n{}",
        warshall_path_matrix.to_latex()
    );
}

fn ford_fulkerson_example() {
    // let flow_network =
    //     FlowNetwork::<AdjacencyList<String, (), (), ()>, Simple, String, (), (), ()>::from_edges(
    //         vec![
    //             ("s", "v1", 11, 16),
    //             ("s", "v2", 8, 13),
    //             ("v1", "v2", 0, 10),
    //             ("v2", "v1", 1, 4),
    //             ("v1", "v3", 12, 12),
    //             ("v3", "v2", 4, 9),
    //             ("v2", "v4", 11, 14),
    //             ("v4", "v3", 7, 7),
    //             ("v3", "t", 15, 20),
    //             ("v4", "t", 4, 4),
    //         ],
    //         "s",
    //         "t",
    //     );

    // let flow_network =
    //     FlowNetwork::<AdjacencyList<String, (), (), ()>, Simple, String, (), (), ()>::from_edges(
    //         vec![
    //             ("s", "b", 0, 12),
    //             ("s", "e", 0, 15),
    //             ("s", "g", 0, 13),
    //             ("b", "c", 0, 9),
    //             ("e", "c", 0, 11),
    //             ("c", "d", 0, 18),
    //             ("c", "f", 0, 10),
    //             ("f", "d", 0, 6),
    //             ("f", "t", 0, 20),
    //             ("d", "t", 0, 12),
    //             ("g", "h", 0, 12),
    //             ("h", "e", 0, 8),
    //             ("h", "f", 0, 6),
    //             ("h", "t", 0, 10),
    //         ],
    //         "s",
    //         "t",
    //     );

    let flow_network =
        FlowNetwork::<AdjacencyList<String, (), (), ()>, Simple, String, (), (), ()>::from_edges(
            vec![
                ("s", "a", 0, 23),
                ("s", "b", 16, 17),
                ("s", "c", 14, 41),
                ("b", "a", 14, 31),
                ("c", "b", 0, 24),
                ("a", "u", 14, 24),
                ("b", "u", 1, 15),
                ("b", "v", 15, 32),
                ("c", "w", 14, 14),
                ("w", "b", 14, 15),
                ("w", "v", 0, 12),
                ("u", "v", 1, 25),
                ("u", "t", 14, 56),
                ("v", "t", 16, 16),
            ],
            "s",
            "t",
        );

    println!("Flow Network:\n{}", flow_network.to_latex_visual());

    let result = gtc::ford_fulkerson(flow_network);
    println!("FF Result: {}", result.to_latex());
}

fn kruskal_example() {
    let graph =
        UndirectedGraph::<GraphDefinition<_, _, _, i32>, _, String, _, _, i32>::from_edges([
            ("a", "b", 7),
            ("a", "c", 1),
            ("a", "d", 4),
            ("b", "e", 6),
            ("b", "f", 5),
            ("c", "d", 1), // Note: Weight is missing in the image; assumed 1
            ("c", "g", 3),
            ("d", "h", 2),
            ("e", "f", 6),
            ("e", "h", 5),
            ("e", "i", 3),
            ("f", "i", 4),
            ("f", "l", 7),
            ("g", "h", 5),
            ("g", "j", 3),
            ("h", "j", 4),
            ("i", "k", 7),
            ("j", "k", 8),
            ("k", "l", 6),
        ]);

    let mst = gtc::kruskal_mst(&graph);
    println!("Kruskal MST Result:\n{}", mst.to_latex());
    println!("Kruskal MST Result:\n{}", mst.to_latex_visual());
}

fn prufer_to_tree_example() {
    let sequence = vec![4, 3, 1, 3, 1];
    let tree = gtc::prufer_to_tree(&sequence);

    let undirected_graph: UndirectedGraph<GraphDefinition<usize>, Simple, usize> =
        UndirectedGraph::new(tree);

    println!("Prufer Sequence: {:?}", sequence);
    println!("Reconstructed Tree:\n{}", undirected_graph.to_latex());
}

fn prufer_from_tree_example() {
    let graph = UndirectedGraph::<GraphDefinition<i32>, Simple, i32>::from_edges([
        (4, 5),
        (4, 2),
        (4, 7),
        (3, 7),
        (1, 7),
        (6, 7),
    ]);

    let prufer_sequence = gtc::tree_to_prufer(&graph);
    println!("Input Tree:\n{}", graph.to_latex());
    println!("Prufer Sequence: {:?}", prufer_sequence);
}

fn coloring_example() {
    let graph = UndirectedGraph::<GraphDefinition<i32>, Simple, i32>::from_edges([
        (7, 8),
        (7, 2),
        (8, 1),
        (1, 6),
        (8, 2),
        (2, 5),
        (2, 3),
        (3, 5),
        (4, 3),
    ]);

    let chromatic_number = gtc::chromatic_number(&graph);
    let chromatic_polynomial =
        gtc::chromatic_polynomial(&graph, gtc::ChromaticPolynomialMethod::Auto);
    let num_2_colorings = chromatic_polynomial.eval(2);
    let num_3_colorings = chromatic_polynomial.eval(3);

    println!("Graph:\n{}", graph.to_latex());
    println!("Chromatic Number: {}", chromatic_number);
    println!("Chromatic Polynomial: {}", chromatic_polynomial);
    println!("Number of 2-Colorings: {}", num_2_colorings);
    println!("Number of 3-Colorings: {}", num_3_colorings);
}

fn graph_distances_example() {
    let graph =
        UndirectedGraph::<GraphDefinition<String, _, _, i32>, Simple, _, _, _, i32>::from_edges([
            ("a", "b", 1),
            ("a", "g", 1),
            ("b", "c", 1),
            ("b", "e", 1),
            ("c", "f", 1),
            ("e", "f", 1),
            ("e", "h", 1),
            ("f", "g", 1),
            ("c", "d", 1),
            ("h", "d", 1),
            ("g", "h", 1),
        ]);

    let matrix = gtc::warshall_lightest_path_matrix(&graph);
    let distances = gtc::compute_graph_distances(&matrix);
    println!("Graph:\n{}", graph.to_latex());
    println!("Graph Distances:\n{}", distances.to_latex());
}

fn main() {
    // ford_fulkerson_example();
    // prufer_to_tree_example();
    // prufer_from_tree_example();
    // coloring_example();
    graph_distances_example();
    // kruskal_example();
    // dijkstra_example();
    // warshall_closure_example();

    // let k5 = UndirectedGraph::complete(5);
    // println!("{}", k5.to_string());

    // let adjacency_matrix = AdjacencyMatrix::from(&k5);
    // println!("{}", adjacency_matrix.to_string());

    // println!("Graph:\n{}", graph.to_latex());

    // let matrix_graph = DirectedGraph::<AdjacencyMatrix>::converted_storage(&graph);

    // println!("Adjacency Matrix:\n{}", matrix_graph.to_latex());

    // let adjacency_matrix = AdjacencyMatrix::from(&graph);
    // println!("Adjacency Matrix:\n{}", adjacency_matrix);

    // println!("Paths of length 2: \n{}", adjacency_matrix.pow(2));

    // println!(
    //     "Transitive Closure via Matrix Exponentiation:\n{}",
    //     adjacency_matrix.transitive_closure_via_matrix_exponentiation()
    // );

    // println!(
    //     "Transitive Closure via Warshall Algorithm:\n{}",
    //     adjacency_matrix.transitive_closure_via_warshall_algorithm()
    // );
}
