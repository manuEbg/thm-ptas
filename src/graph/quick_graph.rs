use crate::graph::reducible::Reducible;
use std::collections::{HashSet};
use std::ptr::null;

#[derive(Debug)]
pub struct QuickGraph {
    pub adjacency: Vec<Option<Vec<usize>>>,
    pub edge_count: usize
}

impl QuickGraph {
    /* creates a new graph with a given number of vertices */
    pub fn new(vertex_count: usize) -> QuickGraph {
        let adjacency: Vec<Option<Vec<usize>>> = vec![Some(Vec::new()); vertex_count];
        QuickGraph { adjacency, edge_count: 0 }
    }

    /* checks if two vertices are adjacent */
    pub fn are_adjacent(&self, u: usize, v: usize) -> bool {
        match &self.adjacency[u] {
            Some(adjacency_list) => adjacency_list.contains(&v),
            None => false
        }
    }

    /* checks if there exist two vertices in three vertices which are adjacent */
    pub fn are_adjacent_triple(&self, u: usize, v: usize, w: usize) -> bool {
        self.are_adjacent(u, v) || self.are_adjacent(u, w) || self.are_adjacent(v, w)
    }

    /* checks if a vertex is an isolated clique */
    pub fn is_isolated_clique(&self, vertex: usize) -> bool {
        /* checks if vertex exists */
        match &self.adjacency[vertex] {
            Some(neighborhood) => {
                /* checks neighbors of given vertex */
                for &neighbor in neighborhood {
                    match &self.adjacency[neighbor] {
                        Some(neighbors_neighbors) => {
                            if !neighborhood.iter()
                                .filter(|&&v| v != neighbor)
                                .all(|v| neighbors_neighbors
                                    .contains(v)) {
                                return false;
                            }
                        },
                        None => {return false; }
                    }
                }
                true
            },
            None => false
        }
    }

    /* find candidates for twin reduction */
    pub fn find_twins(&self) -> Option<(usize, usize)> {
        /* sort adjacency list in order to find twins more easily */
        let sorted_adjacency: Vec<Option<Vec<usize>>> = self.adjacency.iter().map(|adjacency_list| {
            match adjacency_list {
                Some(adjacency_list) => {
                    let mut copy = adjacency_list.clone();
                    copy.sort();
                    Some(copy)
                },
                None => None
            }
        }).collect();

        /* look for twins */
        for u in 0..sorted_adjacency.len() {
            if let Some(current_neighbors) = &sorted_adjacency[u] {
                let current_neighbors: &Vec<usize> = &current_neighbors;
                if current_neighbors.len() == 3 {
                    for v in (u + 1)..sorted_adjacency.len() {
                        if let Some(twins_neighbors) = &sorted_adjacency[v] {
                            if twins_neighbors.len() == 3 && *current_neighbors == *twins_neighbors {
                                return Some((u, v));
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

impl Reducible for QuickGraph {
     fn remove_vertex(&mut self, u: usize) {
         /* look for vertex in adjacency list */
         if let Some(neighbors_of_removed_vertex) = &self.adjacency[u] {
             /* delete vertex itself */
             self.edge_count -= neighbors_of_removed_vertex.len();
             self.adjacency[u] = None;

             /* update neighbors of removed vertex */
             self.adjacency = self.adjacency
                 .iter()
                 .map(|neighborhood| {
                     if let Some(neighborhood) = neighborhood {
                         Some(neighborhood
                             .iter()
                             .filter(|&&neighbor| neighbor != u)
                             .cloned()
                             .collect())
                     } else {
                         None
                     }
                 }).collect();
         }
    }

    fn merge_vertices(&mut self, u: usize, v: usize) {
        /* find neighborhood of resulting vertex */
        let mut new_neighborhood: HashSet<usize> = HashSet::new();
        if let Some(neighbors_of_remaining_vertex) = &self.adjacency[u] {

            /* gather neighbors of remaining vertex */
            for &neighbor in neighbors_of_remaining_vertex {
                if neighbor != v {
                    new_neighborhood.insert(neighbor);
                }
            }

            /* gather neighbors of removed vertex */
            if let Some(neighbors_of_removed_vertex) = &self.adjacency[v] {
                for &neighbor in neighbors_of_removed_vertex {
                    if neighbor != u {
                        new_neighborhood.insert(neighbor);
                    }
                }

                /* update the other vertices of the graph */
                self.adjacency[u] = Some(new_neighborhood.into_iter().collect());
                for neighborhood in &mut self.adjacency {
                    if let Some(neighborhood) = neighborhood {
                        if neighborhood.contains(&v) && !neighborhood.contains(&u) {
                            neighborhood.push(u);
                        }
                    }
                }

                /* remove merged vertex and update graph */
                self.remove_vertex(v);
                self.edge_count = self.adjacency.iter()
                    .map(|neighborhood| {
                        match neighborhood {
                            Some(neighborhood) => neighborhood.len(),
                            None => 0
                        }
                    }).sum::<usize>() / 2;
            }
        }
    }
}