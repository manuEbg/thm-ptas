use super::{FaceId, VertexId};
pub type ArcId = usize;

#[derive(Debug, Copy, Clone)]
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

    pub fn next(&self) -> ArcId {
        self.next
    }

    pub fn prev(&self) -> ArcId {
        self.prev
    }

    pub fn src(&self) -> VertexId {
        self.src
    }

    pub fn dst(&self) -> VertexId {
        self.dst
    }

    pub fn twin(&self) -> ArcId {
        self.twin
    }

    pub fn face(&self) -> FaceId {
        self.face
    }

    pub fn set_face(&mut self, f: FaceId) {
        self.face = f;
    }

    pub fn set_next(&mut self, n: ArcId) {
        self.next = n;
    }

    pub fn set_prev(&mut self, p: ArcId) {
        self.prev = p;
    }
}
