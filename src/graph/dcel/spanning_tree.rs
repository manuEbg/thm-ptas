use super::BfsIter;
use super::{ArcId, Dcel, VertexId};

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
                let twin = self.dcel.arc(a).twin();
                self.contains_arc[a] = true;
                self.contains_arc[twin] = true;
                self.arcs.push(a);
                self.arcs.push(twin);
                self.vertex_level[it.vertex] = it.level;
            }
        }
    }

    pub fn dcel(&self) -> &Dcel {
        self.dcel
    }

    pub fn arcs(&self) -> &Vec<ArcId> {
        &self.arcs
    }

    pub fn num_arcs(&self) -> usize {
        self.arcs.len()
    }

    pub fn contains_arc(&self, arc: ArcId) -> bool {
        self.contains_arc[arc]
    }
}
