#[derive(Debug)]

struct Arc {
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
struct Face {
    start_arc: usize,
}

#[derive(Debug)]
struct Vertex {
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
pub struct DcelBuilder {
    vertices: Vec<Vertex>,
    arcs: Vec<Arc>,
    faces: Vec<Face>,
}

impl DcelBuilder {
    pub fn new() -> Self {
        DcelBuilder {
            vertices: vec![],
            arcs: vec![],
            faces: vec![],
        }
    }

    pub fn push_arc(&mut self, src: usize, dst: usize) {
        self.arcs.push(Arc::new(src, dst));
        let current_arc = self.arcs.len() - 1;

        // If src does not exist, add all missing vertecies
        while self.vertices.len() <= src {
            self.vertices.push(Vertex::new());
        }
        
        // Add arc to source vertex, and set src_port
        let src_v = &mut self.vertices[src]; 
        src_v.arcs.push(current_arc);
        let src_port = src_v.arcs.len() - 1;
        
        self.arcs[current_arc].src_port = Some(src_port);
        
        if self.vertices.len() > dst {
            // find and mark twin

            for possible_twin in self.vertices[dst].arcs.iter() {
                if self.arcs[*possible_twin].dst == src {
                    self.arcs[current_arc].twin = Some(*possible_twin);
                    self.arcs[*possible_twin].twin = Some(current_arc);
                    break;
                }
            }
        }
    }

    fn set_dest_ports(&mut self){
        for i in 0..self.arcs.len() {
            let twin = self.arcs[i].twin.unwrap();
            let src_port = self.arcs[i].src_port;

            self.arcs[twin].dst_port = src_port;
        }
    }
    
}
