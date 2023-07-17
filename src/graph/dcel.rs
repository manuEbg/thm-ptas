pub mod arc;
pub mod face;
pub mod spanning_tree;
pub mod vertex;

use std::{collections::HashSet, error::Error};

use self::face::FaceIterator;
use super::iterators::bfs::BfsIter;
use crate::graph::{builder::dcel_builder::DcelBuilder, dcel::spanning_tree::SpanningTree};
use arc::{Arc, ArcId};
use face::{Face, FaceId};
use vertex::{Vertex, VertexId};

#[derive(Clone, Debug)]
pub struct SubDcel {
    pub dcel: Dcel,
    pub sub: Dcel,
    pub arc_mapping: Vec<arc::ArcId>,
    pub vertex_mapping: Vec<vertex::VertexId>,
}

impl SubDcel {
    pub fn new(
        dcel: Dcel,
        sub: Dcel,
        arc_mapping: Vec<arc::ArcId>,
        vertex_mapping: Vec<vertex::VertexId>,
    ) -> Self {
        Self {
            dcel,
            sub,
            arc_mapping,
            vertex_mapping,
        }
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

    pub fn triangulate(&mut self) {
        self.sub.triangulate();
    }

    pub fn get_untriangulated_arcs(&self) -> Vec<arc::Arc> {
        return self.sub.arcs[0..self.sub.pre_triangulation_arc_count].to_vec();
    }

    pub fn get_triangulated_arcs(&self) -> Vec<arc::Arc> {
        let arc_len = self.sub.arcs().len();
        return self.sub.arcs[self.sub.pre_triangulation_arc_count..arc_len].to_vec();
    }

    pub fn was_triangulated(&self) -> bool {
        self.sub.pre_triangulation_arc_count() > 0
    }
}

#[derive(Debug)]
pub struct SubDcelBuilder {
    pub dcel: Dcel,
    pub dcel_builder: DcelBuilder,
    pub vertex_mapping: Vec<vertex::VertexId>,
    pub arc_mapping: Vec<arc::ArcId>,
    pub last_vertex_id: vertex::VertexId,
}

impl SubDcelBuilder {
    pub fn new(dcel: Dcel) -> Self {
        Self {
            dcel,
            dcel_builder: DcelBuilder::new(),
            vertex_mapping: vec![],
            arc_mapping: vec![],
            last_vertex_id: 0,
        }
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

        self.last_vertex_id - 1
    }

    pub fn push_arc(&mut self, a: &arc::Arc) {
        let src = self.push_vertex(a.src());
        let dst = self.push_vertex(a.dst());
        self.dcel_builder.push_arc(src, dst);
        self.dcel_builder.push_arc(dst, src);
    }

    pub fn build(&mut self) -> Result<SubDcel, Box<dyn Error>> {
        let final_dcel = self.dcel_builder.build();
        let mut arc_mapping = vec![0 as arc::ArcId; final_dcel.num_arcs()];

        /* This probably very slow */
        for (sub_arc_idx, sub_arc) in final_dcel.arcs.iter().enumerate() {
            for (main_arc_idx, main_arc) in self.dcel.arcs.iter().enumerate() {
                if self.vertex_mapping[sub_arc.src()] == main_arc.src()
                    && self.vertex_mapping[sub_arc.dst()] == main_arc.dst()
                {
                    arc_mapping[sub_arc_idx] = main_arc_idx;
                }
            }
        }

        Ok(SubDcel::new(
            self.dcel.clone(),
            final_dcel,
            arc_mapping,
            self.vertex_mapping.clone(),
        ))
    }
}

#[derive(Clone, Debug)]
pub struct Dcel {
    vertices: Vec<Vertex>,
    arcs: Vec<Arc>,
    faces: Vec<Face>,
    arc_set: HashSet<String>,
    pre_triangulation_arc_count: usize,
}

enum FaceInfo {
    Twins,
    Triangle,
    TriangulatedFace,
    NotTriangulated,
}

impl Dcel {
    pub fn new() -> Self {
        Dcel {
            vertices: vec![],
            arcs: vec![],
            faces: vec![],
            arc_set: HashSet::new(),
            pre_triangulation_arc_count: 0,
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
        self.pre_triangulation_arc_count = self.num_arcs();
        let count = self.num_faces();
        for f in 0..count {
            loop {
                if let FaceInfo::TriangulatedFace = self.triangulate_next_triangle(f) {
                    break;
                }
            }
        }
    }

    fn triangulate_next_triangle(&mut self, f: FaceId) -> FaceInfo {
        let face = self.face(f);
        let mut face_iter = FaceIterator::new(self, face.start_arc());
        let start = face_iter.next();
        match start {
            Some((mut a1, _)) => {
                for (a2, _) in face_iter {
                    match self.face_information(a1, a2) {
                        FaceInfo::Twins | FaceInfo::Triangle => {
                            a1 = a2;
                            // println!("1");
                        }
                        FaceInfo::TriangulatedFace => {
                            // println!("3");
                            return FaceInfo::TriangulatedFace;
                        }
                        FaceInfo::NotTriangulated => {
                            self.close_triangle(a1, a2);
                            // println!("2");
                            return FaceInfo::NotTriangulated;
                        }
                    }
                }
                let face_iter2 = FaceIterator::new(self, face.start_arc());
                let mut vec = vec![];
                for (a, _) in face_iter2 {
                    vec.push(a);
                }
                panic!(
                    "FACE {} with {:?} edges iterated. Should never be here",
                    f, vec
                );
                // FaceInfo::TriangulatedFace
            }
            None => {
                panic!("FACE {} IS EMPTY", f);
            }
        }
    }

    fn face_information(&self, a: ArcId, b: ArcId) -> FaceInfo {
        let arc_a = self.arc(a);
        let arc_b = self.arc(b);
        if arc_a.next() != b || arc_b.prev() != a {
            panic!(
                "Arcs a {} and b {} need to be the be consecutive arcs of the same face",
                a, b
            )
        };
        if arc_a.twin() == b {
            return FaceInfo::Twins;
        } else if arc_b.next() == arc_a.prev() {
            return FaceInfo::TriangulatedFace;
        }

        if self.has_arc(arc_b.dst(), arc_a.src()) {
            return FaceInfo::Triangle;
        }
        FaceInfo::NotTriangulated
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

    pub fn find_rings(&self) -> Result<Vec<SubDcel>, Box<dyn Error>> {
        let mut result = vec![];
        let spanning_tree = self.spanning_tree(0);

        for depth in 1..(spanning_tree.max_level() + 1) {
            let mut visited = vec![false; self.vertices.len()];

            let mut builder = SubDcelBuilder::new(self.clone());

            for spanning_arc in spanning_tree.arcs() {
                let arc = self.arc(*spanning_arc);
                let src_level = spanning_tree.vertex_level()[arc.src()];

                /* Is this vertex part of the ring? */
                if src_level == depth && !visited[arc.src()] {
                    visited[arc.src()] = true;

                    let outgoing_arcs = self
                        .arcs()
                        .iter()
                        .filter(|a| a.src() == arc.src())
                        .collect::<Vec<_>>();
                    for outgoing_arc in outgoing_arcs {
                        /* Add ring arcs */
                        let dst_level = spanning_tree.vertex_level()[outgoing_arc.dst()];
                        if dst_level == depth && !visited[outgoing_arc.dst()] {
                            builder.push_arc(outgoing_arc);
                        }
                    }
                }
            }
            let resulting_sub_dcel = builder.build()?;
            result.push(resulting_sub_dcel);
        }

        Ok(result)
    }

    pub fn collect_donut(&self, start: usize, end: usize) -> Result<SubDcel, Box<dyn Error>> {
        let spanning_tree = self.spanning_tree(0);

        if end > spanning_tree.max_level() + 1 {
            return Err("Donut is out of bounds".into());
        }

        let mut visited = vec![false; self.vertices.len()];
        let mut builder = SubDcelBuilder::new(self.clone());

        for vertex in 0..self.vertices().len() {
            let vertex_depth = spanning_tree.vertex_level()[vertex];

            if vertex_depth >= start && vertex_depth < end && !visited[vertex] {
                /* This vertex is part of the donut, so add all its associated arcs in the
                 * donut */
                let outgoing_arcs = self
                    .arcs()
                    .iter()
                    .filter(|a| a.src() == vertex)
                    .filter(|a| !visited[a.dst()])
                    .filter(|a| {
                        spanning_tree.vertex_level()[a.dst()] >= start
                            && spanning_tree.vertex_level()[a.dst()] < end
                    })
                    .collect::<Vec<_>>();

                for arc in outgoing_arcs {
                    builder.push_arc(arc);
                }

                visited[vertex] = true;
            }
        }

        let sub_dcel = builder.build()?;
        Ok(sub_dcel)
    }

    pub fn find_donuts_for_k(&self, k: usize) -> Result<Vec<SubDcel>, Box<dyn Error>> {
        let mut result = vec![];
        let spanning_tree = self.spanning_tree(0);

        let mut last_level = 1;

        for n in 1..(spanning_tree.max_level() + 1) {
            if n % k == 0 {
                /* Current donut is from last_level -> n */
                let mut donut = self.collect_donut(last_level, n)?;
                donut.triangulate();
                result.push(donut);
                last_level = n + 1
            }
        }

        if last_level != spanning_tree.max_level() {
            let mut last_donut = self.collect_donut(last_level, spanning_tree.max_level() + 1)?;
            last_donut.triangulate();
            result.push(last_donut);
        }

        Ok(result)
    }

    pub fn pre_triangulation_arc_count(&self) -> usize {
        self.pre_triangulation_arc_count
    }
}
