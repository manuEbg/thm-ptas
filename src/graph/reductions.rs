use crate::graph::quick_graph::QuickGraph;
use crate::graph::reducible::Reducible;

#[derive(Debug)]
pub struct MergeReduction {
    old_graph: Vec<Vec<usize>>,
    u: usize,
    v: usize
}

#[derive(Debug)]
pub struct RemoveReduction {
    vertex: usize,
    neighborhood: Vec<usize>
}

pub struct IsolatedClique {
    isolated_vertex: usize,
    removed_vertices: Vec<RemoveReduction>
}

#[derive(Debug)]
enum ReduceOperation {
    MergeReduction(MergeReduction),
    RemoveReduction(RemoveReduction)
}

#[derive(Debug)]
pub struct TwinReduction {
    u: usize,
    v: usize,
    neighborhood: Vec<usize>,
    reductions: Vec<ReduceOperation>,
    remaining_vertex: Option<usize>
}

fn merge_vertices_reversible(graph: &mut QuickGraph, u: usize, v: usize) -> MergeReduction {
    let copy: Vec<Vec<usize>> = graph.adjacency.clone();
    graph.merge_vertices(u, v);
    MergeReduction {old_graph: copy, u, v}
}

fn remove_vertex_reversible(graph: &mut QuickGraph, u: usize) -> RemoveReduction {
    let neighborhood: Vec<usize> = graph.adjacency[u].clone();
    graph.remove_vertex(u);
    RemoveReduction {vertex: u, neighborhood}
}

pub fn do_vertice_fold_reduction(graph: &mut QuickGraph) -> Vec<MergeReduction> {
    let mut result: Vec<MergeReduction> = Vec::new();
    loop {
        if let Some(vertex) = graph.adjacency.iter()
            .position(|neighborhood| neighborhood.len() == 2
            && !graph.adjacency[neighborhood[0]].contains(&neighborhood[1])) {
            let neighborhood: Vec<usize> = graph.neighborhood(vertex).clone();
            result.push(merge_vertices_reversible(graph, vertex, neighborhood[0]));
            let updated_vertex: usize = if vertex > neighborhood[0] { vertex - 1} else {vertex};
            let second_neighbor: usize =
                if neighborhood[1] > neighborhood[0] {neighborhood[1] - 1} else {neighborhood[1]};
            result.push(merge_vertices_reversible(
                graph,
                updated_vertex,
                second_neighbor
            ));
        } else {
            break;
        }
    }
    result
}

pub fn transfer_independence_set_vertex_fold(
    independence_set: Vec<usize>,
    reductions: &mut Vec<MergeReduction>
) -> Vec<usize>{
    let mut result: Vec<usize> = independence_set.clone();
    while let Some((second, first)) =
        reductions.pop().zip(reductions.pop()) {
        let second_neighbor = second.v;
        let original_vertex = first.u;
        let first_neighbor = first.v;
        result = result.iter().map(|&vertex| {
            if vertex >= second_neighbor {vertex + 1} else {vertex}
        }).map(|vertex| {
            if vertex >= first_neighbor {vertex + 1} else {vertex}
        }).collect();
        let second_neighbor: usize =
            if second_neighbor >= first_neighbor { second_neighbor + 1} else {second_neighbor};
        if result.contains(&original_vertex) {
            result.retain(|&vertex| vertex != original_vertex);
            result.push(first_neighbor);
            result.push(second_neighbor);
        } else {
            result.push(original_vertex);
        }
    }
    result
}

fn is_isolated_clique(graph: &QuickGraph, vertex: usize) -> bool {
    let neighborhood = &graph.adjacency[vertex];

    for &neighbor in neighborhood {
        let neighbors_neighbors = &graph.adjacency[neighbor];
        if !neighborhood.iter()
            .filter(|&&v| v != neighbor)
            .all(|v| neighbors_neighbors
                .contains(v)) {
            return false;
        }
    }
    true
}

fn decrease_neighborhood(neighborhood: &Vec<usize>, vertex: usize) -> Vec<usize> {
    neighborhood.iter().
        map(|&neighbor| if neighbor > vertex { neighbor - 1} else {neighbor} ).
        collect()
}

pub fn do_isolated_clique_reductions(graph: &mut QuickGraph)
    -> Vec<IsolatedClique> {
    let mut result: Vec<IsolatedClique> = Vec::new();
    loop {
        if let Some(vertex) = (0..graph.num_vertices())
            .find(|&vertex| is_isolated_clique(graph, vertex)) {

            let mut neighborhood: Vec<usize> = graph.adjacency[vertex].clone();
            let mut isolated_clique = IsolatedClique {
                isolated_vertex: vertex,
                removed_vertices: Vec::new()
            };
            isolated_clique.removed_vertices.push(remove_vertex_reversible(graph, vertex));
            neighborhood = decrease_neighborhood(&neighborhood, vertex);

            while !neighborhood.is_empty() {
                let neighbor = neighborhood.pop().unwrap();
                isolated_clique.removed_vertices.push(
                    remove_vertex_reversible(graph, neighbor)
                );
                neighborhood = decrease_neighborhood(&neighborhood, vertex);
            }
            result.push(isolated_clique);
        } else {
            break;
        }
    }
    result
}


pub fn transfer_independence_set_isolated_clique(
    graph: &mut QuickGraph,
    isolated_cliques: &mut Vec<IsolatedClique>,
    independence_set: Vec<usize>) -> Vec<usize> {
    let mut result = independence_set.clone();
    while !isolated_cliques.is_empty() {
        let mut next_isolated_clique = isolated_cliques.pop().unwrap();
        while !next_isolated_clique.removed_vertices.is_empty() {
            let reduction = next_isolated_clique.removed_vertices.pop().unwrap();
            graph.insert_vertex(reduction.vertex, reduction.neighborhood);
            result = result.iter().map(
                |&vertex| if vertex >= reduction.vertex { vertex + 1} else {vertex}).collect();
        }
        result.push(next_isolated_clique.isolated_vertex);
    }
    result
}

pub fn do_twin_reductions(graph: &mut QuickGraph) -> Vec<TwinReduction> {
    let mut result: Vec<TwinReduction> = Vec::new();
    loop {
        if let Some((u, v)) = graph.find_twins() {
            let original_neighborhood = graph.adjacency[u].clone();
            let mut current_neighbors = original_neighborhood.clone();
            let mut reductions: Vec<ReduceOperation> = Vec::new();
            reductions.push(
              ReduceOperation::RemoveReduction(remove_vertex_reversible(graph, u))
            );
            let current_v = v - 1;
            current_neighbors = decrease_neighborhood(&current_neighbors, u);
            reductions.push(
                ReduceOperation::RemoveReduction(remove_vertex_reversible(graph, current_v))
            );
            current_neighbors = decrease_neighborhood(&current_neighbors, current_v);
            let remaining_vertex: Option<usize>;
            if graph.adjacency[current_neighbors[0]].contains(&current_neighbors[1]) ||
                graph.adjacency[current_neighbors[0]].contains(&current_neighbors[2]) ||
                graph.adjacency[current_neighbors[1]].contains(&current_neighbors[2]) {

                for index in 0..current_neighbors.len() {
                    let neighbor = current_neighbors[index];
                    reductions.push(
                        ReduceOperation::RemoveReduction(
                            remove_vertex_reversible(graph, neighbor)
                        )
                    );
                    current_neighbors = decrease_neighborhood(
                        &current_neighbors,
                        neighbor
                    );
                }
                remaining_vertex = None;
            } else {
                reductions.push(
                    ReduceOperation::MergeReduction(merge_vertices_reversible(
                        graph,
                        current_neighbors[0],
                        current_neighbors[1]
                    ))
                );
                current_neighbors = decrease_neighborhood(
                    &current_neighbors,
                    current_neighbors[1]
                );
                reductions.push(
                    ReduceOperation::MergeReduction(merge_vertices_reversible(
                        graph,
                        current_neighbors[0],
                        current_neighbors[2]
                    ))
                );
                current_neighbors = decrease_neighborhood(
                    &current_neighbors,
                    current_neighbors[2]
                );
                remaining_vertex = Some(current_neighbors[0]);
            }
            result.push(TwinReduction {
                u,
                v,
                neighborhood: original_neighborhood,
                reductions,
                remaining_vertex
            });
        } else {
            break;
        }
    }
    result
}