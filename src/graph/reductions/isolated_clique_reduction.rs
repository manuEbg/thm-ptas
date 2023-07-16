/* data structure for isolated clique reduction */
use std::panic::resume_unwind;
use crate::graph::quick_graph::QuickGraph;
use crate::graph::reducible::Reducible;
use crate::graph::reductions::utils;
use crate::graph::reductions::utils::*;

pub struct IsolatedClique {
    isolated_vertex: usize,
    removed_vertices: Vec<usize>
}

pub fn do_isolated_clique_reductions(graph: &mut QuickGraph)
                                     -> Vec<IsolatedClique> {

    let mut result: Vec<IsolatedClique> = Vec::new();
    loop {

        /* find an isolated clique */
        if let Some(vertex) = (0..graph.adjacency.len())
            .find(|&vertex| graph.is_isolated_clique(vertex)) {

            /* prepare data for reduction */
            let mut clique: Vec<usize> = graph.adjacency[vertex].clone();
            clique.push(vertex);
            let mut isolated_clique = IsolatedClique {
                isolated_vertex: vertex,
                removed_vertices: Vec::new()
            };

            /* remove clique from graph */
            while !clique.is_empty() {
                let removed_vertex = clique.pop().unwrap();
                graph.remove_vertex(removed_vertex);
                isolated_clique.removed_vertices.push(removed_vertex);
                clique = decrease_vertices(&clique, removed_vertex);
            }

            /* add clique to result */
            result.push(isolated_clique);
        } else {
            break;
        }
    }
    result
}

/*
restore solution for the original graph from the
solution for the graph after isolated clique reductions
 */

pub fn transfer_isolated_clique(
    isolated_cliques: &mut Vec<IsolatedClique>,
    independence_set: Vec<usize>
) -> Vec<usize> {
    let mut result = independence_set.clone();
    while !isolated_cliques.is_empty() {
        let mut isolated_clique = isolated_cliques.pop().unwrap();
        result = restore_independence_set(result, isolated_clique.removed_vertices);
        result.push(isolated_clique.isolated_vertex);
    }
    result
}