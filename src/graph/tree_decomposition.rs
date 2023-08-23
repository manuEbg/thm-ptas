use std::{fs::File, io::Error, io::Write, process::Command};

use crate::graph::approximated_td::ApproximatedTD;
use arboretum_td::tree_decomposition::TreeDecomposition;
use fxhash::FxHashSet;

use super::{
    iterators::bfs::TreeDecompBfsIter,
    node_relations::{NodeParent, NodeRelations},
};

/// Creates a tree decomposition for the
/// [arboretum_td](https://docs.rs/arboretum-td/latest/arboretum_td/) library.
impl From<&ApproximatedTD<'_>> for TreeDecomposition {
    fn from(atd: &ApproximatedTD) -> Self {
        let mut result = TreeDecomposition {
            bags: vec![],
            root: Some(atd.root_bag()),
            max_bag_size: 0,
        };

        let mut max_bag_size = 0;

        atd.bags().iter().for_each(|bag| {
            if bag.len() > max_bag_size {
                max_bag_size = bag.len();
            }
            result.add_bag(FxHashSet::from_iter(bag.iter().copied()));
        });

        for i in 0..atd.adjacent().len() {
            let neighbors = &atd.adjacent()[i];
            for n in neighbors.iter() {
                result.add_edge(i, *n);
            }
        }

        result.max_bag_size = max_bag_size;
        result
    }
}

/// Writes a tree decomposition to a given dot file.
pub fn td_write_to_dot(
    title: &str,
    file: &mut File,
    td: &TreeDecomposition,
    node_relations: &NodeRelations,
) -> Result<(), Error> {
    writeln!(file, "graph {title} {{")?;

    let iter = TreeDecompBfsIter::new(&td);
    for bag in iter {
        let parent = node_relations.parent.get(&bag.id).unwrap();

        writeln!(
            file,
            "\tB{} [label=\"B{} {:?}\"];",
            bag.id, bag.id, bag.vertex_set
        )?;

        match parent {
            NodeParent::Fake => {}
            NodeParent::Real(parent) => {
                writeln!(file, "\tB{} -- B{};", parent, bag.id)?;
            }
        }
    }

    writeln!(file, "}}")
}

pub fn td_write_to_pdf(
    title: &str,
    path: &str,
    td: &TreeDecomposition,
    node_relations: &NodeRelations,
) {
    let mut dot_path = String::from(path);
    dot_path.push_str(".dot");
    let mut pdf_path = String::from(path);
    pdf_path.push_str(".pdf");

    let mut td_dot = File::create(dot_path.as_str()).unwrap();
    td_write_to_dot(title, &mut td_dot, &td, &node_relations).unwrap();

    match Command::new("dot")
        .args([
            "-Tpdf",
            dot_path.as_str(),
            "-o",
            pdf_path.as_str(),
        ])
        .spawn() {
            Ok(_) => {},
            Err(_) => println!("Could not log the TD. Reason: dot is not installed.\nIf you want to use it, you can install it with graphviz."),
        }
}
