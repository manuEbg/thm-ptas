use super::dcel::Dcel;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct BfsIter<'a> {
    dcel: &'a  Dcel,
    queue: VecDeque<usize>,
    discovered: Vec<bool>,
    discovered_arc: Vec<Option<usize>>,
    visited: Vec<bool>,
    level: Vec<usize>,
}

#[derive(Debug)]
pub struct BfsItem {
    pub vertex: usize,
    pub level: usize,
    //the arc used to discover the current vertex
    pub arc: Option<usize>,
}

impl<'a> BfsIter<'a> {

    pub fn new(dcel: &'a Dcel, start: usize) -> Self {
        let mut iter = BfsIter { 
            dcel, 
            queue: VecDeque::from([start]),
            visited: vec![false; dcel.num_vertices()], 
            discovered: vec![false; dcel.num_vertices()],
            discovered_arc: vec![None; dcel.num_vertices()],
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
                arc: self.discovered_arc[vertex],
            };

            for a in self.dcel.vertex(vertex).arcs() {
                let n = self.dcel.arc(*a).dst();
                
                if self.discovered[n] { continue }
                
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
