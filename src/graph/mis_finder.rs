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

pub trait DynTable<'a, Set>
where
    Set: Default + Clone + Eq + std::fmt::Debug,
{
    fn get(&self, bag_id: usize, subset: &Set) -> (usize, MisSize);
    // TODO: @cleanup This function may only be needed for debugging purposes.
    fn get_by_index(&self, bag_id: usize, subset_index: usize) -> (&Set, MisSize);
    fn get_max_root_set_index(&self, root_id: usize) -> (usize, MisSize);
    fn put<'b: 'a>(&'a mut self, bag_id: usize, subset: Set, size: MisSize);
    fn add_to_mis(&self, bag_id: usize, subset_index: usize, mis: &mut HashSet<usize>);
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

// TODO: Maybe this reconstruction is not correct.
// The reconstructed set is smaller than the size in the table.
fn reconstruct_mis<Set>(
    table: &dyn DynTable<Set>,
    root_id: usize,
    constr_table: &ConstructionTable,
    node_relations: &NodeRelations,
) -> HashSet<usize>
where
    Set: Eq + std::fmt::Debug + Clone + Default,
{
    let result = HashSet::from_iter(Vec::new());

    // This function recursively traverses the table and finds the maximum independent set.
    fn rec<Set>(
        table: &dyn DynTable<Set>,
        bag_id: usize,
        set_index: usize,
        constr_table: &ConstructionTable,
        mut mis: HashSet<usize>, // We move the set every time.
        node_relations: &NodeRelations,
    ) -> HashSet<usize>
    where
        Set: Eq + std::fmt::Debug + Clone + Default,
    {
        table.add_to_mis(bag_id, set_index, &mut mis);

        let children = &node_relations.children[&bag_id];

        match children.len() {
            // Leaf node: We're finished.
            0 => mis,

            // Check the child.
            1 => rec(
                table,
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
                    table,
                    children[0],
                    constr_table[bag_id][set_index].0.unwrap(),
                    &constr_table,
                    left_mis,
                    &node_relations,
                );

                let right = rec(
                    table,
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
    let (root_set_index, size) = table.get_max_root_set_index(root_id);

    println!("MIS size according to table: {size}");

    let result = rec(
        table,
        root_id,
        root_set_index,
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
    adjaceny_matrix: &Vec<Vec<bool>>,
    ntd: &NiceTreeDecomposition,
) -> Result<(HashSet<usize>, usize), FindMisError> {
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
                } else if child.vertex_set == bag.vertex_set {
                    // Some weird edge case. Is this a problem from an earlier phase or can we handle
                    // it this way?
                    for subset in SubsetIter::new(&bag.vertex_set) {
                        let (child_set_index, size) = dyn_table.get(child.id, &subset);
                        entry.sets.push(DynTableValueItem::new(subset, size));
                        // dyn_table.put(bag.id, subset, size);
                        constr_table[bag.id].push((Some(child_set_index), None));
                    }
                } else {
                    panic!("Unreachable");
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

fn is_independent_fast(adjaceny_matrix: &Vec<Vec<bool>>, v: usize, set: &BitSet) -> bool {
    let result = set.iter().all(|u| {
        println!("Check {v} -- {u}");
        !adjaceny_matrix[u][v]
    });
    if v == 15 {
        println!("BitSet: {:?}", set);
        println!("{}", result);
    }
    result
}

// TODO: If possible, merge the two `find_mis` algorithms.
// TODO: Would it be a nice idea to log the steps of the algorithm (print to a string buffer)?

pub fn find_mis_fast(
    adjaceny_matrix: &Vec<Vec<bool>>,
    ntd: &NiceTreeDecomposition,
) -> Result<(HashSet<usize>, usize), FindMisError> {
    let mut table: FastDynTable = FastDynTable::new(usize::pow(2, adjaceny_matrix.len() as u32));

    // TODO: @speed Don't use dynamic vectors, instead compute the maximum size required with
    // `max_bag_size`? This should be at max something like the length of the spanning tree.
    // Idea: The i-th set was constructed by the j-th child set.
    let mut constr_table: ConstructionTable = vec![Vec::new(); ntd.td.bags().len()];

    for bag in PostOrderIter::new(&ntd.td) {
        let children = &ntd.relations.children[&bag.id];

        match children.len() {
            0 => {
                table.put(bag.id, BitSet::new(), MisSize::Valid(0));
                table.put(
                    bag.id,
                    BitSet::from_iter(bag.vertex_set.iter().copied()),
                    MisSize::Valid(1),
                );
            }

            1 => {
                let child = &ntd.td.bags[children[0]];


                if let Some(&v) = bag.vertex_set.difference(&child.vertex_set).nth(0) {
                    // Introduce node.
                    for subset in SubBitSetIter::new(&bag.vertex_set) {
                        let set_index = constr_table[bag.id].len();
                        if !subset.contains(v) {
                            let (child_set_index, size) = table.get(child.id, &subset);
                            println!(
                                "{v} notin {subset:?} => M[{}, {subset:?}] = M[{}, {subset:?}] = {size}",
                                bag.id, child.id
                            );
                            table.put(bag.id, subset, size);
                            constr_table[bag.id].push((Some(child_set_index), None));
                        } else if is_independent_fast(&adjaceny_matrix, v, &subset) {
                            let mut clone = subset.clone();
                            clone.remove(v);
                            let (child_set_index, size) = table.get(child.id, &clone);
                            println!(
                                "{v} in {subset:?} => M[{}, {subset:?}] = M[{}, {clone:?}] + 1 = {size} + 1",
                                bag.id, child.id
                            );
                            table.put(bag.id, subset, size + MisSize::Valid(1));
                            constr_table[bag.id].push((Some(child_set_index), None));
                        } else {
                            println!(
                                "{subset:?} is not independent => M[{}, S] = -infinity",
                                bag.id
                            );
                            table.put(bag.id, subset, MisSize::Invalid);
                            constr_table[bag.id].push((None, None));
                        }
                        // println!("CT[{}, {}] = {:?}", bag.id, constr_table[bag.id].len(), constr_table[bag.id][set_index]);
                    }
                } else if let Some(&v) = child.vertex_set.difference(&bag.vertex_set).nth(0) {
                    // Forget node.
                    for subset in SubBitSetIter::new(&bag.vertex_set) {
                        let set_index = constr_table[bag.id].len();
                        let mut clone = subset.clone();
                        clone.insert(v);

                        let with = table.get(child.id, &clone);
                        let without = table.get(child.id, &subset);

                        let (child_set_index, size) =
                            std::cmp::max_by(with, without, |w, wo| w.1.cmp(&wo.1));

                        table.put(bag.id, subset, size);
                        constr_table[bag.id].push((Some(child_set_index), None));
                    }
                } else if child.vertex_set == bag.vertex_set {
                    // Some weird edge case. Is this a problem from an earlier phase or can we handle
                    // it this way?
                    for subset in SubBitSetIter::new(&bag.vertex_set) {
                        let (child_set_index, size) = table.get(child.id, &subset);
                        table.put(bag.id, subset, size);
                        constr_table[bag.id].push((Some(child_set_index), None));
                    }
                } else {
                    panic!("Unreachable");
                }
            }

            2 => {
                // Join node.
                let left_child = &ntd.td.bags[children[0]];
                let right_child = &ntd.td.bags[children[1]];

                for subset in SubBitSetIter::new(&bag.vertex_set) {
                    let set_index = constr_table[bag.id].len();
                    let (i, left_size) = table.get(left_child.id, &subset);
                    let (j, right_size) = table.get(right_child.id, &subset);
                    let len = MisSize::Valid(subset.len());
                    println!("M[{}, {subset:?}] = M[{}, S] + M[{}, S] - |S| = {left_size} + {right_size} - {len}", bag.id, left_child.id, right_child.id);
                    table.put(bag.id, subset, left_size + right_size - len);
                    constr_table[bag.id].push((Some(i), Some(j))); // Reconstruction.
                }
            }

            _ => panic!("Unreachable"),
        }
    }

    println!("Construction table:");
    // print_constr_table(&constr_table, &table, &ntd.relations);

    let result = reconstruct_mis(&table, ntd.td.root.unwrap(), &constr_table, &ntd.relations);

    Ok((result.clone(), result.len()))
}

/*
pub struct AlgorithmData<Set>
where
    Set: Eq + std::fmt::Debug + Clone + Default,
{
    new_set: dyn Fn(dyn Iterator<Item = usize> + 'static) -> Set,
}

pub fn find_mis_merged<Set>(
    adjaceny_matrix: Vec<Vec<bool>>,
    ntd: &NiceTreeDecomposition,
    table: &mut dyn DynTable<Set>,
    algo_data: &AlgorithmData<Set>,
) -> Result<(HashSet<usize>, usize), FindMisError>
where
    Set: Eq + std::fmt::Debug + Clone + Default,
{
    for bag in PostOrderIter::new(&ntd.td) {
        let children = &ntd.relations.children[&bag.id];

        match children.len() {
            0 => {
                table.put(bag.id, algo_data.new_set.call(vec![].iter()), MisSize::Valid(0));
            }

            1 => {}

            2 => {}

            _ => panic!("Unreachable"),
        }
    }
    todo!()
}
*/

fn print_constr_table<Set>(
    constr_table: &ConstructionTable,
    table: &dyn DynTable<Set>,
    node_relations: &NodeRelations,
) where
    Set: Eq + std::fmt::Debug + Clone + Default,
{
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
                        // println!("Bag ID: {bag_id}, set ID: {set_id}, child ID: {child_id}, child set ID: {p}");
                        let (subset, size) = table.get_by_index(bag_id, set_id);
                        let (child_subset, child_size) = table.get_by_index(child_id, *p);
                        println!(
                            "{bag_id}'s set {set_id} ({:?}, {}) from child {child_id}'s set {p} ({:?}, {})",
                            subset, size,
                            child_subset, child_size,
                        )
                    },

                    (Some(l), Some(r)) => {
                        let left_id = node_relations.children[&bag_id][0];
                        let right_id = node_relations.children[&bag_id][1];
                        println!(
                            "{bag_id}'s set {set_id} {:?} from left child {left_id}'s set {l} {:?} and right child {right_id}'s set {r} {:?}",
                            table.get_by_index(bag_id, set_id).0,
                            table.get_by_index(left_id, *l).0,
                            table.get_by_index(right_id, *r).0,
                        )
                    },

                    _ => panic!("Unreachable"),
                })
        });
}

pub fn find_connected_vertices(set: &HashSet<usize>, adjaceny_matrix: &Vec<Vec<bool>>) -> Vec<usize> {
    let mut result = Vec::new();
    for &u in set.iter() {
        for &v in set.iter() {
            if adjaceny_matrix[u][v] {
                result.push(u);
                result.push(v);
            }
        }
    }
    result
}

#[cfg(test)]
pub mod tests {
    use crate::{
        graph::{
            approximated_td::{ApproximatedTD, TDBuilder},
            dcel::spanning_tree::SpanningTree,
            // mis_finder::{find_mis, find_mis_fast},
            mis_finder::{find_mis, find_mis_fast, find_connected_vertices},
            nice_tree_decomp::NiceTreeDecomposition,
            node_relations::NodeRelations,
            tree_decomposition::td_write_to_dot,
        },
        read_graph_file_into_dcel_builder,
    };
    use arboretum_td::tree_decomposition::TreeDecomposition;
    use fxhash::FxHashSet;
    use std::{fs::File, process::Command};

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

        let (bag_content, size) = find_mis(&vec![vec![false; 6]; 6], &ntd).unwrap();
        println!("{:?} with size = {}", bag_content, size);
    }

    #[test]
    fn real() {
        let mut dcel_b = read_graph_file_into_dcel_builder("data/problem.graph").unwrap();
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

        assert!(ntd.validate(&td, &ntd_rels));

        let (bag_content, size) = find_mis(&adjacency_matrix, &ntd).unwrap();
        println!("MIS: {:?} with size = {}", bag_content, size);
        let connected_vertices = find_connected_vertices(&bag_content, &adjacency_matrix);
        assert!(connected_vertices.len() == 0, "Set is not independet: {:?}", connected_vertices);
    }

    #[test]
    fn fast() {
        let mut dcel_b = read_graph_file_into_dcel_builder("data/problem.graph").unwrap();
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

        assert!(ntd.validate(&td, &ntd_rels));

        let (bag_content, size) = find_mis_fast(&adjacency_matrix, &ntd).unwrap();
        println!("MIS: {:?} with size = {}", bag_content, size);
        let connected_vertices = find_connected_vertices(&bag_content, &adjacency_matrix);
        assert!(connected_vertices.len() == 0, "Set is not independet: {:?}", connected_vertices);
    }
}
