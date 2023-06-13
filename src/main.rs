use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
pub mod graph;
use graph::dcel_file_writer::DcelWriter;
use graph::iterators::bfs::BfsIter;
use graph::{Dcel, DcelBuilder};
use crate::graph::quick_graph::QuickGraph;
use graph::reducible::Reducible;

fn read_graph_file_into_quick_graph(filename: &str) -> Result<QuickGraph, String> {
    return if let Ok(mut lines) = read_lines(filename) {
        let mut graph: QuickGraph;
        let vertex_count: usize = lines.next().unwrap().unwrap().parse().unwrap();
        graph = QuickGraph::new(vertex_count);
        graph.edge_count = lines.next().unwrap().unwrap().parse().unwrap();
        for _ in 0..(2 *graph.edge_count) {
            let edge = lines.next().unwrap().unwrap();
            let mut edge = edge.split(" ");
            let u: usize = edge.next().unwrap().parse().unwrap();
            let v: usize = edge.next().unwrap().parse().unwrap();
            graph.adjacency[u].push(v);
        }
        Ok(graph)
    } else {
        Err(format!("Could not open file {}", filename))
    };
}

fn read_graph_file_into_dcel(filename: &str) -> Result<Dcel, String> {
    return if let Ok(mut lines) = read_lines(filename) {
        let mut dcel_builder: DcelBuilder;
        if let Some(Ok(_)) = lines.next() {
            dcel_builder = DcelBuilder::new();
            let edge_count: usize = lines.next().unwrap().unwrap().parse().unwrap();
            for _ in 0..(2 * edge_count) {
                let edge = lines.next().unwrap().unwrap();
                let mut edge = edge.split(" ");
                let u: usize = edge.next().unwrap().parse().unwrap();
                let v: usize = edge.next().unwrap().parse().unwrap();
                dcel_builder.push_arc(u, v);
            }
        } else {
            return Err(String::from("Error: Could not read line. "));
        }
        let dcel = dcel_builder.build();
        Ok(dcel)
    } else {
        Err(format!("Could not open file {}", filename))
    };
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn write_web_file(filename: &str, dcel: &Dcel) {
    let mut writer = DcelWriter::new(filename, dcel);
    writer.write_dcel()
}

fn main() {
    let mut graph: QuickGraph = read_graph_file_into_quick_graph("example_graphs.txt").unwrap();
    println!("{:?}", graph);
    graph.merge_vertices(2, 0);
    println!("{:?}", graph);
    graph = read_graph_file_into_quick_graph("example_graphs.txt").unwrap();
    println!("{:?}", graph);
    graph.merge_vertices(0, 1);
    println!("{:?}", graph);
}
