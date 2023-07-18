use super::dcel::arc::ArcId;
use super::dcel::face::FaceId;
use super::dcel::vertex::VertexId;
use super::dcel::SubDcel;
use super::{dcel::face::Face, dcel::spanning_tree::SpanningTree, Dcel};

type BagId = usize;

#[derive(Debug)]
pub struct ApproximatedTD<'a> {
    graph: &'a Dcel,
    bags: Vec<Vec<VertexId>>,
    adjacent: Vec<Vec<BagId>>,
    root_bag: BagId,
}

impl<'a> From<&'a mut SubTDBuilder<'a>> for ApproximatedTD<'a> {
    fn from(value: &'a mut SubTDBuilder<'a>) -> Self {
        value.to_td()
    }
}

impl<'a> From<&'a mut TDBuilder<'a>> for ApproximatedTD<'a> {
    fn from(value: &'a mut TDBuilder) -> Self {
        value.to_td()
    }
}

impl<'a> ApproximatedTD<'a> {
    pub fn neighbours(&self, bag: BagId) -> &Vec<BagId> {
        &self.adjacent[bag]
    }

    pub fn adjacent(&self) -> &Vec<Vec<BagId>> {
        &self.adjacent
    }

    pub fn graph(&self) -> &Dcel {
        self.graph
    }

    pub fn num_bags(&self) -> usize {
        self.adjacent.len()
    }
}

pub struct TDBuilder<'a> {
    spanning_tree: &'a SpanningTree<'a>,
    main_graph: &'a Dcel,
    adjacent: Vec<Vec<BagId>>,
    bags: Vec<Vec<VertexId>>,
    on_tree_path: Vec<Vec<BagId>>,
}

impl<'a> TDBuilder<'a> {
    pub fn new(st: &'a SpanningTree) -> Self {
        TDBuilder {
            spanning_tree: st,
            main_graph: st.dcel(),
            adjacent: vec![vec![]; st.dcel().num_faces()],
            bags: vec![vec![]; st.dcel().num_faces()],
            on_tree_path: vec![vec![]; st.dcel().num_faces()],
        }
    }
}

impl<'a> TreeDecomposable for TDBuilder<'a> {
    fn spanning_tree_contains(&self, a: ArcId) -> bool {
        self.spanning_tree.contains_arc(a)
    }

    fn face(&self, f: FaceId) -> Vec<VertexId> {
        self.main_graph
            .walk_face(f)
            .iter()
            .map(|a| self.main_graph.arc(*a).src())
            .collect()
    }

    fn spanning_tree_to_root(&self, start_from: VertexId) -> &Vec<VertexId> {
        &self.on_tree_path[start_from]
    }

    fn vertex_mapping(&self, v: VertexId) -> VertexId {
        v
    }

    fn add_edge(&mut self, a: BagId, b: BagId) {
        self.adjacent[a].push(b);
    }

    fn add_src_vertex(&mut self, arc: ArcId, to: BagId) {
        self.add_vertex(self.main_graph.arc(arc).src(), to);
    }

    fn add_vertex(&mut self, v: VertexId, to: BagId) {
        self.bags[to].push(v);
    }

    fn get_graph(&self) -> &Dcel {
        self.main_graph
    }

    fn vertices(&self, bag: BagId) -> &Vec<VertexId> {
        &self.bags[bag]
    }

    fn to_td(&mut self) -> ApproximatedTD {
        self.build();
        ApproximatedTD {
            graph: self.main_graph,
            bags: self.bags.clone(),
            adjacent: self.adjacent.clone(),
            root_bag: 0,
        }
    }
}

pub struct SubTDBuilder<'a> {
    spanning_tree: &'a SpanningTree<'a>,
    donut: &'a SubDcel,
    adjacent: Vec<Vec<BagId>>,
    bags: Vec<Vec<VertexId>>,
    min_level: usize,
    on_tree_path: Vec<Vec<VertexId>>,
}

impl<'a> TreeDecomposable for SubTDBuilder<'a> {
    fn spanning_tree_contains(&self, a: ArcId) -> bool {
        if let Some(og_arc) = self.donut.get_original_arc(a) {
            self.spanning_tree_contains(*og_arc)
        } else {
            panic!("Arc {} not mapped. Never should be here!", a);
        }
    }

    fn face(&self, f: FaceId) -> Vec<VertexId> {
        fn global_arc_src(g: &SubTDBuilder, local_id: ArcId) -> VertexId {
            let d = g.donut;
            let a = match d.get_original_arc(local_id) {
                Some(res) => *res,
                None => {
                    panic!("Arc {} not mapped. Never should be here!", local_id)
                }
            };
            d.dcel.arc(a).src()
        }
        let donut = self.donut;
        donut
            .sub
            .walk_face(f)
            .iter()
            .map(|a| global_arc_src(self, *a))
            .collect()
    }

    fn spanning_tree_to_root(&self, start_from: VertexId) -> &Vec<VertexId> {
        &self.on_tree_path[start_from]
    }

    fn vertex_mapping(&self, v: VertexId) -> VertexId {
        let id = match self.donut.get_original_vertex(v) {
            Some(vert) => *vert,
            None => {
                panic!("Vertex {} not mapped. Never should be here!", v)
            }
        };
        id
    }

    fn add_edge(&mut self, a: BagId, b: BagId) {
        self.adjacent[a].push(b);
    }

    fn add_src_vertex(&mut self, arc: ArcId, to: BagId) {
        self.add_vertex(self.donut.sub.arc(arc).src(), to);
    }

    fn add_vertex(&mut self, v: VertexId, to: BagId) {
        let mapped_v = self.vertex_mapping(v);
        self.bags[to].push(mapped_v);
    }

    fn get_graph(&self) -> &Dcel {
        &self.donut.sub
    }

    fn vertices(&self, bag: BagId) -> &Vec<VertexId> {
        &self.bags[bag]
    }

    fn to_td(&mut self) -> ApproximatedTD {
        self.build();
        ApproximatedTD {
            graph: &self.donut.dcel,
            bags: self.bags.clone(),
            adjacent: self.adjacent.clone(),
            root_bag: 0,
        }
    }
}

impl<'a> SubTDBuilder<'a> {
    fn new(donut: &'a SubDcel, st: &'a SpanningTree, min_level: usize) -> Self {
        SubTDBuilder {
            spanning_tree: st,
            donut,
            adjacent: vec![vec![]; donut.sub.num_faces()],
            bags: vec![vec![]; donut.sub.num_faces()],
            min_level,
            on_tree_path: vec![vec![]; donut.sub.num_faces()],
        }
    }
}

trait TreeDecomposable {
    fn spanning_tree_contains(&self, a: ArcId) -> bool;

    /// returns the ids of all the vertices along the given face
    fn face(&self, f: FaceId) -> Vec<VertexId>;

    /// returns the ids of all the vertices on the path from start_from to the root of the
    /// spanning_tree
    fn spanning_tree_to_root(&self, start_from: VertexId) -> &Vec<VertexId>;

    /// returns the global id of a vertex igiven the local id of a vertex
    fn vertex_mapping(&self, v: VertexId) -> VertexId;

    /// adds an edge from Bag a to Bag b
    fn add_edge(&mut self, a: BagId, b: BagId);

    /// adds the source vertex of edge a to bag to
    fn add_src_vertex(&mut self, arc: ArcId, to: BagId);

    /// adds a vertex to a bag
    fn add_vertex(&mut self, v: VertexId, to: BagId);

    /// returns the graph used to build the TreeDecomposition
    fn get_graph(&self) -> &Dcel;

    /// returns the ids of all the vertices in a Bag
    fn vertices(&self, bag: BagId) -> &Vec<VertexId>;

    fn to_td(&mut self) -> ApproximatedTD;

    /// adds a face to the TreeDecomposition
    fn add_face(&mut self, face: &Face, bag: BagId) {
        for a in face.walk_face(self.get_graph()) {
            self.add_src_vertex(a, bag);

            if self.spanning_tree_contains(a) {
                continue;
            }
            let twin = self.get_graph().twin(a);
            self.add_edge(bag, twin.face());
        }
    }

    /// append vertices on the path back to the root of the spanning tree to each bag
    fn add_on_path_to_root(&mut self, bag: BagId) {
        let vs = self.vertices(bag).clone();
        for face_v in vs {
            let path_v = self.spanning_tree_to_root(face_v).clone();
            for v in path_v {
                self.add_vertex(v, bag);
            }
        }
    }

    /// builds the treeDecomposition
    fn build(&mut self) {
        let faces = self.get_graph().faces().clone();
        for (i, f) in faces.iter().enumerate() {
            self.add_face(f, i);
            self.add_on_path_to_root(i);
        }
    }
}
