/* data structure for twin reduction */
use std::collections::HashMap;
use crate::graph::dcel::vertex::VertexId;
use crate::graph::DcelBuilder;
use crate::graph::quick_graph::QuickGraph;
use crate::graph::reducible::Reducible;
use crate::graph::reductions::{merge_vertices_and_update_indices, remove_vertex_and_update_indices, update_vertex_indices};

#[derive(Debug)]
pub struct TwinReduction {
    pub(crate) u: usize,
    pub(crate) v: usize,
    pub(crate) neighborhood: Vec<usize>,
    pub(crate) adjacent_neighbors: bool
}

impl TwinReduction {
    pub fn reduce_dcel_builder(
        &self,
        dcel_builder: &mut DcelBuilder,
        vertex_ids: &mut HashMap<VertexId, VertexId>) {
        if self.adjacent_neighbors {
            /* remove twins and neighbors */
            remove_vertex_and_update_indices(dcel_builder, self.u, vertex_ids);
            remove_vertex_and_update_indices(dcel_builder, self.v, vertex_ids);
            for &neighbor in &self.neighborhood {
                remove_vertex_and_update_indices(dcel_builder, neighbor, vertex_ids);
            }
        } else {
            /* merge u into remaining vertex */
            merge_vertices_and_update_indices(
                dcel_builder,
                self.neighborhood[0],
                self.u,
                vertex_ids
            );

            /* merge one neighbor of the twins into the remaining vertex */
            merge_vertices_and_update_indices(
                dcel_builder,
                self.neighborhood[0],
                self.neighborhood[1],
                vertex_ids
            );

            /* merge v */
            merge_vertices_and_update_indices(
                dcel_builder,
                self.neighborhood[0],
                self.v,
                vertex_ids
            );

            /* merge the remaining neighbor into the remaining vertex */
            merge_vertices_and_update_indices(
                dcel_builder,
                self.neighborhood[0],
                self.neighborhood[2],
                vertex_ids
            );
        }
    }
}

pub fn do_twin_reductions(graph: &mut QuickGraph) -> Vec<TwinReduction> {
    let mut result: Vec<TwinReduction> = Vec::new();
    loop {
        if let Some((u, v)) = graph.find_twins() {

            /* create twin reduction datastructure */
            let neighbors = graph.adjacency[u].clone().unwrap();
            let twin_reduction = TwinReduction {
                u,
                v,
                neighborhood: neighbors.clone(),
                adjacent_neighbors: graph.are_adjacent_triple(
                    neighbors[0],
                    neighbors[1],
                    neighbors[2]
                )
            };

            /* remove twins from graph */
            graph.remove_vertex(u);
            graph.remove_vertex(v);

            /* handle neighbors depending from adjacency */
            if twin_reduction.adjacent_neighbors {
                /* remove all five concerned vertices */
                twin_reduction.neighborhood.iter().for_each(|&neighbor| {
                   graph.remove_vertex(neighbor);
                });
            } else {
                /* merge two of the three neighbors */
                graph.merge_vertices(
                    twin_reduction.neighborhood[0],
                    twin_reduction.neighborhood[1]
                );

                /* merge the third neighbor */
                graph.merge_vertices(
                    twin_reduction.neighborhood[0],
                    twin_reduction.neighborhood[2]
                )
            }

            result.push(twin_reduction);
        } else {
            break;
        }
    }
    result
}

pub fn transfer_twin_reductions(
    reductions: &mut Vec<TwinReduction>,
    mut independence_set: &mut Vec<usize>
) {
    while let Some(reduction) = reductions.pop() {
        /* decide which vertices should be taken into the solution */
        if reduction.adjacent_neighbors || !independence_set.contains(&reduction.neighborhood[0]){
            independence_set.push(reduction.u);
            independence_set.push(reduction.v);
        } else {
            for index in 1..reduction.neighborhood.len() {
                independence_set.push(reduction.neighborhood[index]);
            }
        }
    }
}