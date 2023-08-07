use arboretum_td::tree_decomposition::{TreeDecomposition, Bag};

pub struct PostOrderIter<'a> {
    td: &'a TreeDecomposition,
    stack: Vec<usize>,  // Just bag IDs.
    visited: Vec<bool>, // Improvement: Use a bitset.
}

impl<'a> PostOrderIter<'a> {
    pub fn new(td: &'a TreeDecomposition) -> Self {
        PostOrderIter {
            td,
            stack: vec![td.root.unwrap()],
            visited: vec![false; td.bags.len()],
        }
    }

    /// Recursively traverse the subtrees.
    /// We push the child nodes first from left to right.
    /// It is assumed that the sub root is already on the stack.
    fn traverse_subtrees(&mut self, sub_root: &Bag) {
        for &child_id in sub_root.neighbors.iter() {
            if !self.visited[child_id] {
                let child = &self.td.bags[child_id];
                self.stack.push(child_id);
                self.visited[child_id] = true;
                self.traverse_subtrees(child);
            }
        }
    }
}

impl<'a> Iterator for PostOrderIter<'a> {
    type Item = &'a Bag;

    fn next(&mut self) -> Option<Self::Item> {
        // Traverse subtrees until we find a leaf node to return.
        while let Some(&current_id) = self.stack.last() {
            let current = &self.td.bags[current_id];
            if current.neighbors.len() == 0 || self.visited[current_id] {
                self.stack.pop();
                return Some(current);
            }

            self.visited[current_id] = true;
            self.traverse_subtrees(current);
        }
        None
    }
}
