#[derive(Debug)]
pub struct Arc {
    src: usize,
    src_port: usize,
    dst: usize,
    dst_port: usize,
    next: usize,
    prev: usize,
    twin: usize,
    face: usize,
}

impl Arc {
    pub fn new(
        src: usize, 
        src_port: usize, 
        dst: usize,
        dst_port: usize,
        next: usize,
        prev: usize,
        twin: usize,
        face: usize
    ) -> Self {
        Arc {
            src,
            src_port,
            dst,
            dst_port,
            next,
            prev,
            twin,
            face}
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

}
