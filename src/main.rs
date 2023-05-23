use crate::graph::PlanarGraph;

mod graph;

fn main() {
    let mut graph:PlanarGraph = PlanarGraph::new(5);
    println!("{:?}", graph);
    graph.add_edge(0, 1);
    println!("{:?}", graph);
    graph.add_edge(0, 2);
    println!("{:?}", graph);
    graph.add_edge(0, 4);
    println!("{:?}", graph);
    graph.add_edge(1, 3);
    println!("{:?}", graph);
    graph.add_edge_at(0, 2, 3, 1);
    println!("{:?}", graph);
    graph.remove_edge(0, 1);
    println!("{:?}", graph);
}
