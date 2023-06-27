use crate::graph::quick_graph::QuickGraph;
use crate::graph::reducible::Reducible;

pub type MergeReduction = (Vec<Vec<usize>>, usize, usize);

fn merge_vertices_reversible(graph: &mut QuickGraph, u: usize, v: usize) -> MergeReduction {
    let copy: Vec<Vec<usize>> = graph.adjacency.clone();
    graph.merge_vertices(u, v);
    (copy, u, v)
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

pub fn transfer_independence_set(
    independence_set: Vec<usize>,
    reductions: &mut Vec<MergeReduction>
) -> Vec<usize>{
    let mut result: Vec<usize> = independence_set.clone();
    while let Some((second, first)) =
        reductions.pop().zip(reductions.pop()) {
        let (_, _, second_neighbor) = second;
        let (_, original_vertex, first_neighbor) = first;
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

