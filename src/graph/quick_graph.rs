#[derive(Debug)]
pub struct QuickGraph {
    adjacency: Vec<Vec<usize>>,
    edge_count: usize
}

impl QuickGraph {
    pub fn new(vertex_count: usize) -> QuickGraph {
        let adjacency: Vec<Vec<usize>> = vec![Vec::new(); vertex_count];
        QuickGraph { adjacency, edge_count: 0 }
    }

    pub fn num_vertices(&self) -> usize { self.adjacency.len() }
    pub fn num_edges(&self) -> usize { self.edge_count }
    pub fn degree(&self, v: usize) -> usize { self.adjacency[v].len() }
    pub fn neighborhood(&self, v: usize) -> &Vec<usize> { &self.adjacency[v] }
    pub fn are_adjacent(&self, u: usize, v: usize) -> bool { self.adjacency[u].contains(&v)}

    pub fn add_vertex(&mut self) {
        self.adjacency.push(Vec::new())
    }

    pub fn add_edge(&mut self, u: usize, v: usize) {
        if !self.are_adjacent(u, v) {
            self.adjacency[u].push(v);
            self.adjacency[v].push(u);
            self.edge_count += 1;
        }
    }

    pub fn remove_edge(&mut self, u: usize, v:usize) {
        if self.are_adjacent(u, v) {
            self.adjacency[u].retain(|&vertex| vertex != v);
            self.adjacency[v].retain(|&vertex| vertex != u);
            self.edge_count -= 1;
        }
    }

    pub fn remove_vertex(&mut self, u: usize) {
        self.edge_count -= self.adjacency[u].len();
        self.adjacency.remove(u);
        self.adjacency = self.adjacency
            .iter()
            .map(|neighborhood| {
                neighborhood
                    .iter()
                    .map(|neighbor| if *neighbor > u {neighbor - 1} else {*neighbor} )
                    .collect()
            }).collect();
    }

    pub fn contract_edge(&mut self, u: usize, v: usize) {
        if self.adjacency[u].contains(&v) {
            let mut combined_neighbors: Vec<usize> = Vec::new();
            combined_neighbors.extend(self.adjacency[u].iter().take_while(|&&x| x != v));
            combined_neighbors.extend(self.adjacency[v].
                iter().filter(|&&x| x != u));
            combined_neighbors.extend(self.adjacency[u].iter().skip_while(|&&x| x != v).skip(1));
            self.adjacency[u] = combined_neighbors;
            for vertex in 0..self.adjacency.len() {
                if self.adjacency[vertex].contains(&v) {
                    if !self.adjacency[vertex].contains(&u) {
                        self.adjacency[vertex] = self.adjacency[vertex].iter()
                            .map(|&x| if x == v {u} else {x}).collect();
                    }
                }
            }
            self.remove_vertex(v);
            let double_edge_count: usize = self.adjacency.iter().map(|neighborhood| neighborhood.len()).sum() ;
            self.edge_count = double_edge_count / 2;
        }
    }
}