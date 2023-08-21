use arboretum_td::tree_decomposition::{Bag, TreeDecomposition};

use super::dcel::arc::ArcId;
use super::dcel::vertex::VertexId;

use super::dcel::Dcel;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct BfsIter<'a> {
    dcel: &'a Dcel,
    queue: VecDeque<VertexId>,
    discovered: Vec<bool>,
    discovered_arc: Vec<Option<ArcId>>,
    visited: Vec<bool>,
    level: Vec<usize>,
}

#[derive(Debug)]
pub struct BfsItem {
    pub level: usize,
    pub vertex: VertexId,
    //the arc used to discover the current vertex
    pub arc: Option<ArcId>,
}

impl<'a> BfsIter<'a> {
    pub fn new(dcel: &'a Dcel, start: VertexId) -> Self {
        let mut iter = BfsIter {
            dcel,
            queue: VecDeque::from([start]),
            visited: vec![false; dcel.num_vertices()],
            discovered: vec![false; dcel.num_vertices()],
            discovered_arc: vec![None; dcel.num_vertices()],
            level: vec![0; dcel.num_vertices()],
        };
        iter.discovered[start] = true;
        iter
    }
}

impl<'a> Iterator for BfsIter<'a> {
    type Item = BfsItem;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(vertex) = self.queue.pop_front() {
            let it = Self::Item {
                vertex,
                level: self.level[vertex],
                arc: self.discovered_arc[vertex],
            };

            for a in self.dcel.vertex(vertex).arcs() {
                if self.dcel.invalid_arcs[*a] {
                    println!("BFS: skipping invalid arc{a}");
                    continue;
                }
                let n = self.dcel.arc(*a).dst();

                if self.discovered[n] {
                    continue;
                }

                self.discovered_arc[n] = Some(*a);
                self.discovered[n] = true;
                self.level[n] = self.level[vertex] + 1;
                self.queue.push_back(n);
            }

            self.visited[vertex] = true;
            return Some(it);
        }
        None
    }
}

/// Represents a breadth-first search iterator used to traverse tree decompositions.
pub struct TreeDecompBfsIter<'a> {
    td: &'a TreeDecomposition,
    queue: VecDeque<usize>, // Bag IDs.
    visited: Vec<bool>,     // @speed Use a bitset.
}

impl<'a> TreeDecompBfsIter<'a> {
    /// Creates a iterator for the given tree decomposition.
    pub fn new(td: &'a TreeDecomposition) -> Self {
        TreeDecompBfsIter {
            td,
            queue: VecDeque::from([td.root.unwrap()]),
            visited: vec![false; td.bags.len()],
        }
    }
}

impl<'a> Iterator for TreeDecompBfsIter<'a> {
    type Item = &'a Bag;

    /// Returns the next bag of the tree decomposition in post order.
    fn next(&mut self) -> Option<Self::Item> {
        let front = self.queue.pop_front();
        if front.is_none() {
            return None;
        }

        let v = front.unwrap();
        self.visited[v] = true;
        let bag = &self.td.bags[v];

        // Find all unvisited neighbors.
        bag.neighbors
            .iter()
            .filter(|&&n| !self.visited[n])
            .for_each(|&n| self.queue.push_back(n));

        Some(bag)
    }
}
