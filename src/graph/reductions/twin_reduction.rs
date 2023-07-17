/* data structure for twin reduction */
use crate::graph::quick_graph::QuickGraph;
use crate::graph::reducible::Reducible;

pub struct TwinReduction {
    u: usize,
    v: usize,
    neighborhood: Vec<usize>,
    adjacent_neighbors: bool
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
    independence_set: Vec<usize>
) -> Vec<usize> {
    let mut result: Vec<usize> = independence_set.clone();
    while let Some(reduction) = reductions.pop() {
        /* decide which vertices should be taken into the solution */
        if reduction.adjacent_neighbors || !result.contains(&reduction.neighborhood[0]){
            result.push(reduction.u);
            result.push(reduction.v);
        } else {
            for index in 1..reduction.neighborhood.len() {
                result.push(reduction.neighborhood[index]);
            }
        }
    }
    result
}