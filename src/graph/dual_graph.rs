use super::{dcel::face::Face, dcel::spanning_tree::SpanningTree, Dcel};

// TODO: Maybe just walk the faces only once.
// Right now, we walk them in DualGraph::add_face and in TreeDecomposition::from().
#[derive(Debug)]
pub struct DualGraph<'a> {
    spanning_tree: &'a SpanningTree<'a>,
    dcel: &'a Dcel,
    adjacent: Vec<Vec<usize>>,
}

impl<'a> DualGraph<'a> {
    pub fn new(spanning_tree: &'a SpanningTree) -> Self {
        let adjacent = vec![Vec::new(); spanning_tree.dcel().num_faces()];

        Self {
            spanning_tree,
            dcel: spanning_tree.dcel(),
            adjacent,
        }
    }

    pub fn build(&mut self) {
        for (i, f) in self.dcel.faces().iter().enumerate() {
            self.add_face(f, i);
        }
    }

    fn add_face(&mut self, face: &Face, idx: usize) {
        for a in face.walk_face(self.dcel) {
            if self.spanning_tree.contains_arc(a) {
                continue;
            }
            let twin = self.dcel.twin(a);
            self.adjacent[idx].push(twin.face());
        }
    }

    pub fn get_neighbors(&self, idx: usize) -> Vec<usize> {
        self.adjacent[idx].clone()
    }

    pub fn get_vertices(&self) -> Vec<usize> {
        (0..self.adjacent.len()).collect()
    }

    pub fn get_adjacent(&self) -> &Vec<Vec<usize>> {
        &self.adjacent
    }

    pub fn num_vertices(&self) -> usize {
        self.adjacent.len()
    }

    pub fn get_dcel(&self) -> &Dcel {
        self.dcel
    }
}
