use std::ffi::c_ushort;
use std::ops::Index;
use crate::graph::dcel::arc::ArcId;
use crate::graph::dcel::face::FaceId;
use crate::graph::dcel::vertex::VertexId;
use crate::graph::reducible::Reducible;
use super::dcel::Dcel;
use super::dcel;
use super::types::*;

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

        // If src does not exist, add all missing vertices
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

    pub fn build(&mut self) -> Dcel{
        self.set_dest_ports();
        self.build_faces();
        let mut dcel = Dcel::new();
        for v in &(self.vertices) {
            dcel.push_vertex(dcel::vertex::Vertex::new(&v.arcs));
        }
        for a in &(self.arcs) {
            dcel.push_arc(dcel::arc::Arc::new(a.src, a.dst, a.next.unwrap(), a.prev.unwrap(), a.twin.unwrap(), a.face.unwrap()))
        }
        for f in &(self.faces) {
            dcel.push_face(dcel::face::Face::new(f.start_arc))
        }
        dcel
    }

    fn set_dest_ports(&mut self){
        for i in 0..self.arcs.len() {
            let twin = self.arcs[i].twin.unwrap();
            let src_port = self.arcs[i].src_port;

            self.arcs[twin].dst_port = src_port;
        }
    }

    fn build_faces(&mut self){

        let mut visited_arcs = vec![false; self.arcs.len()];
        
        for i in 0..self.arcs.len() {
            if visited_arcs[i] {continue;}
            visited_arcs[i] = true;
            self.faces.push(Face::new(i));
            let current_face_idx = self.faces.len()-1;

            self.arcs[i].face = Some(current_face_idx);
            let mut prev_arc_idx =  i;
            let mut next_arc_idx = self.find_next_arc(i);
            while !visited_arcs[next_arc_idx] {
                visited_arcs[next_arc_idx] = true;
                self.arcs[next_arc_idx].face = Some(current_face_idx);
                self.arcs[next_arc_idx].prev = Some(prev_arc_idx);
                self.arcs[prev_arc_idx].next = Some(next_arc_idx);
                prev_arc_idx = next_arc_idx;
                next_arc_idx = self.find_next_arc(next_arc_idx);
            }
            self.arcs[prev_arc_idx].next = Some(next_arc_idx);
            self.arcs[next_arc_idx].prev = Some(prev_arc_idx);
             
        }


    }

    /**
    * Returns the index of the next arc in the face
    */
    fn find_next_arc(&mut self, cur_arc_idx : usize) -> usize{
        let arc = &mut self.arcs[cur_arc_idx];
        let dest_port = arc.dst_port.unwrap();

        let dest_v = &self.vertices[arc.dst];

        let next_port = match dest_port == dest_v.arcs.len()-1 {
            true => 0,
            false => dest_port + 1
        };


        dest_v.arcs[next_port]
    }


    /* decrease indices of elements when elements with smaller index are removed */
    fn decrease_index(index: usize, removed_indices: &Vec<usize>) -> usize {
        let smaller_indices: Vec<VertexId> = removed_indices.iter()
            .filter(|&&removed_index| removed_index < index).map(|&element| element).
            collect();
        index - smaller_indices.len()
    }

    fn get_neighborhood(&self, vertex: VertexId) -> Vec<VertexId> {
        self.vertices[vertex].arcs.iter().map(|&arc_index| {
            self.arcs[arc_index].dst
        }).collect()
    }

    fn remove_vertex_and_arcs(&mut self, removed_vertex: VertexId, removed_arcs: &mut Vec<ArcId>) {
        /* remove arcs */
        removed_arcs.sort();
        removed_arcs.reverse();
        for i in 0..removed_arcs.len() {
            self.arcs.remove(removed_arcs[i]);
        }

        /* remove vertex */
        self.vertices.remove(removed_vertex);

        /* remove ports */
        for mut vertex in &mut self.vertices {
            vertex.arcs.retain(|arc| !removed_arcs.contains(arc));
        }

        /* update vertices */
        for mut vertex in &mut self.vertices {
            vertex.arcs = vertex.arcs.iter().map(|&arc_index|
                DcelBuilder::decrease_index(arc_index, &removed_arcs)
            ).collect();
        }

        /* update arcs */
        for vertex in &mut self.vertices {
            for index in 0..vertex.arcs.len() {
                let mut arc: &mut Arc = &mut self.arcs[vertex.arcs[index]];
                arc.src = DcelBuilder::decrease_index(arc.src, &vec![removed_vertex]);
                arc.dst = DcelBuilder::decrease_index(arc.dst, &vec![removed_vertex]);
                arc.src_port = Some(index);
                arc.twin = match arc.twin {
                    Some(twin) => {
                        Some(DcelBuilder::decrease_index(
                            twin, &removed_arcs)
                        )},
                    None => None
                };
            }
        }
    }
}

impl Reducible for DcelBuilder{
    fn remove_vertex(&mut self, u: usize) {
        /* find all arcs to be removed */
        let mut arcs_to_be_removed: Vec<usize> = self.arcs.iter().enumerate()
            .filter(|(_, &ref arc)| arc.src == u || arc.dst == u).map(|(i, _)| i).collect();

        self.remove_vertex_and_arcs(u, &mut arcs_to_be_removed);
    }

    fn merge_vertices(&mut self, u: usize, v: usize) {
        /* we can assume that both vertices are adjacent, because we merge only adjacent vertices */

        /* gather neighbors of u and v and the position of each other */
        let neighbors_of_u: Vec<VertexId> = self.get_neighborhood(u);
        let neighbors_of_v: Vec<VertexId> = self.get_neighborhood(v);
        let position_of_u: usize = neighbors_of_v.iter().
            position(|&neighbor| neighbor == u).unwrap();
        let position_of_v: usize = neighbors_of_u.iter().
            position(|&neighbor| neighbor == v).unwrap();

        /* collect bend over and deleted arcs */
        let mut bend_over_arcs: Vec<ArcId> = Vec::new();
        let mut bend_over_twins: Vec<ArcId> = Vec::new();
        let mut deleted_arcs: Vec<ArcId> = vec![
            self.vertices[u].arcs[position_of_v],
            self.vertices[v].arcs[position_of_u]
        ];

        /* check outgoing arcs of v */
        for index in 0..(self.vertices[v].arcs.len() - 1) {
            /* retrieve arc */
            let index: usize = (position_of_u + 1 + index) % self.vertices[v].arcs.len();
            let arc_index: ArcId = self.vertices[v].arcs[index];
            let mut arc: &mut Arc = &mut self.arcs[arc_index];

            /* delete or bend over arc */
            if neighbors_of_u.contains(&arc.dst) {
                deleted_arcs.push(arc_index);
                deleted_arcs.push(arc.twin.unwrap());
            } else {
                arc.src = u;
                bend_over_arcs.push(arc_index);
                bend_over_twins.push(arc.twin.unwrap());
            }
        }

        /* bend over ingoing arcs of v */
        for arc_index in bend_over_twins {
            self.arcs[arc_index].dst = v;
        }

        /* update vertex u */
        self.vertices[u].arcs.remove(position_of_v);
        for (index, arc_index) in bend_over_arcs.iter().enumerate() {
            self.vertices[u].arcs.insert(position_of_v + index, *arc_index);
        }

        /* remove arcs and vertex v */
        self.remove_vertex_and_arcs(v, &mut deleted_arcs);
    }
}
