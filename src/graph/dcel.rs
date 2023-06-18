pub mod arc;
pub mod face;
pub mod spanning_tree;
pub mod vertex;

use super::iterators::bfs::BfsIter;
use crate::graph::dcel::spanning_tree::SpanningTree;
use arc::{Arc, ArcId};
use face::{Face, FaceId};
use vertex::{Vertex, VertexId};

#[derive(Debug)]
pub struct Dcel {
    vertices: Vec<Vertex>,
    arcs: Vec<Arc>,
    faces: Vec<Face>,
}

impl Dcel {
    pub fn new() -> Self {
        Dcel {
            vertices: vec![],
            arcs: vec![],
            faces: vec![],
        }
    }

    pub fn push_vertex(&mut self, v: Vertex) {
        self.vertices.push(v);
    }

    pub fn push_arc(&mut self, a: Arc) {
        self.arcs.push(a);
    }

    pub fn push_face(&mut self, f: Face) {
        self.faces.push(f);
    }

    pub fn walk_face(&self, face: FaceId) -> Vec<ArcId> {
        self.faces[face].walk_face(self)
    }

    pub fn face(&self, idx: FaceId) -> &Face {
        &self.faces[idx]
    }

    pub fn arcs(&self) -> &Vec<Arc> {
        &self.arcs
    }

    pub fn arc(&self, idx: ArcId) -> &Arc {
        &self.arcs[idx]
    }

    pub fn faces(&self) -> &Vec<Face> {
        &self.faces
    }

    pub fn vertices(&self) -> &Vec<Vertex> {
        &self.vertices
    }

    pub fn vertex(&self, idx: VertexId) -> &Vertex {
        &self.vertices[idx]
    }

    pub fn num_arcs(&self) -> usize {
        self.arcs.len()
    }

    pub fn num_vertices(&self) -> usize {
        self.vertices.len()
    }

    pub fn num_faces(&self) -> usize {
        self.faces.len()
    }

    pub fn neighbors(&self, v: VertexId) -> Vec<VertexId> {
        let mut neighbors: Vec<usize> = vec![];
        for a in self.vertex(v).arcs().iter() {
            let n = self.arc(*a).dst();
            neighbors.push(n);
        }
        neighbors
    }

    pub fn spanning_tree(&self, start: VertexId) -> SpanningTree {
        let mut tree = SpanningTree::new(&self);
        tree.build(start);
        tree
    }

    pub fn twin(&self, arc: ArcId) -> &Arc {
        let twin = self.arc(arc).twin();
        self.arc(twin)
    }
}
