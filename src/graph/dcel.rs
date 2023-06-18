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

    pub fn get_arcs(&self) -> &Vec<Arc> {
        &self.arcs
    }

    pub fn get_arc(&self, idx: ArcId) -> &Arc {
        &self.arcs[idx]
    }

    pub fn get_faces(&self) -> &Vec<Face> {
        &self.faces
    }

    pub fn get_vertices(&self) -> &Vec<Vertex> {
        &self.vertices
    }

    pub fn get_vertex(&self, idx: VertexId) -> &Vertex {
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
        for a in self.get_vertex(v).get_arcs().iter() {
            let n = self.get_arc(*a).get_dst();
            neighbors.push(n);
        }
        neighbors
    }

    pub fn spanning_tree(&self, start: VertexId) -> SpanningTree {
        let mut tree = SpanningTree::new(&self);
        tree.build(start);
        tree
    }

    pub fn get_twin(&self, arc: ArcId) -> &Arc {
        let twin = self.get_arc(arc).get_twin();
        self.get_arc(twin)
    }

    pub fn add_edge(from: VertexId, to: VertexId, prev: ArcId, next: ArcId, face: FaceId) {
        todo!()
    }
        

}
