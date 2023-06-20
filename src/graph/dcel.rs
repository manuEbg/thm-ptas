pub mod arc;
pub mod face;
pub mod spanning_tree;
pub mod vertex;

use std::collections::HashSet;

use self::face::FaceIterator;

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
    arc_set: HashSet<String>,
}

impl Dcel {
    pub fn new() -> Self {
        Dcel {
            vertices: vec![],
            arcs: vec![],
            faces: vec![],
            arc_set: HashSet::new(),
        }
    }

    pub fn push_vertex(&mut self, v: Vertex) {
        self.vertices.push(v);
    }

    pub fn push_arc(&mut self, a: Arc) {
        self.arc_set
            .insert([a.src().to_string(), a.dst().to_string()].join(" "));
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
        let mut tree = SpanningTree::new(self);
        tree.build(start);
        tree
    }

    pub fn has_arc(&self, u: VertexId, v: VertexId) -> bool {
        self.arc_set
            .contains(&[u.to_string(), v.to_string()].join(" "))
    }

    pub fn twin(&self, arc: ArcId) -> &Arc {
        let twin = self.arc(arc).twin();
        self.arc(twin)
    }

    pub fn triangulate(&mut self) {
        let count = self.num_faces();
        for f in 0..count {
            while self.triangulate_face(f) {}
        }
    }

    fn triangulate_face(&mut self, f: FaceId) -> bool {
        let face = self.face(f);

        let mut face_iter = FaceIterator::new(self, face.start_arc());

        let whatever = face_iter.next();
        match whatever {
            Some((mut a1, mut arc1)) => {
                let mut a3 = 0;
                for (a2, arc2) in face_iter {
                    match self.triangle(arc1, arc2) {
                        Some(result) => {
                            if result {
                                a3 = a2;
                                break;
                            }
                        }
                        None => {
                            return false;
                        }
                    }
                    arc1 = arc2;
                    a1 = a2;
                }
                self.close_triangle(a1, a3);
                true
            }
            None => {
                panic!("FACE IS EMPTY!")
            }
        }
    }

    fn triangle(&self, a1: &Arc, a2: &Arc) -> Option<bool> {
        let a = a1.src();
        let b = a1.dst();
        let c = a2.dst();
        let d = self.arc(a2.next()).dst();

        if a == d {
            return None;
        }

        if self.has_arc(a, c) || a == c {
            //check next arc
            return Some(false);
        }
        Some(true)
    }

    fn close_triangle(&mut self, a1: ArcId, a2: ArcId) {
        let arc1 = &self.arcs[a1];
        let arc2 = &self.arcs[a2];
        let old_f = arc1.face();
        let new_f = self.num_faces();

        let u = arc1.src();
        let v = arc2.dst();
        let arc3_idx = self.num_arcs();
        let arc3_twin_idx = arc3_idx + 1;

        let arc3 = Arc::new(v, u, a1, a2, arc3_twin_idx, new_f);
        let arc3_twin = Arc::new(u, v, arc2.next(), arc1.prev(), arc3_idx, old_f);
        let new_face = Face::new(arc3_idx);

        self.arcs[a1].set_face(new_f);
        self.arcs[a2].set_face(new_f);
        self.faces[old_f].set_start_arc(arc3_twin_idx);
        self.faces.push(new_face);

        self.add_arc(&arc3, arc3_idx);
        self.add_arc(&arc3_twin, arc3_twin_idx);
    }

    fn add_arc(&mut self, arc: &Arc, id: ArcId) {
        self.push_arc(*arc);
        self.arcs[arc.next()].set_prev(id);
        self.arcs[arc.prev()].set_next(id);
        self.vertices[arc.src()].push_arc(id);
    }
}
