/* data structure for isolated clique reduction */
use crate::graph::quick_graph::QuickGraph;
use crate::graph::reducible::Reducible;

pub fn do_isolated_clique_reductions(graph: &mut QuickGraph)
                                     -> Vec<usize> {

    let mut result: Vec<usize> = Vec::new();
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
            result.push(vertex);
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
    isolated_cliques: Vec<usize>,
    independence_set: Vec<usize>
) -> Vec<usize> {
    let mut result = independence_set.clone();
    result.extend(isolated_cliques.iter());
    result
}