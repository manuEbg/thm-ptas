use super::{FaceId, VertexId};
pub type ArcId = usize;

#[derive(Debug)]
pub struct Arc {
    src: VertexId,
    dst: VertexId,
    next: ArcId,
    prev: ArcId,
    twin: ArcId,
    face: FaceId,
}

impl Arc {
    pub fn new(
        src: VertexId,
        dst: VertexId,
        next: ArcId,
        prev: ArcId,
        twin: ArcId,
        face: FaceId,
    ) -> Self {
        Arc {
            src,
            dst,
            next,
            prev,
            twin,
            face,
        }
    }

    pub fn get_next(&self) -> ArcId {
        self.next
    }

    pub fn get_src(&self) -> VertexId {
        self.src
    }

    pub fn get_dst(&self) -> VertexId {
        self.dst
    }

    pub fn get_twin(&self) -> ArcId {
        self.twin
    }

    pub fn get_face(&self) -> FaceId {
        self.face
    }

    pub fn set_face(&mut self, f: FaceId) {
        self.face = f;
    }
}
