use crate::graph::iterators::bfs::TreeDecompBfsIter;
use arboretum_td::tree_decomposition::{Bag, TreeDecomposition};
use fxhash::FxHashSet;
use std::collections::HashMap;

use super::node_relations::NodeRelations;

pub struct NiceTreeDecomposition {
    pub td: TreeDecomposition,
    pub relations: NodeRelations,
}

impl NiceTreeDecomposition {
    pub fn validate(&self, otd: &TreeDecomposition, relations: &NodeRelations) -> bool {
        let mut present = vec![false; otd.bags.len()]; // Check if all original bags are present.
        self.td.bags.iter().all(|bag| {
            if let Some(b) = otd
                .bags
                .iter()
                .filter(|obag| obag.vertex_set.eq(&bag.vertex_set))
                .nth(0)
            {
                present[b.id] = true;
            }
            let children = &relations.children[&bag.id];
            match children.len() {
                0 => bag.vertex_set.len() == 1,
                1 => {
                    let child = &self.td.bags[children[0]];
                    let parent_to_child_intersection = bag.vertex_set.difference(&child.vertex_set);
                    let child_to_parent_intersection = child.vertex_set.difference(&bag.vertex_set);
                    parent_to_child_intersection.count() == 1
                        || child_to_parent_intersection.count() == 1
                }
                2 => {
                    let left_child = &self.td.bags[children[0]];
                    let right_child = &self.td.bags[children[1]];
                    bag.vertex_set.difference(&left_child.vertex_set).count() == 0
                        && bag.vertex_set.difference(&right_child.vertex_set).count() == 0
                }
                _ => false,
            }
        }) && present.iter().all(|&v| v)
    }
}

impl From<&TreeDecomposition> for NiceTreeDecomposition {
    fn from(td: &TreeDecomposition) -> Self {
        let td_rels = NodeRelations::new(&td);

        let mut ntd = TreeDecomposition {
            bags: Vec::new(),
            root: td.root.clone(),
            max_bag_size: td.max_bag_size,
        };

        let mut bag_relations = BagRelations::new(&td.bags);
        ntd.add_bag(FxHashSet::from_iter(
            td.bags[td.root.unwrap()].vertex_set.iter().copied(),
        ));

        for old_bag in TreeDecompBfsIter::new(&td) {
            let bag_id = bag_relations.to_new[&old_bag.id];

            let children = &td_rels.children[&old_bag.id];

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
                    .map(|&child_id| &td.bags[child_id])
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

                let last_child = &td.bags[*children.last().unwrap()];

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
                let child = &td.bags[children[0]];
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

        let relations = NodeRelations::new(&ntd);

        NiceTreeDecomposition { td: ntd, relations }
    }
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

    println!("Insert between bags for new {new_parent_id} and old {old_child_id:?}");
    println!("Intersection = {intersection:?}, introduces = {b1_diff:?}, forgets = {b2_diff:?}");

    // Build introduces.
    for last_idx in (0..b1_diff.len()).rev() {
        // TODO: Would into_iter break the original vector?
        let diff_part = FxHashSet::from_iter(b1_diff[0..last_idx].iter().copied());
        let set = FxHashSet::from_iter(intersection.union(&diff_part).copied());
        println!("(Introduce) Add {set:?}");
        sets.push(set);
    }

    let mut insert_last_child = false;
    // Build forgets.
    // We skip the first because this would just be the same as the last in the loop above.
    for last_idx in 1..b2_diff.len() {
        // TODO: Would into_iter break the original vector?
        let diff_part = FxHashSet::from_iter(b2_diff[0..last_idx].iter().copied());
        let set = FxHashSet::from_iter(intersection.union(&diff_part).copied());
        println!("(Forget) Add {set:?}");
        sets.push(set);
        insert_last_child = true;
    }

    let mut last_parent = new_parent_id;

    for set in sets.into_iter() {
        let between_bag = ntd.add_bag(set);
        ntd.add_edge(last_parent, between_bag);
        last_parent = between_bag;
    }

    let child_id = if insert_last_child {
        let child_id = ntd.add_bag(s2.clone());
        ntd.add_edge(last_parent, child_id);
        child_id
    } else {
        last_parent
    };

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

#[cfg(test)]
pub mod tests {
    use std::{fs::File, process::Command};

    use arboretum_td::tree_decomposition::TreeDecomposition;
    use fxhash::FxHashSet;

    use crate::graph::{
        nice_tree_decomp::NiceTreeDecomposition, node_relations::NodeRelations,
        tree_decomposition::td_write_to_dot,
    };

    #[test]
    pub fn test_nice_tree_decomposition() {
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
        let mut td_out = File::create(td_path).unwrap();
        td_write_to_dot("td", &mut td_out, &td, &td_rels).unwrap();
        Command::new("dot")
            .args(["-Tpdf", td_path, "-o", "td.pdf"])
            .spawn()
            .expect("dot command did not work.");

        let ntd = NiceTreeDecomposition::from(&td);
        let ntd_rels = NodeRelations::new(&ntd.td);
        assert!(ntd.validate(&td, &ntd_rels));

        let ntd_path = "ntd.dot";
        let mut ntd_out = File::create(ntd_path).unwrap();
        td_write_to_dot("ntd", &mut ntd_out, &ntd.td, &ntd_rels).unwrap();
        Command::new("dot")
            .args(["-Tpdf", ntd_path, "-o", "ntd.pdf"])
            .spawn()
            .expect("dot command did not work.");
    }
}
