/* data structure for nodal fold reduction */
use crate::graph::quick_graph::QuickGraph;
use crate::graph::reducible::Reducible;
use crate::graph::reductions::utils::*;

pub struct NodalFold {
    inner_vertex: usize,
    neighbors: Vec<usize>,
    remaining_vertex: usize
}

pub fn do_nodal_fold_reductions(graph: &mut QuickGraph) -> Vec<NodalFold> {
    let mut result: Vec<NodalFold> = Vec::new();
    loop {
        /* look for a vertex with two nonadjacent neighbors */
        if let Some(vertex) = graph.adjacency.iter()
            .position(|neighborhood| neighborhood.len() == 2
                && !graph.adjacency[neighborhood[0]].contains(&neighborhood[1])) {

            /* gather all information about the reduction */
            let mut nodal_fold: NodalFold = NodalFold {
                inner_vertex: vertex,
                neighbors: graph.adjacency[vertex].clone(),
                remaining_vertex: vertex
            };

            /* collect all concerned vertices */
            let mut concerned_vertices: Vec<usize> = graph.adjacency[vertex].clone();
            concerned_vertices.insert(0, vertex);

            /* merge both neighbors into the inner vertex and update all vertices */
            for index in 1..concerned_vertices.len() {
                graph.merge_vertices(concerned_vertices[0], concerned_vertices[index]);
                concerned_vertices = decrease_vertices(
                    &concerned_vertices, concerned_vertices[index]
                );
            }

            /* update nodal fold and add it to the result */
            nodal_fold.remaining_vertex = concerned_vertices[0];
            result.push(nodal_fold);
        } else {
            break;
        }
    }
    result
}

/* restore solution from solution after nodal fold reductions */
pub fn transfer_nodal_fold_reduction(
    independence_set: Vec<usize>,
    reductions: &mut Vec<NodalFold>
) -> Vec<usize> {

    let mut result: Vec<usize> = independence_set.clone();
    while let Some(mut reduction) = reductions.pop() {

        /* decide if inner vertex or neighbors should be taken into the result */
        let take_neighbors = result.contains(&reduction.remaining_vertex);
        result.retain(|&vector_vertex| vector_vertex != reduction.remaining_vertex);

        /* update result when restoring the original graph */
        let mut removed_vertices: Vec<usize> = vec![reduction.neighbors[0]];
        removed_vertices.push(
            if reduction.neighbors[1] > reduction.neighbors[0] {
                reduction.neighbors[1] - 1
            } else {
                reduction.neighbors[1]
            }
        );
        result = restore_independence_set(result, removed_vertices);

        /* add inner vertex or neighbors to solution */
        if take_neighbors {
            reduction.neighbors.iter().for_each(|&neighbor| result.push(neighbor));
        } else {
            result.push(reduction.inner_vertex);
        }
    }
    result
}