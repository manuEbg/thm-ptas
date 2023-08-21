use super::arc::Arc;
use super::BfsIter;
use super::{ArcId, Dcel, VertexId};

#[derive(Debug)]
pub struct SpanningTree<'a> {
    dcel: &'a Dcel,
    contains_arc: Vec<bool>,
    vertex_level: Vec<usize>,
    arcs: Vec<ArcId>,
    max_level: usize,
    discovered_by: Vec<ArcId>,
    root: VertexId,
}

impl<'a> SpanningTree<'a> {
    pub fn new(dcel: &'a Dcel) -> Self {
        Self {
            dcel,
            contains_arc: vec![false; dcel.num_arcs()],
            arcs: vec![],
            vertex_level: vec![0; dcel.num_vertices()],
            max_level: 0,
            discovered_by: vec![0; dcel.num_vertices()],
            root: 0,
        }
    }

    pub fn build(&mut self, start: VertexId) {
        self.root = start;
        let mut iterator = BfsIter::new(self.dcel, start);
        while let Some(it) = iterator.next() {
            if let Some(a) = it.arc {
                let twin = self.dcel.arc(a).twin();
                self.contains_arc[a] = true;
                self.contains_arc[twin] = true;
                println!("SpanTree: adding arc {a} and twin {twin}");
                self.arcs.push(a);
                self.arcs.push(twin);
                self.vertex_level[it.vertex] = it.level;
                if it.level > self.max_level {
                    self.max_level = it.level;
                }
                self.discovered_by[it.vertex] = a;
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

    pub fn root(&self) -> VertexId {
        self.root
    }

    pub fn vertex_level(&self) -> &[usize] {
        self.vertex_level.as_ref()
    }

    pub fn on_level(&self, level: usize) -> Vec<VertexId> {
        self.vertex_level
            .iter()
            .enumerate()
            .filter(|e| *e.1 == level)
            .map(|e| e.0)
            .collect()
    }

    pub fn max_level(&self) -> usize {
        self.max_level
    }

    pub fn discovered_by(&self, v: VertexId) -> &Arc {
        self.dcel.arc(self.discovered_by[v])
    }
}
