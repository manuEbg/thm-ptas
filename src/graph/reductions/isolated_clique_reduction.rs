/* data structure for isolated clique reduction */
use std::collections::HashMap;
use crate::graph::dcel::vertex::VertexId;
use crate::graph::DcelBuilder;
use crate::graph::quick_graph::QuickGraph;
use crate::graph::reducible::Reducible;
use crate::graph::reductions::{remove_vertex_and_update_indices, update_vertex_indices};

#[derive(Debug)]
pub struct IsolatedClique {
    pub(crate) isolated_vertex: usize,
    pub(crate) members: Vec<usize>
}

impl IsolatedClique {
    pub fn reduce_dcel_builder(
        &self,
        dcel_builder: &mut DcelBuilder,
        vertex_ids: &mut HashMap<VertexId, VertexId>,
    ) {
        for &vertex in &self.members {
            remove_vertex_and_update_indices(dcel_builder, vertex, vertex_ids);
        }
    }
}

pub fn do_isolated_clique_reductions(graph: &mut QuickGraph)
                                     -> Vec<IsolatedClique> {

    let mut result: Vec<IsolatedClique> = Vec::new();
    loop {
        /* find an isolated clique */
        if let Some(vertex) = (0..graph.adjacency.len())
            .find(|&vertex| graph.is_isolated_clique(vertex)) {

            /* prepare data for reduction */
            let mut clique: Vec<usize> = graph.adjacency[vertex].clone().unwrap();
            clique.push(vertex);

            /* remove clique from graph */
            clique.iter().for_each(|&member| graph.remove_vertex(member));

            /* add clique to result */
            result.push(IsolatedClique {
                isolated_vertex: vertex,
                members: clique
            });
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
    mut independence_set: &mut Vec<usize>,
    isolated_cliques: &Vec<IsolatedClique>
) {
    independence_set.extend(isolated_cliques.iter().map(|isolated_clique|
        isolated_clique.isolated_vertex)
    );
}