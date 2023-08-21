use super::{ArcId, Dcel};
pub type VertexId = usize;

#[derive(Clone, Debug)]
pub struct Vertex {
    arcs: Vec<ArcId>,
}

impl Vertex {
    pub fn new(arcs: &Vec<ArcId>) -> Self {
        Vertex { arcs: arcs.clone() }
    }

    pub fn arcs(&self) -> &Vec<ArcId> {
        &self.arcs
    }

    pub fn push_arc(&mut self, a: ArcId) {
        self.arcs.push(a);
    }

    pub fn push_arc_at(&mut self, a: ArcId, position: usize) {
        self.arcs.insert(position, a);
    }

    pub fn remove_arc_at(&mut self, position: usize) {
        self.arcs.remove(position);
    }

    pub fn remove_arcs(&mut self) {
        self.arcs.clear();
    }

    pub fn remove_invalid(&mut self, ia: &Vec<bool>) {
        self.arcs.retain(|a| !ia[*a]);
    }
}
