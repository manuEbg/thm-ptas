use std::error::Error;

use super::{iterators::bfs::BfsIter, DcelBuilder};

#[derive(Debug)]
pub struct Face {
    start_arc: usize,
}

impl Face {
    pub fn new(start_arc: usize) -> Self {
        Face { start_arc }
    }

    pub fn walk_face(&self, dcel: &Dcel) -> Vec<usize> {
        let mut arcs = vec![];
        arcs.push(self.start_arc);
        let mut current_arc = dcel.get_arc(self.start_arc).get_next();
        while current_arc != self.start_arc {
            arcs.push(current_arc);
            current_arc = dcel.get_arc(current_arc).get_next();
        }
        arcs
    }
}

#[derive(Debug)]
pub struct Arc {
    src: usize,
    dst: usize,
    next: usize,
    prev: usize,
    twin: usize,
    face: usize,
}

impl Arc {
    pub fn new(src: usize, dst: usize, next: usize, prev: usize, twin: usize, face: usize) -> Self {
        Arc {
            src,
            dst,
            next,
            prev,
            twin,
            face,
        }
    }

    pub fn get_next(&self) -> usize {
        self.next
    }

    pub fn get_src(&self) -> usize {
        self.src
    }

    pub fn get_dst(&self) -> usize {
        self.dst
    }

    pub fn get_twin(&self) -> usize {
        self.twin
    }

    pub fn get_face(&self) -> usize {
        self.face
    }
}

#[derive(Debug)]
pub struct Vertex {
    arcs: Vec<usize>,
}

#[derive(Debug)]
pub struct SpanningTree<'a> {
    dcel: &'a Dcel,
    contains_arc: Vec<bool>,
    vertex_level: Vec<usize>,
    arcs: Vec<usize>,
}

impl<'a> SpanningTree<'a> {
    pub fn new(dcel: &'a Dcel) -> Self {
        Self {
            dcel,
            contains_arc: vec![false; dcel.num_arcs()],
            arcs: vec![],
            vertex_level: vec![0; dcel.num_vertices()],
        }
    }

    pub fn build(&mut self, start: usize) {
        let mut iterator = BfsIter::new(self.dcel, start);
        while let Some(it) = iterator.next() {
            if let Some(a) = it.arc {
                let twin = self.dcel.get_arc(a).twin;
                self.contains_arc[a] = true;
                self.contains_arc[twin] = true;
                self.arcs.push(a);
                self.arcs.push(twin);
                self.vertex_level[it.vertex] = it.level;
            }
        }
    }

    pub fn get_dcel(&self) -> &Dcel {
        self.dcel
    }

    pub fn get_arcs(&self) -> &Vec<usize> {
        &self.arcs
    }

    pub fn num_arcs(&self) -> usize {
        self.arcs.len()
    }

    pub fn contains_arc(&self, arc: usize) -> bool {
        self.contains_arc[arc]
    }
}


#[derive(Debug)]
pub struct SubDcel<'a> {
    pub dcel: &'a Dcel,
    pub sub: Dcel,
    mapping: Vec<usize>,
}

impl<'a> SubDcel<'a> {
    pub fn new(dcel: &'a Dcel, sub: Dcel, mapping: Vec<usize>) -> Self {
        Self { dcel, sub, mapping }
    }

    pub fn get_original_arc(&self, a: usize) -> Option<&usize> {
        self.mapping.get(a)
    }
}

impl Vertex {
    pub fn new(arcs: &Vec<usize>) -> Self {
        Vertex { arcs: arcs.clone() }
    }

    pub fn get_arcs(&self) -> &Vec<usize> {
        &self.arcs
    }
}

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

    pub fn walk_face(&self, face_idx: usize) -> Vec<usize> {
        self.faces[face_idx].walk_face(self)
    }

    pub fn get_arcs(&self) -> &Vec<Arc> {
        &self.arcs
    }

    pub fn get_arc(&self, idx: usize) -> &Arc {
        &self.arcs[idx]
    }

    pub fn get_faces(&self) -> &Vec<Face> {
        &self.faces
    }

    pub fn get_vertices(&self) -> &Vec<Vertex> {
        &self.vertices
    }

    pub fn get_vertex(&self, idx: usize) -> &Vertex {
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

    pub fn neighbors(&self, v: usize) -> Vec<usize> {
        let mut neighbors: Vec<usize> = vec![];
        for a in self.get_vertex(v).arcs.iter() {
            let n = self.get_arc(*a).get_dst();
            neighbors.push(n);
        }
        neighbors
    }

    pub fn spanning_tree(&self, start: usize) -> SpanningTree {
        let mut tree = SpanningTree::new(&self);
        tree.build(start);
        tree
    }

    pub fn get_twin(&self, arc: usize) -> &Arc {
        let twin = self.get_arc(arc).twin;
        self.get_arc(twin)
    }


    pub fn cut_rings(&self, k: usize) -> Result<Vec<SubDcel>, Box<dyn Error>> {
        let mut result = vec![];
        let spanning_tree = self.spanning_tree(0);

        for n in 1..(k+1) {
            let mut visited = vec![false; self.vertices.len()];

            //let mut ring_dcel = self.clone();
            let mut ring_dcel = DcelBuilder::new();
            for spanning_arc in &spanning_tree.arcs {
                let arc = self.get_arc(*spanning_arc);
                let src_level = spanning_tree.vertex_level[arc.get_src()];

                /* Is this vertex part of the ring? */
                if src_level == n && !visited[arc.get_src()] {
                    visited[arc.get_src()] = true;
                    let outgoing_arcs = self.get_arcs().iter().filter(|a| a.get_src() == arc.get_src()).collect::<Vec<_>>();
                    for outgoing_arc in outgoing_arcs {
                        /* Add only vertices with depth equal or less than the ring depth */
                        let dst_level = spanning_tree.vertex_level[outgoing_arc.get_dst()];
                        if dst_level <= n && !visited[outgoing_arc.get_dst()] {
                            //println!("{:?} {:?}", arc.get_src(), outgoing_arc.get_dst());
                            ring_dcel.push_arc(arc.get_src(), outgoing_arc.get_dst());
                            ring_dcel.push_arc(outgoing_arc.get_dst(), arc.get_src());
                        }
                    }
                }
            }

            let dcel = ring_dcel.build();
            /* Create an arc mapping */
            let mut mapping = vec![0; dcel.num_arcs()];
            /* This probably very slow */
            for (sub_arc_idx, sub_arc) in dcel.arcs.iter().enumerate() {
                for (main_arc_idx, main_arc) in self.arcs.iter().enumerate() {
                    if sub_arc.get_src() == main_arc.get_src() && sub_arc.get_dst() == main_arc.get_dst() {
                        mapping[sub_arc_idx] = main_arc_idx;
                    }
                }
            }
            println!("{:?}", mapping);
            result.push(SubDcel::new(self, dcel, mapping));
        }

        Ok(result)
    }
}
