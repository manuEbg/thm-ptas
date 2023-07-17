pub(crate) trait Reducible {
    fn remove_vertex(&mut self, u: usize);
    fn merge_vertices(&mut self, u: usize, v: usize);
}