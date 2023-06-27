pub mod arc;
pub mod face;
pub mod spanning_tree;
pub mod vertex;


use std::{collections::HashSet, error::Error};

use self::face::FaceIterator;
use super::iterators::bfs::BfsIter;
use crate::graph::{dcel::spanning_tree::SpanningTree, builder::dcel_builder::DcelBuilder};
use arc::{Arc, ArcId};
use face::{Face, FaceId};
use vertex::{Vertex, VertexId};



#[derive(Debug)]
pub struct SubDcel<'a> {
    pub dcel: &'a Dcel,
    pub sub: Dcel,
    pub arc_mapping: Vec<arc::ArcId>,
    pub vertex_mapping: Vec<vertex::VertexId>,
}

impl<'a> SubDcel<'a> {
    pub fn new(dcel: &'a Dcel, sub: Dcel, arc_mapping: Vec<arc::ArcId>, vertex_mapping: Vec<vertex::VertexId>) -> Self {
        Self { dcel, sub, arc_mapping, vertex_mapping }
    }

    pub fn get_original_arc(&self, a: arc::ArcId) -> Option<&arc::ArcId> {
        self.arc_mapping.get(a)
    }

    pub fn get_original_vertex(&self, v: vertex::VertexId) -> Option<&vertex::VertexId> {
        self.vertex_mapping.get(v)
    }

    pub fn get_vertices(&self) -> &Vec<vertex::Vertex> {
        return self.sub.vertices();
    }
}

#[derive(Debug)]
pub struct SubDcelBuilder<'a> {
    pub dcel: &'a Dcel,
    pub dcel_builder: DcelBuilder,
    pub vertex_mapping: Vec<vertex::VertexId>,
    pub arc_mapping: Vec<arc::ArcId>,
    pub last_vertex_id: vertex::VertexId
}

impl<'a> SubDcelBuilder<'a> {
    pub fn new(dcel: &'a Dcel) -> Self {
        Self { dcel, dcel_builder: DcelBuilder::new(), vertex_mapping: vec![], arc_mapping: vec![], last_vertex_id: 0 }
    }

    /* Returns the mapped vertex id */
    pub fn push_vertex(&mut self, v: vertex::VertexId) -> vertex::VertexId {
        /* Is already mapped? */
        for (idx, vertex) in self.vertex_mapping.iter().enumerate() {
            if *vertex == v {
                return idx;
            }
        }

        /* Add this vertex */
        //self.vertex_mapping[self.last_vertex_id] = v;
        self.vertex_mapping.push(v);
        self.last_vertex_id += 1;

        return self.last_vertex_id-1;
    }

    pub fn push_arc(&mut self, a: &arc::Arc) {
        let src = self.push_vertex(a.src());
        let dst = self.push_vertex(a.dst());
        self.dcel_builder.push_arc(src, dst);
        self.dcel_builder.push_arc(dst, src);
    }

    // pub fn build(&mut self) -> Result<SubDcel, Box<dyn Error>> {
    //     let final_dcel = self.dcel_builder.build();
    //     let mut arc_mapping = vec![0 as arc::ArcId; final_dcel.num_arcs()];
    //     /* This probably very slow */
    //     for (sub_arc_idx, sub_arc) in final_dcel.arcs.iter().enumerate() {
    //         for (main_arc_idx, main_arc) in self.dcel.arcs.iter().enumerate() {
    //             if self.vertex_mapping[sub_arc.src()] == main_arc.src() && self.vertex_mapping[sub_arc.dst()] == main_arc.dst() {
    //                 arc_mapping[sub_arc_idx] = main_arc_idx;
    //             }
    //         }
    //     }

    //     Ok(SubDcel { dcel: &self.dcel, sub: final_dcel, arc_mapping, vertex_mapping: self.vertex_mapping.clone() })
    // }
}

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


    pub fn find_rings(&self, n: usize) -> Result<Vec<SubDcel>, Box<dyn Error>> {
        let mut result = vec![];
        let spanning_tree = self.spanning_tree(0);

        for depth in 1..(n+1) {
            let mut visited = vec![false; self.vertices.len()];

            let mut builder = SubDcelBuilder::new(self);

            for spanning_arc in spanning_tree.arcs() {
                let arc = self.arc(*spanning_arc);
                let src_level = spanning_tree.vertex_level()[arc.src()];

                /* Is this vertex part of the ring? */
                if src_level == depth && !visited[arc.src()] {
                    visited[arc.src()] = true;

                    let outgoing_arcs = self.arcs().iter().filter(|a| a.src() == arc.src()).collect::<Vec<_>>();
                    for outgoing_arc in outgoing_arcs {

                        /* Add ring arcs */
                        let dst_level = spanning_tree.vertex_level()[outgoing_arc.dst()];
                        if dst_level == depth && !visited[outgoing_arc.dst()] {
                            //println!("{:?} {:?}", arc.get_src(), outgoing_arc.get_dst());
                            // builder.push_arc(arc.src(), outgoing_arc.dst());
                            // builder.push_arc(outgoing_arc.dst(), arc.src());

                            builder.push_arc(outgoing_arc);
                        }
                    }
                }
            }

            let final_dcel = builder.dcel_builder.build();
            let mut arc_mapping = vec![0 as arc::ArcId; final_dcel.num_arcs()];
            /* This probably very slow */
            for (sub_arc_idx, sub_arc) in final_dcel.arcs.iter().enumerate() {
                for (main_arc_idx, main_arc) in self.arcs.iter().enumerate() {
                    if builder.vertex_mapping[sub_arc.src()] == main_arc.src() && builder.vertex_mapping[sub_arc.dst()] == main_arc.dst() {
                        arc_mapping[sub_arc_idx] = main_arc_idx;
                    }
                }
            }

            result.push(SubDcel { dcel: self, sub: final_dcel, arc_mapping, vertex_mapping: builder.vertex_mapping });
        }

        Ok(result)
    }
}
