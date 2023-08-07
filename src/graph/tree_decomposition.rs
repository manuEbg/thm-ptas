use std::{fs::File, io::Error, io::Write};

use crate::graph::approximated_td::ApproximatedTD;
use arboretum_td::tree_decomposition::TreeDecomposition;
use fxhash::FxHashSet;

use super::{iterators::bfs::TreeDecompBfsIter, node_relations::{NodeRelations, NodeParent}};

/// This function is used to create a tree decomposition on one of the rings
/// in a DCEL data structure.
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

// TODO: This should probably be a trait that can be implemented.
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

    writeln!(file, "}}")?;

    Ok(())
}

/*
#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::graph::approximated_td::{ApproximatedTD, TDBuilder};
    use crate::graph::dcel::spanning_tree::SpanningTree;
    use crate::graph::node_relations::NodeRelations;
    use crate::read_graph_file_into_dcel_builder;
    use std::process::Command;


    #[test]
    fn test_find_mis() -> Result<(), Box<dyn std::error::Error>> {
        let mut td = TreeDecomposition {
            bags: Vec::new(),
            root: None,
            max_bag_size: 2,
        };

        let b0 = td.add_bag(FxHashSet::from_iter(vec![0, 1]));
        let b1 = td.add_bag(FxHashSet::from_iter(vec![2, 3]));
        let b2 = td.add_bag(FxHashSet::from_iter(vec![4, 5]));
        td.add_edge(b0, b1);
        td.add_edge(b0, b2);

        let td_rels = NodeRelations::new(&td);

        let td_path = "td.dot";
        let mut td_out = File::create(td_path)?;
        td_write_to_dot("td", &mut td_out, &td, &td_rels)?;
        Command::new("dot")
            .args(["-Tpdf", td_path, "-o", "td.pdf"])
            .spawn()
            .expect("dot command did not work.");

        let nice_td = td.make_nice(&td_rels);
        let ntd_rels = NodeRelations::new(&nice_td);
        assert!(validate_nice_td(&td, &nice_td, &ntd_rels));

        let ntd_path = "ntd.dot";
        let mut ntd_out = File::create(ntd_path)?;
        td_write_to_dot("ntd", &mut ntd_out, &nice_td, &ntd_rels)?;
        Command::new("dot")
            .args(["-Tpdf", ntd_path, "-o", "ntd.pdf"])
            .spawn()
            .expect("dot command did not work.");

        let (bag_content, size) = find_mis(vec![vec![false; 6]; 6], &nice_td, &ntd_rels)?;
        println!("{:?} with size = {}", bag_content, size);

        Ok(())
    }

    #[test]
    fn real() {
        let mut dcel_b = read_graph_file_into_dcel_builder("data/exp.graph").unwrap();
        let mut dcel = dcel_b.build();
        let adjacency_matrix = dcel.adjacency_matrix();
        // dcel.triangulate();
        let mut spanning_tree = SpanningTree::new(&dcel);
        spanning_tree.build(0);
        let mut td_builder = TDBuilder::new(&spanning_tree);
        let atd = ApproximatedTD::from(&mut td_builder);
        let td = TreeDecomposition::from(&atd);
        let td_rels = NodeRelations::new(&td);
        let ntd = td.make_nice(&td_rels);
        let ntd_rels = NodeRelations::new(&ntd);

        validate_nice_td(&td, &ntd, &ntd_rels);

        let td_path = "td.dot";
        let mut td_out = File::create(td_path).unwrap();
        td_write_to_dot("td", &mut td_out, &td, &td_rels).unwrap();
        Command::new("dot")
            .args(["-Tpdf", td_path, "-o", "td.pdf"])
            .spawn()
            .expect("dot command did not work.");

        let ntd_path = "ntd.dot";
        let mut ntd_out = File::create(ntd_path).unwrap();
        td_write_to_dot("ntd", &mut ntd_out, &ntd, &ntd_rels).unwrap();
        Command::new("dot")
            .args(["-Tpdf", ntd_path, "-o", "ntd.pdf"])
            .spawn()
            .expect("dot command did not work.");

        let (bag_content, size) = find_mis(adjacency_matrix, &ntd, &ntd_rels).unwrap();
        println!("{:?} with size = {}", bag_content, size);
    }

}
*/
