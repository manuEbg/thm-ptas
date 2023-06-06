trait Reducible {
    fn remove_vertex(&mut self, u: usize);
    fn contract_edge(&mut self, u: usize, v: usize)
}