use std::collections::HashMap;
use crate::graph::dcel::vertex::VertexId;
use crate::graph::DcelBuilder;
use crate::graph::quick_graph::QuickGraph;
use crate::graph::reducible::Reducible;
use crate::graph::reductions::isolated_clique_reduction::{do_isolated_clique_reductions, IsolatedClique, transfer_isolated_clique};
use crate::graph::reductions::nodal_fold_reduction::{do_nodal_fold_reductions, NodalFold, transfer_nodal_fold_reductions};
use crate::graph::reductions::twin_reduction::{do_twin_reductions, transfer_twin_reductions, TwinReduction};

pub mod nodal_fold_reduction;
pub mod isolated_clique_reduction;
pub mod twin_reduction;

#[derive(Debug)]
pub struct Reductions {
    isolated_cliques: Vec<IsolatedClique>,
    twin_reductions: Vec<TwinReduction>,
    nodal_fold_reductions: Vec<NodalFold>
}

pub trait ApplicableReduction {
    fn reduce_dcel_builder(&self,
                           dcel_builder: &mut DcelBuilder,
                           vertex_indices: &mut HashMap<VertexId, VertexId>
    );
}

impl Reductions {
    pub fn reduce_quick_graph(mut quick_graph: &mut QuickGraph) -> Self {
        let isolated_cliques: Vec<IsolatedClique> = do_isolated_clique_reductions(&mut quick_graph);
        let twin_reductions: Vec<TwinReduction> = do_twin_reductions(&mut quick_graph);
        let nodal_fold_reductions: Vec<NodalFold> = do_nodal_fold_reductions(&mut quick_graph);
        Reductions {isolated_cliques, twin_reductions, nodal_fold_reductions}
    }

    pub fn reduce_dcel_builder(&self, mut dcel_builder: &mut DcelBuilder) -> HashMap<VertexId, VertexId> {
        /* initialize map with vertex indices */
        let mut result: HashMap<VertexId, VertexId> = HashMap::new();
        for vertex in 0..dcel_builder.vertex_count() {
            result.insert(vertex, vertex);
        }

        /* do isolated clique reductions */
        for isolated_clique in &self.isolated_cliques {
            isolated_clique.reduce_dcel_builder(&mut dcel_builder, &mut result);
        }

        /* do twin reductions */
        for twin_reduction in &self.twin_reductions {
            twin_reduction.reduce_dcel_builder(&mut dcel_builder, &mut result);
        }

        /* do nodal fold reductions */
        for nodal_fold_reduction in &self.nodal_fold_reductions {
            nodal_fold_reduction.reduce_dcel_builder(&mut dcel_builder, &mut result);
        }
        result
    }

    pub fn transfer_independence_set(
        &mut self,
        vertex_indices: &HashMap<VertexId, VertexId>,
        mut independence_set: &mut Vec<VertexId>
    ) {
        /* restore old vertices in independence_set */
        let mut inverse_vertex_indices: HashMap<VertexId, VertexId> = HashMap::new();
        vertex_indices.iter().for_each(|(&original_index, &recent_index)| {
           inverse_vertex_indices.insert(recent_index, original_index);
        });
        for index in 0..independence_set.len() {
            independence_set[index] = inverse_vertex_indices[&independence_set[index]];
        }

        /* transfer solutions for reductions in reversed order */
        transfer_nodal_fold_reductions(&mut independence_set, &mut self.nodal_fold_reductions);
        transfer_twin_reductions(&mut independence_set, &mut self.twin_reductions);
        transfer_isolated_clique(&mut independence_set, &self.isolated_cliques);
    }
}


pub fn update_vertex_indices(
    vertex_indices: &mut HashMap<VertexId, VertexId>,
    removed_vertex: VertexId) {

    vertex_indices.retain(|_, value| {
        *value != removed_vertex
    });
    vertex_indices.iter_mut().for_each(|(key, value)| {
       *value = DcelBuilder::decrease_index(*value, &vec![removed_vertex])
    });
}

pub fn remove_vertex_and_update_indices(
    dcel_builder: &mut DcelBuilder,
    vertex: VertexId,
    vertex_indices: &mut HashMap<VertexId, VertexId>) {

    let removed_vertex = vertex_indices[&vertex];
    dcel_builder.remove_vertex(removed_vertex);
    update_vertex_indices(vertex_indices, removed_vertex);
}

pub fn merge_vertices_and_update_indices(
    dcel_builder: &mut DcelBuilder,
    u: VertexId,
    v: VertexId,
    vertex_indices: &mut HashMap<VertexId, VertexId>
) {
    let updated_u = vertex_indices[&u];
    let updated_v = vertex_indices[&v];
    dcel_builder.merge_vertices(updated_u, updated_v);
    update_vertex_indices(vertex_indices, updated_v);
}