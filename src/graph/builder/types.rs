#[derive(Clone, Debug)]
pub struct Arc {
    pub src: usize,
    pub src_port: Option<usize>,
    pub dst: usize,
    pub dst_port: Option<usize>,
    pub next: Option<usize>,
    pub prev: Option<usize>,
    pub twin: Option<usize>,
    pub face: Option<usize>,
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

#[derive(Clone, Debug)]
pub struct Face {
    pub start_arc: usize,
}

impl Face {
    pub fn new(start_arc: usize) -> Self {
        Face { start_arc }
    }
}

#[derive(Clone, Debug)]
pub struct Vertex {
    pub arcs: Vec<usize>,
}

impl Vertex {
    pub fn new() -> Self {
        Vertex { arcs: vec![] }
    }
}
