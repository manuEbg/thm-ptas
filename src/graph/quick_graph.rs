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

    pub fn num_vertices(&self) -> usize { self.adjacency.len() }
    pub fn num_edges(&self) -> usize { self.edge_count }
    pub fn degree(&self, v: usize) -> usize { self.adjacency[v].len() }
    pub fn neighborhood(&self, v: usize) -> &Vec<usize> { &self.adjacency[v] }
    pub fn are_adjacent(&self, u: usize, v: usize) -> bool { self.adjacency[u].contains(&v)}

    pub fn add_vertex(&mut self) {
        self.adjacency.push(Vec::new())
    }

    pub fn remove_edge(&mut self, u: usize, v:usize) {
        if self.are_adjacent(u, v) {
            self.adjacency[u].retain(|&vertex| vertex != v);
            self.adjacency[v].retain(|&vertex| vertex != u);
            self.edge_count -= 1;
        }
    }

    pub fn insert_vertex(&mut self, vertex: usize, neighborhood: Vec<usize>) {
        self.adjacency = self.adjacency.iter().map(|neighbors|
            neighbors.iter().map(|&neighbor|
                if neighbor >= vertex { neighbor + 1} else {neighbor}).collect()
        ).collect();
        self.adjacency.insert(vertex, neighborhood.clone());
        neighborhood.iter().for_each(|&neighbor| self.adjacency[neighbor].push(vertex));
        self.edge_count += self.adjacency[vertex].len();
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

        self.adjacency[u] = new_neighborhood.into_iter().collect();
        for neighborhood in &mut self.adjacency {
            if neighborhood.contains(&v) && !neighborhood.contains(&u) {
                neighborhood.push(u);
            }
        }
        self.remove_vertex(v);
        self.edge_count = self.adjacency.iter()
            .map(|neighborhood| neighborhood.len()).sum::<usize>() / 2;
    }
}