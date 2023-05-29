use super::iterators::bfs::BfsIter;

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
    contains_arc : Vec<bool>,
    arcs : Vec<usize>,
}

impl<'a> SpanningTree<'a> {
    pub fn new(dcel: &'a Dcel) -> Self {
        Self {
            dcel,
            contains_arc: vec![false; dcel.num_arcs()],
            arcs: vec![],
        }
    }

    pub fn build(&mut self, start: usize) {
        let mut iterator = BfsIter::new(self.dcel, start);
        while let Some(it) = iterator.next()  {
            if let Some(a) = it.arc {
                let twin = self.dcel.get_arc(a).twin;
                self.contains_arc[a] = true;
                self.contains_arc[twin] = true;
                self.arcs.push(a);
                self.arcs.push(twin);
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
}
