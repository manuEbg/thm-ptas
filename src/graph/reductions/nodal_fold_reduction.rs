/* data structure for nodal fold reduction */
use crate::graph::quick_graph::QuickGraph;
use crate::graph::reducible::Reducible;

pub struct NodalFold {
    inner_vertex: usize,
    neighbors: Vec<usize>
}

pub fn do_nodal_fold_reductions(graph: &mut QuickGraph) -> Vec<NodalFold> {
    let mut result: Vec<NodalFold> = Vec::new();
    loop {
        /* look for a vertex with two nonadjacent neighbors */
        if let Some(vertex) = graph.adjacency.iter()
            .position(|neighborhood| {
                match neighborhood {
                    Some(neighborhood) => {
                        neighborhood.len() == 2 &&
                            !graph.are_adjacent(neighborhood[0], neighborhood[1])
                    },
                    None => false
                }
            }) {

            /* gather all information about the reduction */
            let nodal_fold: NodalFold = NodalFold {
                inner_vertex: vertex,
                neighbors: graph.adjacency[vertex].clone().unwrap()
            };

            /* merge neighbors into inner vertex */
            nodal_fold.neighbors.iter().for_each(
                |&neighbor| graph.merge_vertices(nodal_fold.inner_vertex, neighbor));

            /* add nodal fold to the result */
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
    let mut result = independence_set.clone();
    while let Some(reduction) = reductions.pop() {
        /* decide if the inner vertex or the neighbors should be taken into the solution */
        if result.contains(&reduction.inner_vertex) {
            result.retain(|&vertex| vertex != reduction.inner_vertex);
            reduction.neighbors.iter().for_each(|&neighbor| result.push(neighbor));
        } else {
            result.push(reduction.inner_vertex);
        }
    }
    result
}