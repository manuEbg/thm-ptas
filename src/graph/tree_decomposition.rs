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

/*
fn filter_split<'a, T, F>(v: Vec<T>, pred: F) -> (Vec<&'a T>, Vec<&'a T>)
where
    F: Fn(&'a T) -> bool,
{
    let mut wanted = Vec::new();
    let mut unwanted = Vec::new();

    for item in v.iter() {
        if pred(item) {
            wanted.push(item);
        } else {
            unwanted.push(item);
        }
    }

    (wanted, unwanted)
}
*/

fn get_bag_intersection(
    s1: &FxHashSet<usize>,
    s2: &FxHashSet<usize>,
) -> (FxHashSet<usize>, Vec<usize>, Vec<usize>) {
    let intersection = FxHashSet::from_iter(s1.intersection(&s2).copied());
    let b1_diff = s1.difference(&intersection).copied().collect();
    let b2_diff = s2.difference(&intersection).copied().collect();
    (intersection, b1_diff, b2_diff)
}

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
        let copy = set.clone();
        let between_bag = ntd.add_bag(set);
        println!("Between bag {between_bag}: {{{}}}", vertex_set_to_string(copy.iter()));
        println!("{last_parent} -> {between_bag}");
        ntd.add_edge(last_parent, between_bag);
        println!("Between bag neighbors: {{{}}}", vertex_set_to_string(ntd.bags[between_bag].neighbors.iter()));
        last_parent = between_bag;
    }

    let child_id = ntd.add_bag(s2.clone());
    ntd.add_edge(last_parent, child_id);
    if let Some(id) = old_child_id {
        bag_relations.to_old.insert(child_id, id);
        bag_relations.to_new.insert(id, child_id);
    }

    println!(
        "Parent: ({}) {:?} -> Child: ({child_id}) {:?}",
        new_parent_id, ntd.bags[new_parent_id].vertex_set, ntd.bags[child_id].vertex_set
    );
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

// TODO: Maybe do a dfs instead of a bfs?
impl NiceTreeDecomposition for TreeDecomposition {
    fn make_nice(&self, node_relations: &NodeRelations) -> TreeDecomposition {
        let mut ntd = TreeDecomposition {
            bags: Vec::new(),
            root: self.root.clone(),
            max_bag_size: self.max_bag_size,
        };

        // let (inner_nodes, leafs) = filter_split();

        // 1. Joins
        // For each bag that is not a leaf do
        // if |children| >= 2 and the bags have different vertex sets then
        //   Clone the bag |children| - 1 times.
        //   Set new edges and parents for each cloned self and child.

        // TODO: We need to remember the updated parents.
        // Now we look up the wrong values when e.g. a join inserts new bags.
        // Or we create a mapping from old bag ids to new bag ids. (THIS is better)

        /*
        let mut node_mappings = self.bags.iter().fold(HashMap::new(), |mut ms, bag| {
            ms.insert(bag.id, bag.id);
            ms
        });
        */

        let mut bag_relations = BagRelations::new(&self.bags);

        ntd.add_bag(FxHashSet::from_iter(self.bags[self.root.unwrap()].vertex_set.iter().copied()));

        for old_bag in BfsIter::new(&self) {
            // TODO: Do not insert always because the bag might have been a child of another and is
            // already inserted?
            // let bag_id = ntd.add_bag(old_bag.vertex_set.clone());
            let bag_id = bag_relations.to_new[&old_bag.id];

            /*
            let mapped_bag = match bag_relations.to_old.get(&bag_id) {
                Some(id) => *id,
                None => {
                    bag_relations.to_old.insert(bag_id, old_bag.id);
                    old_bag.id
                }
            };
            */

            println!("New {} -> Old {}", bag_id, old_bag.id);

            let children = &node_relations.children[&old_bag.id];

            println!(
                "|children| = ({}) {{{}}}",
                children.len(),
                vertex_set_to_string(children.iter())
            );

            if children.len() >= 2 {
                // Join node.
                println!("Join node: {:?}", old_bag.vertex_set);

                // Clone the bag |children| - 1 times.
                // Set new edges and parents for each cloned self and child.
                let last_clone = children
                    .iter()
                    .take(children.len() - 1)
                    .map(|&child_id| &self.bags[child_id])
                    .fold(bag_id, |parent_id, old_child| {
                        let left_clone_id = ntd.add_bag(old_bag.vertex_set.clone());
                        let right_clone_id = ntd.add_bag(old_bag.vertex_set.clone());
                        ntd.add_edge(parent_id, left_clone_id);
                        ntd.add_edge(parent_id, right_clone_id);

                        println!("Left clone : ({left_clone_id}) {{{}}}", vertex_set_to_string(ntd.bags[left_clone_id].vertex_set.iter()));
                        println!("Child      : ({}) {{{}}}", old_child.id, vertex_set_to_string(old_child.vertex_set.iter()));

                        insert_between_bags(&mut ntd, &mut bag_relations, &old_bag.vertex_set, &old_child.vertex_set, left_clone_id, Some(old_child.id));

                        println!("Parent: ({parent_id}) {:?} -> ({left_clone_id}) {:?}, ({right_clone_id}) {:?}",
                            ntd.bags[parent_id].vertex_set,
                            ntd.bags[left_clone_id].vertex_set,
                            ntd.bags[right_clone_id].vertex_set);

                        right_clone_id
                    });

                let last_child = &self.bags[*children.last().unwrap()];
                println!("Add last child {}: {{{}}}", last_child.id, vertex_set_to_string(last_child.vertex_set.iter()));
                insert_between_bags(
                    &mut ntd,
                    &mut bag_relations,
                    &old_bag.vertex_set,
                    &last_child.vertex_set,
                    last_clone,
                    Some(last_child.id),
                );
            } else if children.len() == 1 {
                println!("Inner node: {{{}}}", vertex_set_to_string(old_bag.vertex_set.iter()));
                let child = &self.bags[children[0]];
                println!("Child: {{{}}}", vertex_set_to_string(child.vertex_set.iter()));
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
                println!("Leaf node: {:?}", old_bag.vertex_set);
                let end_set = FxHashSet::from_iter(old_bag.vertex_set.iter().take(1).copied());
                let inserted_id = bag_relations.to_new[&bag_relations.to_old[&bag_id]];

                insert_between_bags(
                    &mut ntd,
                    &mut bag_relations,
                    &old_bag.vertex_set,
                    &end_set,
                    inserted_id,
                    None
                );
            }
            println!("-----");
        }

        /*
        println!("Second iteration for inner and leaf nodes.");
        let tmp_ntd_node_rels = NodeRelations::new(&ntd);
        for bag in BfsIter::new(&ntd) {
            let children = tmp_ntd_node_rels.children.get(&bag.id).unwrap();
            if children.len() < 2 {
                // Inner node.
                println!("Inner or leaf node: ({}) {:?}", bag.id, bag.vertex_set);

                if let NodeParent::Real(parent_id) = tmp_ntd_node_rels.parent.get(&bag.id).unwrap()
                {
                    let parent = &ntd.bags[*parent_id];
                    println!("\tParent: ({parent_id}) {:?}", parent.vertex_set);
                    // Calculate the intersection with the parent bag.
                    let intersection = get_bag_intersection(bag, parent);
                    println!(
                        "\tB({parent_id}) ∩ B({}) = {{{}}} ∩ {{{}}} = {{{}}}",
                        bag.id,
                        vertex_set_to_string(parent.vertex_set.iter()),
                        vertex_set_to_string(bag.vertex_set.iter()),
                        vertex_set_to_string(intersection.iter()),
                    );
                    for i in (intersection.len()..parent.vertex_set.len()) {}

                    // Introduce and forget until the intersection is met.
                }
            }

            if children.len() == 0 {
                // Leaf node.
                println!("Leaf node: ({}) {:?}", bag.id, bag.vertex_set);

                // Introduce bags from a set of one element to the leaf set.
            }
        }
        */

        /*
        for bag in BfsIter::new(&self) {
            let parent = node_relations.parent.get(&bag.id).unwrap();
            let children = node_relations.children.get(&bag.id).unwrap();

            if children.len() < 2 {
                continue;
            }

            let mut diff = false;

            // TODO: Think about if we need to clone for all children when we finde one difference
            // or if we just need to clone for the children that actually have a different vertex
            // set.
            for &child_id in children.iter() {
                let child = &self.bags[child_id];
                let intersection = child
                    .vertex_set
                    .intersection(&bag.vertex_set)
                    .map(|&v| v)
                    .collect::<Vec<usize>>();

                if intersection.len() < bag.vertex_set.len() {
                    diff = true;
                    break;
                }
            }

            // When we do not find any difference in the child sets
            if !diff {
                continue;
            }

            if children.is_empty() {
                continue;
            }

            let clone_count = children.len() - 1;
            for i in 0..clone_count {}
        }

        // 2. Introduces and forgets
        // For each bag that is not a leaf do
        //   Calculate the intersection with the parent bag.
        //   Introduce and forget until the intersection is met.

        for bag in BfsIter::new(&self) {
            match node_relations.parent.get(&bag.id).unwrap() {
                NodeParent::Fake => {} // Nothing to do for the fake root parent.
                NodeParent::Real(parent_id) => {
                    let parent = &self.bags[*parent_id];
                    let intersection = get_bag_intersection(bag, parent);
                    let bag_size = bag.vertex_set.len();
                    let parent_size = parent.vertex_set.len();
                    let forget_count = bag_size - intersection.len();
                    let introduce_count = parent_size - intersection.len();

                    // Remove the neighbors mutually in the result tree decomposition.
                    // Since there is no `remove_edge` we have to do it on the set itself.
                    ntd
                        .bags
                        .get_mut(bag.id)
                        .unwrap()
                        .neighbors
                        .remove(&parent.id);
                    ntd
                        .bags
                        .get_mut(*parent_id)
                        .unwrap()
                        .neighbors
                        .remove(&bag.id);

                    let mut last_bag_id = bag.id;
                    for i in 1..forget_count + 1 {
                        let new_vertex_set = FxHashSet::from_iter(
                            self.bags[last_bag_id]
                                .vertex_set
                                .iter()
                                .take(bag_size - i) // Drop the last i values.
                                .copied(),
                        );
                        let new_bag_id = ntd.add_bag(new_vertex_set);
                        ntd.add_edge(last_bag_id, new_bag_id);
                        last_bag_id = new_bag_id;
                    }

                    for i in 1..introduce_count {}
                }
            }
        }

        // 3. Introduces
        // For each leaf:
        //   Introduce bags from a set of one element to the leaf set.

        for bag in self.bags.iter() {
            let mut pred_id = bag.id;
            let vertices = bag.vertex_set.iter().copied().collect::<Vec<usize>>();

            for i in (1..bag.vertex_set.len()).rev() {
                let mut vs: FxHashSet<usize> = FxHashSet::default();
                for v2 in vertices[0..i].iter() {
                    vs.insert(*v2);
                }
                let new_id = ntd.add_bag(vs);
                ntd.add_edge(pred_id, new_id);
                pred_id = new_id;
            }
        }
        */

        ntd
    }
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

        let ntd_path = "ntd.dot";
        let mut ntd_out = File::create(ntd_path)?;
        td_write_to_dot("ntd", &mut ntd_out, &nice_td, &ntd_rels)?;
        Command::new("dot")
            .args(["-Tpdf", ntd_path, "-o", "ntd.pdf"])
            .spawn()
            .expect("dot command did not work.");

        Ok(())