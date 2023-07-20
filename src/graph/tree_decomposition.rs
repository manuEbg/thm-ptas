use crate::graph::approximated_td::ApproximatedTD;
use arboretum_td::tree_decomposition::{Bag, TreeDecomposition};
use fxhash::FxHashSet;
use std::collections::{HashMap, VecDeque};

/// This function is used to create a tree decomposition on one of the rings
/// in a DCEL data structure.
impl From<&ApproximatedTD<'_>> for TreeDecomposition {
    fn from(approx_td: &ApproximatedTD) -> Self {
        let dcel = approx_td.graph();
        let faces = &dcel.faces();
        let mut result = TreeDecomposition {
            bags: vec![],
            root: None,
            max_bag_size: faces // TODO: Remove and count.
                .iter()
                .map(|face| face.walk_face(dcel).len())
                .fold(0, Ord::max),
        };

        for face in *faces {
            let mut vertices: FxHashSet<usize> = FxHashSet::default();
            for arc in face.walk_face(&dcel) {
                vertices.insert(dcel.arc(arc).src());
            }
            result.add_bag(vertices);
            if result.bags.len() == 1 {
                result.root = Some(0);
            }
        }

        for i in 0..approx_td.num_bags() {
            let neighbors = &approx_td.adjacent()[i];
            for n in neighbors {
                result.add_edge(i, *n);
            }
        }

        result
    }
}

#[derive(Clone, Copy)]
enum NodeParent {
    Fake,
    Real(usize),
}

// TODO: When the children are not needed, this struct can be replaced by the hash map itself.
// @speed The hash map could be replaced with a parent matrix.
struct NodeRelations {
    parent: HashMap<usize, NodeParent>,
    // children: HashMap<usize, Vec<usize>>,
}

impl NodeRelations {
    pub fn new(td: &TreeDecomposition) -> Self {
        let mut parent = HashMap::new();
        if let Some(root) = td.root {
            parent.insert(root, NodeParent::Fake);
        }
        Self {
            parent,
            // children: HashMap::new(),
        }
    }
}

struct BfsIter<'a> {
    td: &'a TreeDecomposition,
    queue: VecDeque<usize>,   // Bag IDs.
    visited: Vec<bool>,       // @speed Use a bitset.
    relations: NodeRelations, // Should be filled by the iterator.
}

impl<'a> BfsIter<'a> {
    pub fn new(td: &'a TreeDecomposition) -> Self {
        BfsIter {
            td,
            queue: VecDeque::from([td.root.unwrap()]),
            visited: vec![false; td.bags.len()],
            relations: NodeRelations::new(&td),
        }
    }
}

impl<'a> Iterator for BfsIter<'a> {
    type Item = (&'a Bag, NodeParent);

    fn next(&mut self) -> Option<Self::Item> {
        let front = self.queue.pop_front();
        if front.is_none() {
            return None;
        }

        let v = front.unwrap();
        self.visited[v] = true;
        let bag = &self.td.bags[v];
        bag.neighbors // Find all unvisited neighbors.
            .iter()
            .filter(|&&n| !self.visited[n])
            .for_each(|&n| {
                self.queue.push_back(n);
                // Every unvisited node thats parent is not set must be a child.
                if self.relations.parent.get(&n).is_none() {
                    self.relations.parent.insert(n, NodeParent::Real(bag.id));
                }
            });

        Some((bag, *self.relations.parent.get(&bag.id).unwrap()))
    }
}

pub trait NiceTreeDecomposition {
    fn make_nice(&self) -> TreeDecomposition;
}

// TODO: Rework with the 3 cases.
impl NiceTreeDecomposition for TreeDecomposition {
    fn make_nice(&self) -> TreeDecomposition {
        let mut result = self.clone();
        assert_eq!(self.bags.len(), result.bags.len());

        for bag in self.bags.iter() {
            let mut pred_id = bag.id;
            let vertices = bag.vertex_set.iter().copied().collect::<Vec<usize>>();

            for i in (0..bag.vertex_set.len()).rev() {
                let mut vs: FxHashSet<usize> = FxHashSet::default();
                for v2 in vertices[0..i].iter() {
                    vs.insert(*v2);
                }
                let new_id = result.add_bag(vs);
                result.add_edge(pred_id, new_id);
                pred_id = new_id;
            }
        }

        result
    }
}

use std::fs::File;
use std::io::{Error, Write};
fn td_write_to_dot(file: &mut File, td: &TreeDecomposition) -> Result<(), Error> {
    writeln!(file, "graph td {{")?;

    let iter = BfsIter::new(&td);
    for (bag, parent) in iter {
        writeln!(
            file,
            "\tB{} [label=\"{{{}}}\"];",
            bag.id,
            bag.vertex_set
                .iter()
                .map(|&a| a.to_string())
                .collect::<Vec<String>>()
                .join(", ")
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

    use crate::read_graph_file_into_dcel;
    use std::io::Error;
        let dcel = read_graph_file_into_dcel("data/exp.graph").unwrap();
        let dcel = read_graph_file_into_dcel("data/exp.graph").unwrap();
    }

    #[test]
    pub fn test_dot() -> Result<(), Error> {
        let mut td = TreeDecomposition {
            bags: Vec::new(),
            root: None,
            max_bag_size: 2,
        };

        let ab = td.add_bag(FxHashSet::from_iter(vec![0, 1]));
        let cd = td.add_bag(FxHashSet::from_iter(vec![2, 3]));
        let ef = td.add_bag(FxHashSet::from_iter(vec![4, 5]));
        td.add_edge(ab, cd);
        td.add_edge(ab, ef);

        let path = "td.dot";
        let mut td_out = File::create(path)?;
        td_write_to_dot(&mut td_out, &td)?;

        Ok(())