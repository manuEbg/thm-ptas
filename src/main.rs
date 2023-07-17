use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
pub mod graph;
use graph::dcel::spanning_tree::SpanningTree;
use graph::dcel_file_writer::JsDataWriter;
use graph::dual_graph::DualGraph;
use graph::iterators::bfs::BfsIter;

use graph::{Dcel, DcelBuilder};
use graph::quick_graph::QuickGraph;
use graph::reducible::Reducible;
use graph::reductions::*;
use crate::graph::reductions::isolated_clique_reduction::{do_isolated_clique_reductions, transfer_isolated_clique};
use crate::graph::reductions::nodal_fold_reduction::{do_nodal_fold_reductions, transfer_nodal_fold_reduction};
use crate::graph::reductions::twin_reduction::{do_twin_reductions, transfer_twin_reductions};

fn read_graph_file_into_quick_graph(filename: &str) -> Result<QuickGraph, String> {
    return if let Ok(mut lines) = read_lines(filename) {
        /* create datastructure for graph */
        let mut graph: QuickGraph;
        let vertex_count: usize = lines.next().unwrap().unwrap().parse().unwrap();
        graph = QuickGraph::new(vertex_count);
        graph.edge_count = lines.next().unwrap().unwrap().parse().unwrap();

        /* read in edges */
        for _ in 0..(2 *graph.edge_count) {
            let edge = lines.next().unwrap().unwrap();
            let mut edge = edge.split(" ");
            let u: usize = edge.next().unwrap().parse().unwrap();
            let v: usize = edge.next().unwrap().parse().unwrap();
            if let Some(ref mut adjacency_u) = &mut graph.adjacency[u] {
                adjacency_u.push(v);
            } else {
                return Err(String::from("Could not push neighbor to adjacency list. "));
            }
        }

        Ok(graph)
    } else {
        Err(format!("Could not open file {}", filename))
    };
}

fn read_graph_file_into_dcel(filename: &str) -> Result<Dcel, String> {
    return if let Ok(mut lines) = read_lines(filename) {
        /* build datastructure for DECL */
        let mut dcel_builder: DcelBuilder;
        if let Some(Ok(_)) = lines.next() {
            dcel_builder = DcelBuilder::new();
            let edge_count: usize = lines.next().unwrap().unwrap().parse().unwrap();

            /* read edges into DECL */
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

        /* build and return DECL */
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
    let mut writer = JsDataWriter::new(filename, dcel);
    writer.write_data()
}

fn main() {

    let args: Vec<String> = env::args().collect();
    let mut dcel = read_graph_file(&args[1]).unwrap();

    for a in BfsIter::new(&dcel, 0) {
        print!("{:?}", a);
    }

    //let mut st =  SpanningTree::new(&dcel);
    // st.build(0);

    //dcel.triangulate();

    write_web_file("data/test.js", &dcel);
    // let mut dg = DualGraph::new(&st);
    // dg.build();

    println!("{:?}", dcel);

}

