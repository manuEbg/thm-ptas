use super::{Dcel, dcel::{SpanningTree, Face}};

#[derive(Debug)]
pub struct DualGraph<'a> {
    spanning_tree: &'a SpanningTree<'a>,
    dcel: &'a Dcel,
    adjacent: Vec<Vec<usize>>,
}

impl<'a> DualGraph<'a> {

    pub fn new(spanning_tree: &'a SpanningTree) -> Self {
        let adjacent = vec![Vec::new(); spanning_tree.get_dcel().num_faces()];
        
        Self { 
            spanning_tree,
            dcel: spanning_tree.get_dcel(),
            adjacent,
        }
    }

    pub fn build(&mut self) { 
        
        for (i, f) in self.dcel.get_faces().iter().enumerate() {
            self.add_face(f, i);
        }
    }

    pub fn add_face(&mut self, face: &Face, idx: usize) {
        for a in face.walk_face(self.dcel) {
            if self.spanning_tree.contains_arc(a) { continue; }
            let twin = self.dcel.get_twin(a);
            self.adjacent[idx].push(twin.get_face());
        }
    }

    
} 
