use super::{ArcId, Dcel};

pub type FaceId = usize;

#[derive(Debug)]
pub struct Face {
    start_arc: ArcId,
}

impl Face {
    pub fn new(start_arc: ArcId) -> Self {
        Face { start_arc }
    }

    pub fn walk_face(&self, dcel: &Dcel) -> Vec<ArcId> {
        let mut arcs = vec![];
        arcs.push(self.start_arc);
        let mut current_arc = dcel.get_arc(self.start_arc).get_next();
        while current_arc != self.start_arc {
            arcs.push(current_arc);
            current_arc = dcel.get_arc(current_arc).get_next();
        }
        arcs
    }

    pub fn start_arc(&self) -> ArcId {
        self.start_arc
    }
}
