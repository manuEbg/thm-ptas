pub mod arc;
pub mod face;
pub mod spanning_tree;
pub mod vertex;

use std::{collections::HashSet, error::Error};

use self::face::FaceIterator;
use super::{
    iterators::bfs::BfsIter,
    reducible::Reducible,
    sub_dcel::{SubDcel, SubDcelBuilder},
};
use crate::graph::{builder::dcel_builder::DcelBuilder, dcel::spanning_tree::SpanningTree};
use arc::{Arc, ArcId};
use face::{Face, FaceId};
use vertex::{Vertex, VertexId};

#[derive(Clone, Debug)]
pub struct Dcel {
    vertices: Vec<Vertex>,
    pub arcs: Vec<Arc>,
    faces: Vec<Face>,
    arc_set: HashSet<String>,
    pub pre_triangulation_arc_count: usize,
    invalid_faces: Vec<bool>,
    pub invalid_arcs: Vec<bool>,
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
            invalid_faces: vec![],
            invalid_arcs: vec![],
        }
    }

    pub fn adjacency_matrix(&self) -> Vec<Vec<bool>> {
        let mut matrix = vec![vec![false; self.num_vertices()]; self.num_vertices()];
        for (i, v) in self.vertices.iter().enumerate() {
            for a in v.arcs().iter() {
                matrix[i][self.arc(*a).dst()] = true;
            }
        }
        matrix
    }

    pub fn push_vertex(&mut self, v: Vertex) {
        self.vertices.push(v);
    }

    pub fn push_arc(&mut self, a: Arc) {
        self.arc_set
            .insert([a.src().to_string(), a.dst().to_string()].join(" "));
        self.arcs.push(a);

        self.invalid_arcs.push(false);
    }

    pub fn push_face(&mut self, f: Face) {
        self.faces.push(f);
        self.invalid_faces.push(false);
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
            if self.invalid_faces[f] {
                continue;
            }
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

    fn remove_arc(&mut self, id: ArcId) {
        let ap = self.arc(id).prev();
        let an = self.arc(id).next();
        let af = self.arc(id).face();
        if self.face(af).start_arc() == id {
            self.faces[af].set_start_arc(an);
        }
        self.arcs[ap].set_next(an);
        self.arcs[an].set_prev(ap);

        let src = self.arc(id).src();
        let twin = self.arc(an).twin();
        self.arcs[twin].reset_dst(src);
        if self.arc(ap).src() == self.arc(an).dst() {
            // we collapsed a triangle into a line
            let face = self.arc(id).face();
            self.invalid_faces[face] = true;
        }
        self.invalid_arcs[id] = true;
    }
    /// merge vertex from into vertex into
    fn merge_vertices(&mut self, into: VertexId, from: VertexId) {
        /* gather neighbors of u and v and the position of each other */
        let neighbors_of_into: Vec<VertexId> = self.neighbors(into);
        let neighbors_of_from: Vec<VertexId> = self.neighbors(from);
        let position_of_into: usize = match neighbors_of_from
            .iter()
            .position(|&neighbor| neighbor == into)
        {
            Some(v) => v,
            None => {
                panic!("cannot merge not adjacent vertices into: {into} and from: {from}")
            }
        };
        let position_of_from: usize = neighbors_of_into
            .iter()
            .position(|&neighbor| neighbor == from)
            .unwrap();

        /* collect bend over and deleted arcs */
        let mut bend_over_arcs: Vec<ArcId> = Vec::new();
        let mut bend_over_twins: Vec<ArcId> = Vec::new();
        let into_to_from = self.vertices[into].arcs()[position_of_from];
        let from_to_into = self.vertices[from].arcs()[position_of_into];

        // update src of all remaining arcs of from
        // update dst of all their twins
        // Add them to into
        let mut arcs = self.vertices[from].arcs().clone();
        for a in arcs.into_iter() {
            self.arcs[a].reset_src(into);
            let twin = self.arcs[a].twin();
            self.arcs[twin].reset_dst(into);
            self.vertices[into].push_arc(a);
        }
        // remove u_v, v_u
        self.remove_arc(into_to_from);
        self.remove_arc(from_to_into);
    }

    pub fn find_rings(&self) -> Result<Vec<SubDcel>, Box<dyn Error>> {
        let mut result = vec![];
        let spanning_tree = self.spanning_tree(0);

        for depth in 1..(spanning_tree.max_level() + 1) {
            let mut visited = vec![false; self.vertices.len()];

            let mut builder = SubDcelBuilder::new(self.clone(), depth);

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
                        if dst_level == depth {
                            builder.push_arc(outgoing_arc);
                        }
                    }
                }
            }
            let resulting_sub_dcel = builder.build(None)?;
            result.push(resulting_sub_dcel);
        }

        Ok(result)
    }

    pub fn collect_donut(
        &self,
        start: usize,
        end: usize,
        // collapsed_dcel: &DcelBuilder,
        collapsed_root: VertexId,
    ) -> Result<SubDcel, Box<dyn Error>> {
        let spanning_tree = self.spanning_tree(0);

        if end > spanning_tree.max_level() + 1 {
            return Err("Donut is out of bounds".into());
        }

        let mut visited = vec![false; self.vertices.len()];
        let mut builder = SubDcelBuilder::new(self.clone(), start);

        // add collapsed_root as fake_root and push all its arcs
        // collapsed_dcel
        //     .arcs(collapsed_root)
        //     .iter()
        //     .map(|id| Arc::from(collapsed_dcel.arc(*id)))
        //     .for_each(|a| builder.push_arc(&a));

        for vertex in 0..self.vertices().len() {
            let vertex_depth = spanning_tree.vertex_level()[vertex];

            if vertex_depth >= start && vertex_depth < end && !visited[vertex] {
                /* This vertex is part of the donut, so add all its associated arcs in the
                 * donut */
                let outgoing_arcs = self.vertices[vertex]
                    .arcs()
                    .iter()
                    .map(|arc_id| self.arc(*arc_id))
                    .collect::<Vec<_>>();

                for arc in outgoing_arcs {
                    if spanning_tree.vertex_level()[arc.dst()] >= start
                        && spanning_tree.vertex_level()[arc.dst()] < end
                    {
                        builder.push_arc(arc);
                    } else if spanning_tree.discovered_by(vertex).src() == arc.dst() {
                        let mut copy = arc.clone();
                        copy.reset_dst(collapsed_root);
                        builder.push_arc(&copy);
                    }
                }

                // TODO:
                // 1. Dcel into Dcel BUilder
                // 2. Merge all vertices inside the donut hole
                // 3. use result as fake root
                visited[vertex] = true;
            }
        }

        let sub_dcel = builder.build(Some(collapsed_root))?;
        Ok(sub_dcel)
    }

    pub fn find_donuts_for_k(&self, k: usize) -> Result<Vec<SubDcel>, Box<dyn Error>> {
        let mut result = vec![];
        let root = 0;
        // let mut clone = self.clone();
        let spanning_tree = self.spanning_tree(root);
        // let mut collapsed_dcel_builder = DcelBuilder::from(self);

        let mut last_level = 1;

        for n in 1..(spanning_tree.max_level() + 1) {
            println!("Find Donuts: Going through level {}", n);
            // if n > 1 {
            if false {
                // Collapse all nodes on the level before into the root node
                // to create a fake root
                // spanning_tree
                //     .on_level(n - 1)
                //     .iter()
                //     .for_each(|v| self.merge_vertices(root, *v));
            }
            if n % k == 0 {
                /* Current donut is from last_level -> n */
                let mut donut = self.collect_donut(last_level, n, root)?;
                donut.triangulate();
                result.push(donut);
                last_level = n + 1
            }
        }

        if last_level != spanning_tree.max_level() {
            let mut last_donut =
                self.collect_donut(last_level, spanning_tree.max_level() + 1, root)?;
            last_donut.triangulate();
            result.push(last_donut);
        }

        Ok(result)
    }

    pub fn pre_triangulation_arc_count(&self) -> usize {
        self.pre_triangulation_arc_count
    }
}
#[cfg(test)]
mod tests {
    use crate::{read_graph_file_into_dcel_builder, write_web_file};

    #[test]
    fn adjacency_matrix() {
        let mut dcel_b = read_graph_file_into_dcel_builder("data/tree.graph").unwrap();
        let dcel = dcel_b.build();
        let am = dcel.adjacency_matrix();
        println!("{:?}", am)
    }
    #[test]
    fn merge_vertices() {
        let mut dcel_b = read_graph_file_into_dcel_builder("data/merge_test.graph").unwrap();
        let mut dcel = dcel_b.build();
        dcel.merge_vertices(0, 7);
        write_web_file("data/test.js", &dcel);
        // TODO: merge v7 into v0
    }
}
