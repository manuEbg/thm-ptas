use std::collections::{HashMap, HashSet};

use bit_set::BitSet;
use fxhash::FxHashSet;

use crate::graph::iterators::subset::SubsetIter;

use super::{
    dyn_table::{
        dt_fast::FastDynTable,
        dt_normal::{DynTableValue, DynTableValueItem, NormalDynTable},
    },
    iterators::{post_order::PostOrderIter, subset::SubBitSetIter},
    nice_tree_decomp::NiceTreeDecomposition,
    node_relations::NodeRelations,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum MisSize {
    Invalid,
    Valid(usize),
}

pub trait DynTable {
    fn get(&self, bag_id: usize, subset: &FxHashSet<usize>) -> (usize, MisSize);
    fn put(&mut self, bag_id: usize, subset: FxHashSet<usize>, size: MisSize);
}

impl std::ops::Add for MisSize {
    type Output = MisSize;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            MisSize::Invalid => MisSize::Invalid,
            MisSize::Valid(l) => match rhs {
                MisSize::Invalid => MisSize::Invalid,
                MisSize::Valid(r) => MisSize::Valid(l + r),
            },
        }
    }
}

impl std::ops::Sub for MisSize {
    type Output = MisSize;

    fn sub(self, rhs: Self) -> Self::Output {
        match self {
            MisSize::Invalid => MisSize::Invalid,
            MisSize::Valid(l) => match rhs {
                MisSize::Invalid => MisSize::Invalid,
                MisSize::Valid(r) => MisSize::Valid(l - r),
            },
        }
    }
}

impl std::fmt::Display for MisSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MisSize::Invalid => write!(f, "-infinity"),
            MisSize::Valid(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Debug)]
pub enum FindMisError {
    InvalidNiceTD,
    NoMisFound,
}

impl std::error::Error for FindMisError {}

impl std::fmt::Display for FindMisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FindMisError::InvalidNiceTD => write!(f, "Invalid nice tree decomposition!"),
            FindMisError::NoMisFound => write!(f, "Could not find an maximum independent set!"),
        }
    }
}

type ConstructionTable = Vec<Vec<(Option<usize>, Option<usize>)>>;

fn is_independent(adjaceny_matrix: &Vec<Vec<bool>>, v: usize, set: &FxHashSet<usize>) -> bool {
    set.iter().all(|u| !adjaceny_matrix[v][*u])
}

fn reconstruct_mis(
    table: &NormalDynTable,
    root_id: usize,
    constr_table: &ConstructionTable,
    node_relations: &NodeRelations,
) -> FxHashSet<usize> {
    let result = FxHashSet::from_iter(Vec::new());

    // This function recursively traverses the table and finds the maximum independent set.
    fn rec(
        table: &NormalDynTable,
        bag_id: usize,
        set_index: usize,
        constr_table: &ConstructionTable,
        mut mis: FxHashSet<usize>, // We move the set every time.
        node_relations: &NodeRelations,
    ) -> FxHashSet<usize> {
        let item = &table.0[&bag_id].sets[set_index];

        item.mis.iter().for_each(|&v| {
            mis.insert(v);
        });

        let children = &node_relations.children[&bag_id];

        match children.len() {
            // Leaf node: We're finished.
            0 => mis,

            // Check the child.
            1 => rec(
                &table,
                children[0],
                constr_table[bag_id][set_index].0.unwrap(),
                &constr_table,
                mis,
                &node_relations,
            ),

            // Find the maximum from the left and right child.
            2 => {
                let left_mis = mis.clone();
                let right_mis = mis.clone();

                let left = rec(
                    &table,
                    children[0],
                    constr_table[bag_id][set_index].0.unwrap(),
                    &constr_table,
                    left_mis,
                    &node_relations,
                );

                let right = rec(
                    &table,
                    children[1],
                    constr_table[bag_id][set_index].1.unwrap(),
                    &constr_table,
                    right_mis,
                    &node_relations,
                );

                std::cmp::max_by(left, right, |l, r| l.len().cmp(&r.len()))
            }

            _ => panic!("Unreachable"),
        }
    }

    // Find the largest set in the root node. This begins the table traversal.
    let (set_index, _) = &table.0[&root_id]
        .sets
        .iter()
        .enumerate()
        .max_by(|(_, l), (_, r)| l.size.cmp(&r.size))
        .unwrap();

    let result = rec(
        &table,
        root_id,
        *set_index,
        &constr_table,
        result,
        &node_relations,
    );

    result
}

// TODO:
// 1. Remove the `.0` for the `dyn_table` and use the common interface.
// 2. Add an adjaceny matrix/list to check whether two vertices are connected or not.
pub fn find_mis(
    adjaceny_matrix: Vec<Vec<bool>>,
    ntd: &NiceTreeDecomposition,
) -> Result<(FxHashSet<usize>, usize), FindMisError> {
    let mut dyn_table = NormalDynTable::default();

    // TODO: @speed Don't use dynamic vectors, instead compute the maximum size required with
    // `max_bag_size`? This should be at max something like the length of the spanning tree.
    // Idea: The i-th set was constructed by the j-th child set.
    let mut constr_table: ConstructionTable = vec![Vec::new(); ntd.td.bags().len()];

    for bag in PostOrderIter::new(&ntd.td) {
        let children = &ntd.relations.children[&bag.id];
        let mut entry = DynTableValue::default();

        match children.len() {
            0 => {
                entry.add(DynTableValueItem::new(
                    FxHashSet::from_iter(Vec::new().into_iter()),
                    MisSize::Valid(0),
                ));
                entry.add(DynTableValueItem::new(
                    FxHashSet::from_iter(bag.vertex_set.iter().copied()),
                    MisSize::Valid(1),
                ));
            }

            1 => {
                let child = &ntd.td.bags[children[0]];
                let child_entry = dyn_table.0.get(&child.id).unwrap();
                if let Some(&v) = bag.vertex_set.difference(&child.vertex_set).nth(0) {
                    // Introduce node.
                    println!(
                        "Introduce {v}: B{} -> B{} = {:?} -> {:?}",
                        &child.id, &bag.id, &child.vertex_set, &bag.vertex_set
                    );

                    for subset in SubsetIter::new(&bag.vertex_set) {
                        if !subset.contains(&v) {
                            let (i, value) = dyn_table.get(child.id, &subset);
                            entry.sets.push(DynTableValueItem::new(subset, value));
                            constr_table[bag.id].push((Some(i), None)); // Reconstruction.
                        } else if is_independent(&adjaceny_matrix, v, &subset) {
                            // @speed This clone could be expensive.
                            let mut clone = subset.clone();
                            clone.remove(&v);
                            println!("{subset:?} + 1");
                            let (i, value) = dyn_table.get(child.id, &clone);
                            entry
                                .sets
                                .push(DynTableValueItem::new(subset, value + MisSize::Valid(1)));
                            constr_table[bag.id].push((Some(i), None)); // Reconstruction.
                        } else {
                            println!("{subset:?} is not independent.");
                            entry
                                .sets
                                .push(DynTableValueItem::new(subset, MisSize::Invalid));
                            constr_table[bag.id].push((None, None)); // Reconstruction.
                        }
                    }
                } else if let Some(&v) = child.vertex_set.difference(&bag.vertex_set).nth(0) {
                    // Forget node.
                    // forall subsets of bag: M[bag, subset] = max { ... }.

                    println!(
                        "Forget: B{} -> B{} = {:?} -> {:?}",
                        &child.id, &bag.id, &child.vertex_set, &bag.vertex_set
                    );

                    for subset in SubsetIter::new(&bag.vertex_set) {
                        // @speed This clone could be expensive.
                        let mut clone = subset.clone();
                        clone.insert(v);

                        let without = dyn_table.get(child.id, &subset);
                        let with = dyn_table.get(child.id, &clone);

                        let (i, value) = std::cmp::max_by(with, without, |w, wo| w.1.cmp(&wo.1));

                        entry.sets.push(DynTableValueItem::new(subset, value));
                        constr_table[bag.id].push((Some(i), None)); // Reconstruction.
                    }
                }
            }

            2 => {
                // Join node.
                // forall subsets of bag: M[bag, subset] = M[lc, subset] + M[rc, subset] - |subset|

                let left_child = &ntd.td.bags[children[0]];
                let right_child = &ntd.td.bags[children[1]];

                for subset in SubsetIter::new(&bag.vertex_set) {
                    let (i, left_value) = dyn_table.get(left_child.id, &subset);
                    let (j, right_value) = dyn_table.get(right_child.id, &subset);
                    let len = MisSize::Valid(subset.len());

                    println!("{left_value} + {right_value} - {len}");
                    entry.sets.push(DynTableValueItem::new(
                        subset,
                        left_value + right_value - len,
                    ));
                    constr_table[bag.id].push((Some(i), Some(j))); // Reconstruction.
                }
            }

            _ => assert!(false, "Unreachable"),
        }

        dyn_table.0.insert(bag.id.clone(), entry);
    }

    println!("{}", &dyn_table);

    let result = reconstruct_mis(
        &dyn_table,
        ntd.td.root.unwrap(),
        &constr_table,
        &ntd.relations,
    );

    Ok((result.clone(), result.len()))
}

// TODO:
/*
pub fn find_mis_fast(
    adjaceny_matrix: Vec<Vec<bool>>,
    ntd: &NiceTreeDecomposition,
) -> Result<(FxHashSet<usize>, usize), FindMisError> {
    let mut table: FastDynTable = HashMap::new();

    // TODO: @speed Don't use dynamic vectors, instead compute the maximum size required with
    // `max_bag_size`? This should be at max something like the length of the spanning tree.
    // Idea: The i-th set was constructed by the j-th child set.
    let mut constr_table: ConstructionTable = vec![Vec::new(); ntd.bags().len()];

    for bag in PostOrderIter::new(&ntd) {
        let children = &ntd.relations.children[&bag.id];

        match children.len() {
            0 => {
                table.insert((bag.id, BitSet::new()), 0);
                table.insert(
                    (bag.id, BitSet::from_iter(bag.vertex_set.iter().copied())),
                    1,
                );
            }

            1 => {
                let child = &ntd.bags[children[0]];
                if let Some(&v) = bag.vertex_set.difference(&child.vertex_set).nth(0) {
                    // Introduce node.
                    for subset in SubBitSetIter::new(&bag.vertex_set) {
                        if !subset.contains(v) {
                            table.insert((bag.id, subset), table[(bag.id, subset)]);
                        } else if is_independent2(&adjaceny_matrix, v, &subset) {

                        } else {

                        }
                    }
                } else if let Some(&v) = child.vertex_set.difference(&bag.vertex_set).nth(0) {
                    // Forget node.
                }
            }

            2 => {}

            _ => panic!("Unreachable"),
        }
    }

    // TODO: Reconstruction.

    Ok((FxHashSet::from_iter(vec![]), 0))
}
*/

#[cfg(test)]
pub mod tests {
    use crate::{
        graph::{
            approximated_td::{ApproximatedTD, TDBuilder},
            dcel::spanning_tree::SpanningTree,
            dyn_table::dt_normal::NormalDynTable,
            iterators::bfs::TreeDecompBfsIter,
            mis_finder::find_mis,
            nice_tree_decomp::NiceTreeDecomposition,
            node_relations::{NodeParent, NodeRelations}, tree_decomposition::td_write_to_dot,
        },
        read_graph_file_into_dcel_builder,
    };
    use arboretum_td::tree_decomposition::TreeDecomposition;
    use fxhash::FxHashSet;
    use std::{fs::File, io::Error, io::Write, process::Command};

    use super::ConstructionTable;

    #[test]
    fn simple() {
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

        let (bag_content, size) = find_mis(vec![vec![false; 6]; 6], &ntd).unwrap();
        println!("{:?} with size = {}", bag_content, size);
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
        let ntd = NiceTreeDecomposition::from(&td);
        let ntd_rels = NodeRelations::new(&ntd.td);

        ntd.validate(&td, &ntd_rels);

        let td_path = "td.dot";
        let mut td_out = File::create(td_path).unwrap();
        td_write_to_dot("td", &mut td_out, &td, &td_rels).unwrap();
        Command::new("dot")
            .args(["-Tpdf", td_path, "-o", "td.pdf"])
            .spawn()
            .expect("dot command did not work.");

        let ntd_path = "ntd.dot";
        let mut ntd_out = File::create(ntd_path).unwrap();
        td_write_to_dot("ntd", &mut ntd_out, &ntd.td, &ntd_rels).unwrap();
        Command::new("dot")
            .args(["-Tpdf", ntd_path, "-o", "ntd.pdf"])
            .spawn()
            .expect("dot command did not work.");

        let (bag_content, size) = find_mis(adjacency_matrix, &ntd).unwrap();
        println!("MIS: {:?} with size = {}", bag_content, size);
    }

    fn print_constr_table(
        constr_table: &ConstructionTable,
        table: &NormalDynTable,
        node_relations: &NodeRelations,
    ) {
        constr_table
            .iter()
            .enumerate()
            .for_each(|(bag_id, preds)| {
                preds
                    .iter()
                    .enumerate()
                    .for_each(|(set_id, preds)| match preds {
                        (None, None) => {},

                        (Some(p), None) => {
                            let child_id = node_relations.children[&bag_id][0];
                            let item = &table.0[&bag_id].sets[set_id];
                            let child_item = &table.0[&child_id].sets[*p];
                            println!(
                                "{bag_id}'s set {set_id} ({:?}, {}) from child {child_id}'s set {p} ({:?}, {})",
                                item.mis, item.size,
                                child_item.mis, child_item.size,
                            )
                        },

                        (Some(l), Some(r)) => {
                            let left_id = node_relations.children[&bag_id][0];
                            let right_id = node_relations.children[&bag_id][1];
                            println!(
                                "{bag_id}'s set {set_id} {:?} from left child {left_id}'s set {l} {:?} and right child {right_id}'s set {r} {:?}",
                                table.0[&bag_id].sets[set_id].mis,
                                table.0[&left_id].sets[*l].mis,
                                table.0[&right_id].sets[*r].mis,
                            )
                        },

                        _ => panic!("Unreachable"),
                    })
            });
    }
}
