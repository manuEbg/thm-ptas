use std::env;
use crate::graph::{PlanarGraph, Vertex};
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
mod graph;

fn read_graph_file(filename: &str) -> Result<Vec<PlanarGraph>, String>{
    return if let Ok(mut lines) = read_lines(filename) {
        let mut result: Vec<PlanarGraph> = Vec::new();
        let mut line;
        loop {
            line = lines.next();

            if line.is_none() {
                break;
            }

            if let Some(Ok(line)) = line {
                let mut recent_graph: PlanarGraph = PlanarGraph::new(line.parse().unwrap());
                let edge_count: usize = lines.next().unwrap().unwrap().parse().unwrap();
                for _ in 0..edge_count {
                    let edge = lines.next().unwrap().unwrap();
                    let mut edge = edge.split(" ");
                    let u: Vertex = edge.next().unwrap().parse().unwrap();
                    let v: Vertex = edge.next().unwrap().parse().unwrap();
                    recent_graph.add_edge(u, v);
                }
                result.push(recent_graph);
            } else {
                return Err(String::from("Error: Could not read line. "));
            }
        }

        Ok(result)
    } else {
        Err(format!("Could not open file {}", filename))
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", read_graph_file(&args[1]))
}
