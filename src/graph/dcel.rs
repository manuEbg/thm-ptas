#[derive(Debug)]
pub struct Arc {
    src: usize,
    src_port: Option<usize>,
    dst: usize,
    dst_port: Option<usize>,
    next: Option<usize>,
    prev: Option<usize>,
    twin: Option<usize>,
    face: Option<usize>,
}

impl Arc {
    pub fn new(src: usize, dst: usize) -> Self {
        Arc {
            src,
            src_port: None,
            dst,
            dst_port: None,
            next: None,
            prev: None,
            twin: None,
            face: None,
        }
    }
}

#[derive(Debug)]
pub struct Face {
    start_arc: usize,
}

#[derive(Debug)]
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

    pub fn push_arc(&mut self, src: usize, dst: usize) {
        self.arcs.push(Arc::new(src, dst));
        let current_arc = self.arcs.len() - 1;

        while self.vertices.len() <= src {
            self.vertices.push(Vertex::new());
        }

        self.vertices[src].arcs.push(self.arcs.len()-1);

        if self.vertices.len() > dst {
            // find and mark twin

            for possible_twin in self.vertices[dst].arcs.iter() {
                if self.arcs[*possible_twin].dst == src {
                    self.arcs[current_arc].twin = Some(*possible_twin);
                    break;
                }
            }
        }
    }
}
