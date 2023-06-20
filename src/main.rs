use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
pub mod graph;
use graph::dcel::SpanningTree;
use graph::dcel_file_writer::JsDataWriter;
use graph::dual_graph::DualGraph;
use graph::iterators::bfs::BfsIter;
use graph::{Dcel, DcelBuilder};
use arboretum_td::tree_decomposition::TreeDecomposition;
use graph::tree_decomposition::{NiceTreeDecomposition, td_make_nice};

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
    let mut writer = JsDataWriter::new(filename, dcel);
    writer.write_data()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let dcel = read_graph_file(&args[1]).unwrap();
    let mut spanning_tree = SpanningTree::new(&dcel);
    spanning_tree.build(0);
    let mut dual_graph = DualGraph::new(&spanning_tree);
    dual_graph.build();
    let tree_decomposition = TreeDecomposition::from(&dual_graph);
    println!("{tree_decomposition:?}");

    let nice_td = tree_decomposition.make_nice();
    // let nice_td = td_make_nice(&tree_decomposition);

    let mut found = vec![false; 18];

    for bag in nice_td.bags.iter() {
        found[bag.id] = true;
        assert_eq!(true, nice_td.bags.iter().any(|bag| {
            bag.id == 0
        }));
        println!("{:?}", bag);
    }

    println!("Found = {found:?}");
    assert!(found.iter().all(|f| *f == true));

    write_web_file("data/test.js", &dcel);

    for a in BfsIter::new(&dcel,0) {
        print!("{:?}", a);
    }

    let mut st =  SpanningTree::new(&dcel);
    st.build(0);

    let mut dg = DualGraph::new(&st);
    dg.build();

    println!("{:?}", dg);
}
