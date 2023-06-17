use super::iterators::bfs::BfsIter;


#[derive(Debug)]
pub struct Face {
    start_arc: usize,
}

pub type FaceId = usize;

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
    src: VertexId,
    dst: VertexId,
    next: ArcId,
    prev: ArcId,
    twin: ArcId,
    face: FaceId,
}

pub type ArcId = usize; 

impl Arc {
    pub fn new(src: VertexId, dst: VertexId, next: ArcId, prev: ArcId, twin: ArcId, face: FaceId) -> Self {
        Arc {
            src,
            dst,
            next,
            prev,
            twin,
            face,
        }
    }

    pub fn get_next(&self) -> ArcId {
        self.next
    }

    pub fn get_src(&self) -> VertexId {
        self.src
    }

    pub fn get_dst(&self) -> VertexId {
        self.dst
    }

    pub fn get_twin(&self) -> ArcId {
        self.twin
    }

    pub fn get_face(&self) -> FaceId {
        self.face
    }

    pub fn set_face(&mut self, f: FaceId) {
        self.face = f;
    }
}

#[derive(Debug)]
pub struct Vertex {
    arcs: Vec<ArcId>,
}

pub type VertexId = usize;

#[derive(Debug)]
pub struct SpanningTree<'a> {
    dcel: &'a Dcel,
    contains_arc: Vec<bool>,
    vertex_level: Vec<usize>,
    arcs: Vec<ArcId>,
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

    pub fn build(&mut self, start: VertexId) {
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

    pub fn get_arcs(&self) -> &Vec<ArcId> {
        &self.arcs
    }

    pub fn num_arcs(&self) -> usize {
        self.arcs.len()
    }

    pub fn contains_arc(&self, arc: ArcId) -> bool {
        self.contains_arc[arc]
    }
}

impl Vertex {
    pub fn new(arcs: &Vec<ArcId>) -> Self {
        Vertex { arcs: arcs.clone() }
    }

    pub fn get_arcs(&self) -> &Vec<ArcId> {
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

    pub fn walk_face(&self, face: FaceId) -> Vec<ArcId> {
        self.faces[face].walk_face(self)
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
        for a in self.get_vertex(v).arcs.iter() {
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
        let twin = self.get_arc(arc).twin;
        self.get_arc(twin)
    }

    pub fn add_edge(from: VertexId, to: VertexId, prev: ArcId, next: ArcId, face: FaceId) {
        todo!()
    }
}
