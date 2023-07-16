use crate::graph::reducible::Reducible;
use std::collections::{HashSet};

#[derive(Debug)]
pub struct QuickGraph {
    pub adjacency: Vec<Vec<usize>>,
    pub edge_count: usize
}

impl QuickGraph {
    pub fn new(vertex_count: usize) -> QuickGraph {
        let adjacency: Vec<Vec<usize>> = vec![Vec::new(); vertex_count];
        QuickGraph { adjacency, edge_count: 0 }
    }

    fn are_adjacent(&self, u: usize, v: usize) -> bool {
        self.adjacency[u].contains(&v)
    }

    pub fn remove_edge(&mut self, u: usize, v:usize) {
        if self.are_adjacent(u, v) {
            self.adjacency[u].retain(|&vertex| vertex != v);
            self.adjacency[v].retain(|&vertex| vertex != u);
            self.edge_count -= 1;
        }
    }

    pub fn insert_vertex(&mut self, vertex: usize, neighborhood: Vec<usize>) {
        /* update other vertices */
        self.adjacency = self.adjacency.iter().map(|neighbors|
            neighbors.iter().map(|&neighbor|
                if neighbor >= vertex { neighbor + 1} else {neighbor}).collect()
        ).collect();

        /* insert new vertex */
        self.adjacency.insert(vertex, neighborhood.clone());
        neighborhood.iter().for_each(|&neighbor| self.adjacency[neighbor].push(vertex));
        self.edge_count += self.adjacency[vertex].len();
    }

    pub fn is_isolated_clique(&self, vertex: usize) -> bool {
        let neighborhood = &self.adjacency[vertex];
        for &neighbor in neighborhood {
            let neighbors_neighbors = &self.adjacency[neighbor];
            if !neighborhood.iter()
                .filter(|&&v| v != neighbor)
                .all(|v| neighbors_neighbors
                    .contains(v)) {
                return false;
            }
        }
        true
    }

    pub fn find_twins(&self) -> Option<(usize, usize)> {
        /* sort adjacency list in order to find twins more easily */
        let sorted_adjacency: Vec<Vec<usize>> = self.adjacency.iter().map(|adjacency_list| {
            let mut copy = adjacency_list.clone();
            copy.sort();
            copy
        }).collect();

        /* look for twins */
        for u in 0..sorted_adjacency.len() {
            let current_neighbors: &Vec<usize> = &sorted_adjacency[u];
            if current_neighbors.len() == 3 {
                for v in (u + 1)..sorted_adjacency.len() {
                    if sorted_adjacency[v].len() == 3 && *current_neighbors == sorted_adjacency[v] {
                        return Some((u, v));
                    }
                }
            }
        }
        None
    }
}

impl Reducible for QuickGraph {
     fn remove_vertex(&mut self, u: usize) {
        self.edge_count -= self.adjacency[u].len();
        self.adjacency.remove(u);
         self.adjacency = self.adjacency
            .iter()
            .map(|neighborhood| {
                neighborhood
                    .iter()
                    .filter(|&&neighbor| neighbor != u)
                    .map(|&neighbor| if neighbor > u {neighbor - 1} else {neighbor} )
                    .collect()
            }).collect();
    }

    fn merge_vertices(&mut self, u: usize, v: usize) {
        /* find neighborhood of resulting vertex */
        let mut new_neighborhood: HashSet<usize> = HashSet::new();
         for &neighbor in &self.adjacency[u] {
            if neighbor != v {
                new_neighborhood.insert(neighbor);
            }
        }
        for &neighbor in &self.adjacency[v] {
            if neighbor != u {
                new_neighborhood.insert(neighbor);
            }
        }

        /* update the other vertices of the graph */
        self.adjacency[u] = new_neighborhood.into_iter().collect();
        for neighborhood in &mut self.adjacency {
            if neighborhood.contains(&v) && !neighborhood.contains(&u) {
                neighborhood.push(u);
            }
        }

        /* remove merged vertex and update graph */
        self.remove_vertex(v);
        self.edge_count = self.adjacency.iter()
            .map(|neighborhood| neighborhood.len()).sum::<usize>() / 2;
    }
}