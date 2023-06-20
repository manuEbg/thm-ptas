use crate::graph::quick_graph::QuickGraph;
use crate::graph::reducible::Reducible;

pub fn do_vertex_fold_reductions(graph: &mut QuickGraph) -> Vec<(usize, usize, usize)> {
    let mut result: Vec<(usize, usize, usize)> = Vec::new();
    loop {
        if let Some(vertex) = graph.adjacency.iter()
            .position(|neighborhood| neighborhood.len() == 2) {
            let neighborhood: Vec<usize> = graph.neighborhood(vertex).to_vec();
            graph.merge_vertices(vertex, neighborhood[0]);
            let updated_vertex = if vertex > neighborhood[0] { vertex - 1} else {vertex};
            let second_neighbor
                = if neighborhood[1] > neighborhood[0] {neighborhood[1] - 1} else {neighborhood[1]};
            graph.merge_vertices(updated_vertex, second_neighbor);
            result.push((vertex, neighborhood[0], second_neighbor));
        } else {
            break;
        }
    }
    result
}