use super::ArcId;
pub type VertexId = usize;

#[derive(Debug)]
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
}
