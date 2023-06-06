use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
pub mod graph;
use graph::dcel_file_writer::DcelWriter;
use graph::iterators::bfs::BfsIter;
use graph::{Dcel, DcelBuilder};
use crate::graph::quick_graph::QuickGraph;

fn read_graph_file(filename: &str) -> Result<Dcel, String> {
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
    let mut graph = QuickGraph::new(6);
    graph.add_edge(0, 1);
    graph.add_edge(1, 2);
    graph.add_edge(1, 3);
    graph.add_edge(2, 4);
    graph.add_edge(3, 4);
    graph.add_edge(4, 5);

    graph.contract_edge(3, 4);
    println!("{:?}", graph);
}
