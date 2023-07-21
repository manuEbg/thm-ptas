/* data structure for nodal fold reduction */
use std::collections::HashMap;
use crate::graph::dcel::vertex::VertexId;
use crate::graph::DcelBuilder;
use crate::graph::quick_graph::QuickGraph;
use crate::graph::reducible::Reducible;
use crate::graph::reductions::{merge_vertices_and_update_indices, update_vertex_indices};

#[derive(Debug)]
pub struct NodalFold {
    pub(crate) inner_vertex: usize,
    pub(crate) neighbors: Vec<usize>
}

impl NodalFold {
    pub fn reduce_dcel_builder(
        &self,
        dcel_builder: &mut DcelBuilder,
        vertex_ids: &mut HashMap<usize, usize>
    ) {
        for &neighbor in &self.neighbors {
            merge_vertices_and_update_indices(
                dcel_builder,
                self.inner_vertex,
                neighbor,
                vertex_ids
            );
        }
    }
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
    mut independence_set: Vec<usize>,
    mut reductions: Vec<NodalFold>
) {
    while let Some(reduction) = reductions.pop() {
        /* decide if the inner vertex or the neighbors should be taken into the solution */
        if independence_set.contains(&reduction.inner_vertex) {
            independence_set.retain(|&vertex| vertex != reduction.inner_vertex);
            reduction.neighbors.iter().for_each(|&neighbor| independence_set.push(neighbor));
        } else {
            independence_set.push(reduction.inner_vertex);
        }
    }
}