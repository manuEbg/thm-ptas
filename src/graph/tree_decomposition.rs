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
pub struct NodeRelations {
    parent: HashMap<usize, NodeParent>,
    children: HashMap<usize, Vec<usize>>,
}

impl NodeRelations {
    pub fn new(td: &TreeDecomposition) -> Self {
        let mut parent = HashMap::new();
        let mut children = HashMap::new();

        let mut queue = VecDeque::from([td.root.unwrap()]);
        let mut visited = vec![false; td.bags.len()];

        parent.insert(td.root.unwrap(), NodeParent::Fake);

        while let Some(bag_id) = queue.pop_front() {
            visited[bag_id] = true;
            children.insert(bag_id, Vec::new());
            let bag = &td.bags[bag_id];

            bag.neighbors
                .iter()
                .filter(|&&n| !visited[n])
                .for_each(|&n| {
                    queue.push_back(n);
                    if parent.get(&n).is_none() {
                        parent.insert(n, NodeParent::Real(bag_id));
                        children.get_mut(&bag_id).unwrap().push(n);
                    }
                });
        }

        Self { parent, children }
    }
}

struct BfsIter<'a> {
    td: &'a TreeDecomposition,
    queue: VecDeque<usize>, // Bag IDs.
    visited: Vec<bool>,     // @speed Use a bitset.
}

impl<'a> BfsIter<'a> {
    pub fn new(td: &'a TreeDecomposition) -> Self {
        BfsIter {
            td,
            queue: VecDeque::from([td.root.unwrap()]),
            visited: vec![false; td.bags.len()],
        }
    }
}

// TODO: Remove initialization of the node relations from the iterator.
impl<'a> Iterator for BfsIter<'a> {
    type Item = &'a Bag;

    fn next(&mut self) -> Option<Self::Item> {
        let front = self.queue.pop_front();
        if front.is_none() {
            return None;
        }

        let v = front.unwrap();
        self.visited[v] = true;
        let bag = &self.td.bags[v];

        // Find all unvisited neighbors.
        bag.neighbors
            .iter()
            .filter(|&&n| !self.visited[n])
            .for_each(|&n| self.queue.push_back(n));

        Some(bag)
    }
}

pub trait NiceTreeDecomposition {
    fn make_nice(&self, node_relations: &NodeRelations) -> TreeDecomposition;
}

/// This function returns the intersection between two bags and two vectors.
/// The first vector is the set difference between the first set and the intersection. Similarly, the
/// second vector is the set differences between the second set and the intersection.
/// They are used to easier create the between bags in [insert_between_bags] function.
fn get_bag_intersection(
    s1: &FxHashSet<usize>,
    s2: &FxHashSet<usize>,
) -> (FxHashSet<usize>, Vec<usize>, Vec<usize>) {
    let intersection = FxHashSet::from_iter(s1.intersection(&s2).copied());
    let b1_diff = s1.difference(&intersection).copied().collect();
    let b2_diff = s2.difference(&intersection).copied().collect();
    (intersection, b1_diff, b2_diff)
}

/// This function inserts between bags into the nice tree decomposition.
/// The between bags are connected to the given 'new parent bag' and if the old bag had some child,
/// the relation will be updated.
/// The sets for the between bags are calculated by [get_bag_intersection].
fn insert_between_bags(
    ntd: &mut TreeDecomposition,
    bag_relations: &mut BagRelations,
    s1: &FxHashSet<usize>,
    s2: &FxHashSet<usize>,
    new_parent_id: usize,
    old_child_id: Option<usize>,
) {
    let mut sets = Vec::new();
    let (intersection, b1_diff, b2_diff) = get_bag_intersection(s1, s2);

    // Build introduces.
    for last_idx in (intersection.len()..b1_diff.len()).rev() {
        // TODO: Would into_iter break the original vector?
        let diff_part = FxHashSet::from_iter(b1_diff[0..last_idx].iter().copied());
        let set = FxHashSet::from_iter(intersection.union(&diff_part).copied());
        sets.push(set);
    }

    // Build forgets.
    // We skip the first because this would just be the same as the last in the loop above.
    for last_idx in intersection.len() + 1..b2_diff.len() {
        // TODO: Would into_iter break the original vector?
        let diff_part = FxHashSet::from_iter(b2_diff[0..last_idx].iter().copied());
        let set = FxHashSet::from_iter(intersection.union(&diff_part).copied());
        sets.push(set);
    }

    let mut last_parent = new_parent_id;

    for set in sets.into_iter() {
        let between_bag = ntd.add_bag(set);
        ntd.add_edge(last_parent, between_bag);
        last_parent = between_bag;
    }

    let child_id = ntd.add_bag(s2.clone());
    ntd.add_edge(last_parent, child_id);
    if let Some(id) = old_child_id {
        bag_relations.to_old.insert(child_id, id);
        bag_relations.to_new.insert(id, child_id);
    }
}

struct BagRelations {
    to_new: HashMap<usize, usize>,
    to_old: HashMap<usize, usize>,
}

impl BagRelations {
    fn new(bags: &Vec<Bag>) -> Self {
        let (to_new, to_old) = bags.iter().fold(
            (HashMap::new(), HashMap::new()),
            |(mut to_new, mut to_old), bag| {
                to_new.insert(bag.id, bag.id);
                to_old.insert(bag.id, bag.id);
                (to_new, to_old)
            },
        );
        Self { to_new, to_old }
    }
}

impl NiceTreeDecomposition for TreeDecomposition {
    fn make_nice(&self, node_relations: &NodeRelations) -> TreeDecomposition {
        let mut ntd = TreeDecomposition {
            bags: Vec::new(),
            root: self.root.clone(),
            max_bag_size: self.max_bag_size,
        };

        let mut bag_relations = BagRelations::new(&self.bags);
        ntd.add_bag(FxHashSet::from_iter(
            self.bags[self.root.unwrap()].vertex_set.iter().copied(),
        ));

        for old_bag in BfsIter::new(&self) {
            let bag_id = bag_relations.to_new[&old_bag.id];

            let children = &node_relations.children[&old_bag.id];

            if children.len() >= 2 {
                // 1. Joins
                // For each bag that is not a leaf do
                // if |children| >= 2 and the bags have different vertex sets then
                //   Clone the bag |children| - 1 times.
                //   Set new edges and parents for each cloned self and child.

                // We take all children but the last, iterate from left to right and insert join nodes.
                let last_clone = children
                    .iter()
                    .take(children.len() - 1)
                    .map(|&child_id| &self.bags[child_id])
                    .fold(bag_id, |parent_id, old_child| {
                        let left_clone_id = ntd.add_bag(old_bag.vertex_set.clone());
                        let right_clone_id = ntd.add_bag(old_bag.vertex_set.clone());
                        ntd.add_edge(parent_id, left_clone_id);
                        ntd.add_edge(parent_id, right_clone_id);

                        // Inserts between bags between the clone and the current child.
                        insert_between_bags(
                            &mut ntd,
                            &mut bag_relations,
                            &old_bag.vertex_set,
                            &old_child.vertex_set,
                            left_clone_id,
                            Some(old_child.id),
                        );

                        right_clone_id
                    });

                let last_child = &self.bags[*children.last().unwrap()];

                // Inserts between bags between the last clone and the most right child.
                insert_between_bags(
                    &mut ntd,
                    &mut bag_relations,
                    &old_bag.vertex_set,
                    &last_child.vertex_set,
                    last_clone,
                    Some(last_child.id),
                );
            } else if children.len() == 1 {
                // 2. Introduces and forgets
                // For each bag that is not a leaf do
                //   Calculate the intersection with the parent bag.
                //   Introduce and forget until the intersection is met.
                let child = &self.bags[children[0]];
                let inserted_id = bag_relations.to_new[&bag_relations.to_old[&bag_id]];

                insert_between_bags(
                    &mut ntd,
                    &mut bag_relations,
                    &old_bag.vertex_set,
                    &child.vertex_set,
                    inserted_id,
                    Some(child.id),
                );
            } else {
                // 3. Introduces
                // For each leaf:
                //   Introduce bags from a set of one element to the leaf set.
                let end_set = FxHashSet::from_iter(old_bag.vertex_set.iter().take(1).copied());
                let inserted_id = bag_relations.to_new[&bag_relations.to_old[&bag_id]];

                insert_between_bags(
                    &mut ntd,
                    &mut bag_relations,
                    &old_bag.vertex_set,
                    &end_set,
                    inserted_id,
                    None,
                );
            }
        }

        ntd
    }
}

fn validate_nice_td(ntd: &TreeDecomposition, relations: &NodeRelations) -> bool {
    ntd.bags.iter().all(|bag| {
        let children = &relations.children[&bag.id];
        match children.len() {
            0 => bag.vertex_set.len() == 1,
            1 => {
                let child = &ntd.bags[children[0]];
                let parent_to_child_intersection = bag.vertex_set.difference(&child.vertex_set);
                let child_to_parent_intersection = child.vertex_set.difference(&bag.vertex_set);
                parent_to_child_intersection.count() == 1
                    || child_to_parent_intersection.count() == 1
            }
            2 => {
                let left_child = &ntd.bags[children[0]];
                let right_child = &ntd.bags[children[1]];
                bag.vertex_set.difference(&left_child.vertex_set).count() == 0
                    && bag.vertex_set.difference(&right_child.vertex_set).count() == 0
            }
            _ => false,
        }
    })
}

fn vertex_set_to_string<'a, T>(vs: T) -> String
where
    T: Iterator<Item = &'a usize>,
{
    vs.map(|&a| a.to_string())
        .collect::<Vec<String>>()
        .join(", ")
}

use std::fs::File;
use std::io::{Error, Write};
fn td_write_to_dot(
    title: &str,
    file: &mut File,
    td: &TreeDecomposition,
    node_relations: &NodeRelations,
) -> Result<(), Error> {
    writeln!(file, "graph {title} {{")?;

    let iter = BfsIter::new(&td);
    for bag in iter {
        let parent = node_relations.parent.get(&bag.id).unwrap();
        // let children = node_relations.children.get(&bag.id).unwrap();

        writeln!(
            file,
            "\tB{} [label=\"B{} {{{}}}\"];",
            bag.id,
            bag.id,
            vertex_set_to_string(bag.vertex_set.iter())
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
    use std::process::Command;
        let dcel = read_graph_file_into_dcel("data/exp.graph").unwrap();
        let dcel = read_graph_file_into_dcel("data/exp.graph").unwrap();
        /*
        */
    }

    #[test]
    pub fn test_dot() -> Result<(), Error> {
        let mut td = TreeDecomposition {
            bags: Vec::new(),
            root: None,
            max_bag_size: 2,
        };

        let b0 = td.add_bag(FxHashSet::from_iter(vec![0, 1]));
        let b1 = td.add_bag(FxHashSet::from_iter(vec![2, 3]));
        let b2 = td.add_bag(FxHashSet::from_iter(vec![4, 5]));
        let b3 = td.add_bag(FxHashSet::from_iter(vec![6, 7]));
        let b4 = td.add_bag(FxHashSet::from_iter(vec![8, 9]));
        let b5 = td.add_bag(FxHashSet::from_iter(vec![10, 11]));
        let b6 = td.add_bag(FxHashSet::from_iter(vec![12, 13]));
        td.add_edge(b0, b1);
        td.add_edge(b0, b2);
        td.add_edge(b0, b3);
        td.add_edge(b1, b4);
        td.add_edge(b1, b5);
        td.add_edge(b5, b6);

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
        assert!(validate_nice_td(&nice_td, &ntd_rels));

        let ntd_path = "ntd.dot";
        let mut ntd_out = File::create(ntd_path)?;
        td_write_to_dot("ntd", &mut ntd_out, &nice_td, &ntd_rels)?;
        Command::new("dot")
            .args(["-Tpdf", ntd_path, "-o", "ntd.pdf"])
            .spawn()
            .expect("dot command did not work.");

        Ok(())