/* data structure for twin reduction */
use crate::graph::quick_graph::QuickGraph;
use crate::graph::reducible::Reducible;
use crate::graph::reductions::utils::*;

pub struct TwinReduction {
    u: usize,
    v: usize,
    neighborhood: Vec<usize>,
    removed_vertices: Vec<usize>,
    remaining_vertex: Option<usize>
}

pub fn do_twin_reductions(graph: &mut QuickGraph) -> Vec<TwinReduction> {
    let mut result: Vec<TwinReduction> = Vec::new();
    loop {
        if let Some((u, v)) = graph.find_twins() {

            /* create twin reduction datastructure */
            let mut twin_reduction = TwinReduction {
                u,
                v,
                neighborhood: graph.adjacency[u].clone(),
                removed_vertices: vec![u, v - 1],
                remaining_vertex: None,
            };

            /* copy neighbors for updating and remove twins */
            let mut current_neighbors = graph.adjacency[u].clone();
            for &twin in &twin_reduction.removed_vertices {
                graph.remove_vertex(twin);
                current_neighbors = decrease_vertices(&current_neighbors, twin);
            }

            /* check if any of the three neighbors are adjacent */
            if graph.adjacency[current_neighbors[0]].contains(&current_neighbors[1]) ||
                graph.adjacency[current_neighbors[0]].contains(&current_neighbors[2]) ||
                graph.adjacency[current_neighbors[1]].contains(&current_neighbors[2]) {


                /* remove all three neighbors */
                for index in 0..current_neighbors.len() {
                    let neighbor = current_neighbors[index];
                    graph.remove_vertex(neighbor);
                    twin_reduction.removed_vertices.push(neighbor);
                    current_neighbors = decrease_vertices(&current_neighbors, neighbor);
                }
            } else {
                for index in 1..current_neighbors.len() {
                    graph.merge_vertices(current_neighbors[0], current_neighbors[index]);
                    twin_reduction.removed_vertices.push(current_neighbors[index]);
                    current_neighbors = decrease_vertices(
                        &current_neighbors,
                        current_neighbors[index]
                    );
                }
                twin_reduction.remaining_vertex = Some(current_neighbors[0]);
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
    independence_set: Vec<usize>
) -> Vec<usize> {
    let mut result: Vec<usize> = independence_set.clone();
    while !reductions.is_empty() {
        let mut reduction: TwinReduction = reductions.pop().unwrap();
        let mut take_neighbors: bool = false;

        /* decide if neighbors or twins should be taken into solution */
        if let Some(vertex) = reduction.remaining_vertex {
            take_neighbors = result.contains(&vertex);
            result.retain(|&result_member| result_member != vertex);
        }

        result = restore_independence_set(result, reduction.removed_vertices);
        if take_neighbors {
            reduction.neighborhood.iter().for_each(|&vertex| result.push(vertex));
        } else {
            result.push(reduction.u);
            result.push(reduction.v);
        }
    }
    result
}