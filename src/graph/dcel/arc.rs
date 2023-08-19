use super::{FaceId, VertexId};
use crate::graph::builder::types;
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

impl From<&types::Arc> for Arc {
    fn from(arc: &types::Arc) -> Self {
        Arc {
            src: arc.src,
            dst: arc.dst,
            next: arc.next.unwrap(),
            prev: arc.prev.unwrap(),
            twin: arc.twin.unwrap(),
            face: arc.face.unwrap(),
        }
    }
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

    pub fn reset_src(&mut self, src: ArcId) {
        self.src = src;
    }

    pub fn dst(&self) -> VertexId {
        self.dst
    }

    pub fn reset_dst(&mut self, dst: ArcId) {
        self.dst = dst;
    }
    pub fn reset_twin(&mut self, twin: ArcId) {
        self.twin = twin;
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
