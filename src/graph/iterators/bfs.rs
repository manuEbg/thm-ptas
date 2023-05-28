use super::dcel::Dcel;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct BfsIter<'a> {
    dcel: &'a  Dcel,
    queue: VecDeque<usize>,
    discovered: Vec<bool>,
    visited: Vec<bool>,
    level: Vec<usize>,
}

#[derive(Debug)]
pub struct BfsItem {
    vertex: usize,
    level: usize,
}

impl<'a> BfsIter<'a> {

    pub fn new(dcel: &'a Dcel, start: usize) -> Self {
        let mut iter = BfsIter { 
            dcel, 
            queue: VecDeque::from([start]),
            visited: vec![false; dcel.num_vertices()], 
            discovered: vec![false; dcel.num_vertices()], 
            level: vec![0; dcel.num_vertices()] 
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
            };

            for n in self.dcel.neighbors(vertex) {
                if self.discovered[n] { continue }
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
