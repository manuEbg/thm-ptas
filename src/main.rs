use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
pub mod graph;
use graph::dcel_file_writer::DcelWriter;
use graph::iterators::bfs::BfsIter;
use graph::{Dcel, DcelBuilder};

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
    let args: Vec<String> = env::args().collect();
    let dcel = read_graph_file(&args[1]).unwrap();
    write_web_file("data/test.js", &dcel);

    for a in BfsIter::new(&dcel,0) {
        print!("{:?}", a);
    }
}
