pub struct Arc {
    src: usize,
    dst: usize,
    next: Option<usize>,
    prev: Option<usize>,
    twin: Option<usize>,
    face: Option<usize>,
}

impl Arc {
    pub fn new(src: usize, dst: usize) -> Self {
        Arc {
            src: src,
            dst: dst,
            next: None,
            prev: None,
            twin: None,
            face: None,
        }
    }
}

pub struct Face {
    start_arc: usize,
}

pub struct Vertex {
    arcs: Vec<usize>,
}

impl Vertex {
    pub fn new() -> Self {
        Vertex{
            arcs: vec![]
        }
    }
}
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
    pub fn push_arc(&mut self, src: usize, dst: usize) {
        self.arcs.push(Arc::new(src, dst));


        while self.vertices.len() <= src {

            self.vertices.push(Vertex::new());
        }
        self.vertices[src].arcs.push(self.arcs.len()-1);
        
        if self.vertices.len() > dst {
            // find and mark twin
        }
    }
}
