pub type Vertex = u32;

#[derive(Debug)]
pub struct PlanarGraph {
    pub adjacency: Vec<Vec<Vertex>>
}

fn insert_after<T : Eq>(list: &mut Vec<T>, predecessor: T, element: T) {

    if let Some(index) = list.iter().position(|x| *x == predecessor) {
        list.insert(index + 1, element)
    }
}

impl PlanarGraph {
   pub fn new(vertex_count: usize) -> Self {
        let adjacency: Vec<Vec<Vertex>> = vec![Vec::new(); vertex_count];
        PlanarGraph {
            adjacency
        }
    }

    pub fn add_edge(&mut self, u: Vertex, v: Vertex) {
        if !self.adjacency[u as usize].contains(&v) {
            self.adjacency[u as usize].push(v);
        }
        if !self.adjacency[v as usize].contains(&u) {
            self.adjacency[v as usize].push(u)
        }
    }

    pub fn add_edge_at(
        &mut self,
        u: Vertex,
        predecessor_in_u: Vertex,
        v: Vertex,
        predecessor_in_v: Vertex
    ) {
        if !self.adjacency[u as usize].contains(&v) {
            insert_after(&mut self.adjacency[u as usize], predecessor_in_u, v);
        }

        if !self.adjacency[v as usize].contains(&u) {
            insert_after(&mut self.adjacency[v as usize], predecessor_in_v, u);
        }
    }

    pub fn remove_edge(&mut self, u: Vertex, v: Vertex) {
        if self.adjacency[u as usize].contains(&v) {
            self.adjacency[u as usize].retain(|x| *x != v)
        }
        if self.adjacency[v as usize].contains(&u) {
            self.adjacency[v as usize].retain(|x| *x != u)
        }
    }

    pub fn neighborhood(&self, u: Vertex) -> &[Vertex] {
        &self.adjacency[u as usize]
    }

    pub fn add_vertex(&mut self) {
        self.adjacency.push(Vec::new());
    }

    pub fn remove_vertex(&mut self, u: Vertex) {
        let mut neighborhood: Vec<Vertex> = Vec::new();
        for neighbor in self.neighborhood(u) {
            neighborhood.push(*neighbor);
        }
        for neighbor in neighborhood {
            self.remove_edge(u, neighbor)
        }
    }
}