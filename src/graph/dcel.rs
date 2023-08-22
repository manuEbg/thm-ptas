pub mod arc;
pub mod face;
pub mod spanning_tree;
pub mod vertex;

use std::{collections::HashSet, error::Error};

use self::face::FaceIterator;
use super::{
    iterators::bfs::BfsIter,
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
                println!("Skipping Face {f} ");
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
                println!(
                    "FACE {} with edges {:?} iterated. Should never be here",
                    f, vec
                );
                FaceInfo::TriangulatedFace
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

    fn update_face(&mut self, id: ArcId) -> (ArcId, ArcId) {
        let ap = self.arc(id).prev();
        let an = self.arc(id).next();
        let af = self.arc(id).face();
        if self.face(af).start_arc() == id {
            self.faces[af].set_start_arc(an);
        }
        self.arcs[ap].set_next(an);
        self.arcs[an].set_prev(ap);
        (an, ap)
    }

    fn is_line(&self, a: ArcId, b: ArcId) -> bool {
        self.arc(a).next() == b && self.arc(a).prev() == b
    }

    fn remove_arc(&mut self, id1: ArcId, id2: ArcId) {
        let r1 = self.update_face(id1);
        let r2 = self.update_face(id2);
        let is_line1 = self.is_line(r1.0, r1.1);
        let is_line2 = self.is_line(r2.0, r2.1);

        if is_line1 && !is_line2 {
            println!("is line 1");
            self.invalid_arcs[r1.0] = true;
            self.invalid_arcs[r1.1] = true;
            let t0 = self.arcs[r1.0].twin();
            let t1 = self.arcs[r1.1].twin();
            self.arcs[r1.0].reset_twin(r1.1);
            self.arcs[t1].reset_twin(t0);
            self.arcs[r1.1].reset_twin(r1.0);
            self.arcs[t0].reset_twin(t1);
            self.invalid_faces[self.arcs[id1].face()] = true;
        } else if !is_line1 && is_line2 {
            println!("is line 2");
            self.invalid_arcs[r2.0] = true;
            self.invalid_arcs[r2.1] = true;
            let t0 = self.arcs[r2.0].twin();
            let t1 = self.arcs[r2.1].twin();
            self.arcs[r2.0].reset_twin(r2.1);
            self.arcs[r2.1].reset_twin(r2.0);
            self.arcs[t1].reset_twin(t0);
            self.arcs[t0].reset_twin(t1);
            self.invalid_faces[self.arcs[id2].face()] = true;
        } else if is_line1 && is_line2 {
            println!("is line 1 and 2");
            self.invalid_arcs[r1.0] = true;
            self.invalid_arcs[r1.1] = true;
            let t0 = self.arcs[r1.0].twin();
            let t1 = self.arcs[r1.1].twin();
            self.arcs[r1.0].reset_twin(r1.1);
            self.arcs[t1].reset_twin(t0);
            self.arcs[r1.1].reset_twin(r1.0);
            self.arcs[t0].reset_twin(t1);
            self.invalid_faces[self.arcs[id1].face()] = true;
            // println!("is line 2");
            self.invalid_arcs[r2.0] = true;
            self.invalid_arcs[r2.1] = true;
            let t0 = self.arcs[r2.0].twin();
            let t1 = self.arcs[r2.1].twin();
            self.arcs[r2.0].reset_twin(r2.1);
            self.arcs[r2.1].reset_twin(r2.0);
            self.arcs[t1].reset_twin(t0);
            self.arcs[t0].reset_twin(t1);
            self.invalid_faces[self.arcs[id2].face()] = true;
        } else {
            println!("no line");
        }

        // let src = self.arc(id1).src();

        // let twin_n = self.arc(an).twin();
        // let twin_p = self.arc(ap).twin();
        // if r1.0 == r2.1 && r1.1 == r2.0 {
        //     // we collapsed a triangle into a line
        //     self.invalid_arcs[twin_p] = true;
        //     self.invalid_arcs[twin_n] = true;
        //     // TODO: update Twins
        //     self.arcs[ap].reset_twin(an);
        //     self.arcs[an].reset_twin(ap);
        //     let face = self.arc(id1).face();
        //     println!("Face {face} is invalid");
        //     self.invalid_faces[face] = true;
        // }
        // self.arcs[twin_n].reset_dst(src);
        self.invalid_arcs[id1] = true;
        self.invalid_arcs[id2] = true;
        // self.invalid_arcs[self.arcs[id].twin()] = true;
    }

    /// merge vertex from into vertex into
    fn merge_vertices(&mut self, into: VertexId, from: VertexId) {
        println!("MERGE VERTEX {from} into {into}");
        if into == from {
            println!("merging v{from} into v{into} is not allowed!");
            return;
        }
        /* gather neighbors of u and v and the position of each other */
        let neighbors_of_into: Vec<VertexId> = self.neighbors(into);
        let neighbors_of_from: Vec<VertexId> = self.neighbors(from);
        let position_of_into: usize = match neighbors_of_from
            .iter()
            .position(|&neighbor| neighbor == into)
        {
            Some(v) => v,
            None => {
                println!("cannot merge not adjacent vertices into: {into} and from: {from}");
                return;
            }
        };
        let position_of_from: usize = neighbors_of_into
            .iter()
            .position(|&neighbor| neighbor == from)
            .unwrap();

        /* collect bend over and deleted arcs */
        let into_to_from = self.vertices[into].arcs()[position_of_from];
        let from_to_into = self.vertices[from].arcs()[position_of_into];
        println!(
            "v{into} arcs: {:?}",
            self.vertices[into]
                .arcs()
                .iter()
                .map(|a| self.arcs[*a])
                .collect::<Vec<_>>()
        );
        println!("Position of from: {position_of_from}");

        // update src of all remaining arcs of from
        // update dst of all their twins
        // Add them to into
        self.vertices[into].remove_arc_at(position_of_from);
        self.vertices[from].remove_arc_at(position_of_into);

        let arcs = self.vertices[from].arcs().clone();
        for a in arcs.into_iter().rev() {
            println!("pushing arc {:?}", self.arc(a));
            let twin = self.arcs[a].twin();
            self.arcs[a].reset_src(into);
            self.arcs[twin].reset_dst(into);

            self.arcs[a].reset_src(into);
            self.arcs[twin].reset_dst(into);
            // if (self.invalid_arcs[a]) {
            //     continue;
            // }
            if (self.neighbors(into).contains(&self.arc(a).dst())) {
                let ups = self.arc(a).dst();
                let upsi = self.neighbors(into).iter().position(|n| *n == ups).unwrap();
                self.vertices[into].push_arc_at(a, upsi);
                continue;
            }
            self.vertices[into].push_arc_at(a, position_of_from);
        }
        // remove u_v, v_u
        self.remove_arc(into_to_from, from_to_into);
        println!(
            "v{into} arcs: {:?}",
            self.vertices[into]
                .arcs()
                .iter()
                .map(|a| self.arcs[*a])
                .collect::<Vec<_>>()
        );
        println!("removing invalid arcs");
        for a in 0..self.num_vertices() {
            self.vertices[a].remove_invalid(&self.invalid_arcs);
        }
        // self.vertices[into].remove_invalid(&self.invalid_arcs);
        self.vertices[from].remove_arcs();
        println!(
            "v{into} arcs: {:?}",
            self.vertices[into]
                .arcs()
                .iter()
                .map(|a| self.arcs[*a])
                .collect::<Vec<_>>()
        );
    }

    pub fn find_rings(&self) -> Result<Vec<SubDcel>, Box<dyn Error>> {
        let mut result = vec![];
        // return Ok(result);
        let spanning_tree = self.spanning_tree(0);

        for depth in 1..(spanning_tree.max_level() + 1) {
            let mut visited = vec![false; self.vertices.len()];
            println!("building ring{depth}");

            let mut builder = SubDcelBuilder::new(self.clone(), depth);

            for (i, spanning_arc) in spanning_tree.arcs().iter().enumerate() {
                let arc = self.arc(*spanning_arc);
                // if self.invalid_arcs[*spanning_arc] {
                //     continue;
                // }
                let src_level = spanning_tree.vertex_level()[arc.src()];

                /* Is this vertex part of the ring? */
                if src_level == depth && !visited[arc.src()] {
                    visited[arc.src()] = true;

                    let outgoing_arcs = self
                        .arcs()
                        .iter()
                        .enumerate()
                        .filter(|(i, a)| a.src() == arc.src() && !self.invalid_arcs[*i])
                        .map(|(i, a)| a)
                        .collect::<Vec<_>>();
                    for outgoing_arc in outgoing_arcs {
                        /* Add ring arcs */
                        let dst_level = spanning_tree.vertex_level()[outgoing_arc.dst()];
                        if dst_level == depth {
                            println!("pushing arc into ring{depth} {outgoing_arc:?}");
                            builder.push_arc(outgoing_arc);
                        }
                    }
                }
            }
            let resulting_sub_dcel = builder.build(None, None)?;
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
        spanning_tree: &SpanningTree,
    ) -> Result<SubDcel, Box<dyn Error>> {
        let mut aId = 0;
        // let spanning_tree = self.spanning_tree(0);

        if end > spanning_tree.max_level() + 1 {
            return Err("Donut is out of bounds".into());
        }

        let mut visited = vec![false; self.vertices.len()];
        let mut builder = SubDcelBuilder::new(self.clone(), start);

        // add collapsed_root as fake_root and push all its arcs
        let fake_lvl = spanning_tree.vertex_level()[collapsed_root];
        if fake_lvl < start {
            self.vertex(collapsed_root)
                .arcs()
                .iter()
                .map(|id| (id, self.arc(*id)))
                .for_each(|(id, a)| {
                    if !self.invalid_arcs[*id] {
                        builder.push_arc(a);
                        // builder.push_arc(self.twin(*id));
                        println!("pushing fake(from root) and twin arc{aId} g{id} {:?}", a);
                        // println!("twin: {:?}", self.twin(*id));
                        aId += 2;
                    } else {
                        println!("not pushing fake(fromroot) arc g{id} {:?}", a);
                    }
                });
        }

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

                for (i, arc) in outgoing_arcs.iter().enumerate() {
                    let global_arc_id = self.vertices[vertex].arcs()[i];
                    // if builder.contains_arc(arc) {
                    //     continue;
                    // }
                    if spanning_tree.vertex_level()[arc.dst()] >= start
                        && spanning_tree.vertex_level()[arc.dst()] < end
                    {
                        if self.invalid_arcs[global_arc_id] {
                            println!(
                                "not pushing arc g{global_arc_id} {}",
                                self.vertices[vertex].arcs()[i]
                            );
                        } else {
                            println!("pushing arc{aId} g{global_arc_id} {:?}", arc);
                            aId += 1;
                            if self.invalid_arcs[self.arcs[global_arc_id].twin()]
                                || self.invalid_arcs[self.arcs[global_arc_id].next()]
                                || self.invalid_arcs[self.arcs[global_arc_id].prev()]
                            {
                                println!(
                                    "twin,next or prev of g{global_arc_id} {:?} is invalid",
                                    arc
                                );
                            }
                            builder.push_arc(arc);
                        }
                    } else if arc.dst() == collapsed_root {
                        println!("arc point to fake_root {arc:?}");
                        // if fake_lvl >= start && fake_lvl < end
                        //     || self.invalid_arcs[self.vertex(vertex).arcs()[i]]
                        // {
                        //     continue;
                        // }
                        builder.push_arc(&arc);
                        // println!("pushing fake(to root) arc{aId} g{global_arc_id} {:?}", arc);
                        // aId += 1;
                    }
                }

                visited[vertex] = true;
            }
        }

        let sub_dcel = builder.build(Some(collapsed_root), Some(start))?;
        for (i, a) in sub_dcel.dcel.arcs().iter().enumerate() {
            println!("Subdcelarcs({i}) {:?}", a);
        }
        Ok(sub_dcel)
    }

    pub fn find_donuts_for_k(
        &self,
        k: usize,
        i: usize,
        spanning_tree: &SpanningTree,
    ) -> Result<Vec<SubDcel>, Box<dyn Error>> {
        let mut result = vec![];
        let mut clone = self.clone();
        let root = spanning_tree.root();

        let mut last_level = 0;

        for n in 1..(spanning_tree.max_level() + 1) {
            if n % (k + 1) == i {
                println!("Find Donuts: level {last_level} to {n}");
                /* Current donut is from last_level -> n */
                let mut donut = clone.collect_donut(last_level, n, root, &spanning_tree)?;
                // todo:
                println!(
                    "arc count: {}, face count: {}",
                    donut.sub.num_arcs(),
                    donut.sub.num_faces()
                );
                for f in 0..donut.sub.num_faces() {
                    print!("face{f}:");
                    for a in donut.sub.walk_face(f) {
                        print!(" v{},", self.arc(*donut.get_original_arc(a).unwrap()).src())
                    }
                    println!();
                }
                donut.triangulate();
                println!(
                    "after triangulation arc count: {}, face count: {}",
                    donut.sub.num_arcs(),
                    donut.sub.num_faces()
                );
                for f in 0..donut.sub.num_faces() {
                    print!("face{f}:");
                    for a in donut.sub.walk_face(f) {
                        print!(" v{}", donut.vertex_mapping[donut.sub.arc(a).src()]);
                        // if a >= donut.sub.pre_triangulation_arc_count() {
                        //     println!("a{a} comes from triangulation");
                        //     continue;
                        // }
                        // print!(" v{},", self.arc(*donut.get_original_arc(a).unwrap()).src())
                    }
                    println!();
                }
                result.push(donut);

                // after creating the donut we merge it into the root of the tree to create a fake
                // root for the following donut
                for i in last_level..=n {
                    spanning_tree
                        .on_level(i)
                        .iter()
                        .for_each(|v| clone.merge_vertices(root, *v))
                }
                last_level = n + 1;
            }
        }

        if last_level < spanning_tree.max_level() + 1 {
            let mut last_donut = clone.collect_donut(
                last_level,
                spanning_tree.max_level() + 1,
                root,
                &spanning_tree,
            )?;
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

    use super::Dcel;

    #[test]
    fn adjacency_matrix() {
        let mut dcel_b = read_graph_file_into_dcel_builder("data/tree.graph").unwrap();
        let dcel = dcel_b.build();
        let am = dcel.adjacency_matrix();
        println!("{:?}", am)
    }

    #[test]
    fn merge_vertices_simple() {
        let mut dcel_b =
            read_graph_file_into_dcel_builder("data/simple_merge/graph.graph").unwrap();
        let mut dcel = dcel_b.build();
        for (i, a) in dcel.arcs().iter().enumerate() {
            println!("Arc{i}: {} {a:?} ", dcel.invalid_arcs[i]);
        }
        println!("merge ");
        dcel.merge_vertices(0, 1);
        // dcel.merge_vertices(0, 2);
        for (i, a) in dcel.arcs().iter().enumerate() {
            println!("Arc{i}: {} {a:?} ", dcel.invalid_arcs[i]);
        }
        // dcel.merge_vertices(0, 7);
        // dcel.merge_vertices(0, 6);
        let mut clone = dcel.clone();
        write_web_file("data/test.js", &clone);
    }

    #[test]
    fn merge_vertices_random() {
        let mut dcel_b =
            read_graph_file_into_dcel_builder("data/random_merge/graph.graph").unwrap();
        let mut dcel = dcel_b.build();
        show_relevant_stuff(&dcel);

        let mut clone = dcel.clone();
        let st = dcel.spanning_tree(0);
        show_relevant_stuff(&clone);
        write_web_file("data/test.js", &clone);
    }
    #[test]
    fn merge_vertices_circ() {
        let mut dcel_b =
            read_graph_file_into_dcel_builder("data/circular_merge/graph.graph").unwrap();
        let mut dcel = dcel_b.build();
        show_relevant_stuff(&dcel);

        let mut clone = dcel.clone();
        let st = dcel.spanning_tree(0);
        show_relevant_stuff(&clone);
        write_web_file("data/test.js", &clone);
    }

    fn show_relevant_stuff(g: &Dcel) {
        for (i, a) in g.arcs().iter().enumerate() {
            println!("Arc{i}: {} {a:?} ", g.invalid_arcs[i]);
        }
        for (i, a) in g.vertices().iter().enumerate() {
            println!("Vertex{i}: {a:?} ");
        }
    }

    #[test]
    fn merge_vertices_big() {
        let mut dcel_b =
            read_graph_file_into_dcel_builder("data/bigger_merge/graph.graph").unwrap();
        let mut dcel = dcel_b.build();
        for (i, a) in dcel.arcs().iter().enumerate() {
            println!("Arc{i}: {} {a:?} ", dcel.invalid_arcs[i]);
        }
        println!("merge ");
        dcel.merge_vertices(0, 7);
        for (i, a) in dcel.arcs().iter().enumerate() {
            println!("Arc{i}: {} {a:?} ", dcel.invalid_arcs[i]);
        }
        let mut clone = dcel.clone();
        // let st = dcel.spanning_tree(0);

        // for level in 1..6 {
        //     st.on_level(level)
        //         .iter()
        //         .for_each(|v| clone.merge_vertices(0, *v));
        // }
        write_web_file("data/test.js", &clone);
    }
}
