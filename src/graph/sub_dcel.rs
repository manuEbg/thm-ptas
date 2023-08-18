use std::error::Error;

use super::{
    dcel::{arc, vertex},
    Dcel, DcelBuilder,
};

#[derive(Clone, Debug)]
pub struct SubDcel {
    pub dcel: Dcel,
    pub sub: Dcel,
    pub arc_mapping: Vec<arc::ArcId>,
    pub vertex_mapping: Vec<vertex::VertexId>,
}

impl SubDcel {
    pub fn new(
        dcel: Dcel,
        sub: Dcel,
        arc_mapping: Vec<arc::ArcId>,
        vertex_mapping: Vec<vertex::VertexId>,
    ) -> Self {
        Self {
            dcel,
            sub,
            arc_mapping,
            vertex_mapping,
        }
    }

    pub fn get_original_arc(&self, a: arc::ArcId) -> Option<&arc::ArcId> {
        self.arc_mapping.get(a)
    }

    pub fn get_original_vertex(&self, v: vertex::VertexId) -> Option<&vertex::VertexId> {
        self.vertex_mapping.get(v)
    }

    pub fn get_vertices(&self) -> &Vec<vertex::Vertex> {
        return self.sub.vertices();
    }

    pub fn triangulate(&mut self) {
        self.sub.triangulate();
    }

    pub fn get_untriangulated_arcs(&self) -> Vec<arc::Arc> {
        return self.sub.arcs[0..self.sub.pre_triangulation_arc_count].to_vec();
    }

    pub fn get_triangulated_arcs(&self) -> Vec<arc::Arc> {
        let arc_len = self.sub.arcs().len();
        return self.sub.arcs[self.sub.pre_triangulation_arc_count..arc_len].to_vec();
    }

    pub fn was_triangulated(&self) -> bool {
        self.sub.pre_triangulation_arc_count() > 0
    }
}

#[derive(Debug)]
pub struct SubDcelBuilder {
    pub dcel: Dcel,
    pub dcel_builder: DcelBuilder,
    pub vertex_mapping: Vec<vertex::VertexId>,
    pub arc_mapping: Vec<arc::ArcId>,
    pub last_vertex_id: vertex::VertexId,
}

impl SubDcelBuilder {
    pub fn new(dcel: Dcel) -> Self {
        Self {
            dcel,
            dcel_builder: DcelBuilder::new(),
            vertex_mapping: vec![],
            arc_mapping: vec![],
            last_vertex_id: 0,
        }
    }

    /* Returns the mapped vertex id */
    pub fn push_vertex(&mut self, v: vertex::VertexId) -> vertex::VertexId {
        /* Is already mapped? */
        for (idx, vertex) in self.vertex_mapping.iter().enumerate() {
            if *vertex == v {
                return idx;
            }
        }

        /* Add this vertex */
        //self.vertex_mapping[self.last_vertex_id] = v;
        self.vertex_mapping.push(v);
        self.last_vertex_id += 1;

        self.last_vertex_id - 1
    }

    pub fn push_arc(&mut self, a: &arc::Arc) {
        let src = self.push_vertex(a.src());
        let dst = self.push_vertex(a.dst());
        self.dcel_builder.push_arc(src, dst);
    }

    pub fn build(&mut self) -> Result<SubDcel, Box<dyn Error>> {
        let final_dcel = self.dcel_builder.build();
        let mut arc_mapping = vec![0 as arc::ArcId; final_dcel.num_arcs()];

        /* This probably very slow */
        for (sub_arc_idx, sub_arc) in final_dcel.arcs.iter().enumerate() {
            for (main_arc_idx, main_arc) in self.dcel.arcs.iter().enumerate() {
                if self.vertex_mapping[sub_arc.src()] == main_arc.src()
                    && self.vertex_mapping[sub_arc.dst()] == main_arc.dst()
                {
                    arc_mapping[sub_arc_idx] = main_arc_idx;
                }
            }
        }

        Ok(SubDcel::new(
            self.dcel.clone(),
            final_dcel,
            arc_mapping,
            self.vertex_mapping.clone(),
        ))
    }
}
