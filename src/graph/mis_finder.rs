use std::io::Write;
use std::{
    collections::HashSet,
    fs::File,
};

use bit_set::BitSet;
use fxhash::FxHashSet;

use crate::graph::{
    dyn_table::{dt_fast::NtdAndFastTable, dt_normal::NtdAndNormalTable},
    iterators::subset::SubsetIter,
};

use super::{
    dyn_table::{
        dt_fast::FastDynTable,
        dt_normal::{DynTableValue, DynTableValueItem, NormalDynTable},
    },
    iterators::{post_order::PostOrderIter, subset::SubBitSetIter},
    nice_tree_decomp::NiceTreeDecomposition,
    node_relations::NodeRelations,
};

/// Represents a maximum independent set size that can either be a positive integer or negative
/// infinity. In order to avoid arithmetic overflows, the addition and subtraction operators are
/// overloaded and negative infinity "consumes" valid values.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum MisSize {
    Invalid,
    Valid(usize),
}

/// Represents a dynamic table for the maximum independent set algorithm.
/// The functions for the algorithm are [find_mis] and [find_mis_fast].
pub trait DynTable<'a, Set>
where
    Set: Default + Clone + Eq + std::fmt::Debug,
{
    /// Returns the set index and the maximum independent set size for a given bag ID and subset.
    fn get(&self, bag_id: usize, subset: &Set) -> (usize, MisSize);

    // TODO: @cleanup This function may only be needed for debugging purposes.
    /// Returns the subset and its maximum independent set size for a given abg ID and the subset
    /// index.
    fn get_by_index(&self, bag_id: usize, subset_index: usize) -> (&Set, MisSize);

    /// Returns a list of set indices and corresponding maximum independent set sizes for the root
    /// bag. It is used to start the reconstruction of the maximum independent set, see
    /// [reconstruct_mis].
    fn get_max_root_set_indices(&self, root_id: usize) -> Vec<(usize, MisSize)>;

    /// Inserts a bag ID, a subset and the calculated maximum independent set size into the table.
    fn put<'b: 'a>(&'a mut self, bag_id: usize, subset: Set, size: MisSize);

    /// Adds the vertices from the subset described by the given bag ID and subset index to the
    /// maximum independent set [`mis`]. This function is used in the reconstruction algorithm.
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

/// Represents possible errors that can occur when the algorithm tries to find the maximum
/// independent set.
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

/// Represents a 2D matrix that stores information about the construction of dynamic table values.
type ConstructionTable = Vec<Vec<(Option<usize>, Option<usize>)>>;

/// Checks whether `v` is independent from all vertices in `set` or not.
fn is_independent(adjaceny_matrix: &Vec<Vec<bool>>, v: usize, set: &FxHashSet<usize>) -> bool {
    set.iter().all(|u| !adjaceny_matrix[v][*u])
}

/// Reconstructs the maximum independent set from the dynamic table.
/// It recursively traverses the construction table while building the maximum independent set.
fn reconstruct_mis<Set>(
    table: &dyn DynTable<Set>,
    root_id: usize,
    constr_table: &ConstructionTable,
    node_relations: &NodeRelations,
    adjaceny_matrix: &Vec<Vec<bool>>,
) -> HashSet<usize>
where
    Set: Eq + std::fmt::Debug + Clone + Default,
{
    /// Recursively traverses the table while building the maximum independent set.
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

        let (subset, size) = table.get_by_index(bag_id, set_index);
        println!("({set_index}): MIS at M[{bag_id}, {subset:?}] = {size}:\t{mis:?}");

        let children = &node_relations.children[&bag_id];
        // println!("{bag_id}'s children: {children:?}");

        match children.len() {
            // Leaf node: We're finished.
            0 => {
                // println!("{bag_id} is a leaf node => MIS = {mis:?}");
                mis
            }

            // Check the child.
            1 => {
                let child_index = constr_table[bag_id][set_index].0.unwrap();
                // println!(
                //     "{bag_id}'s {set_index} {:?} was constructed by {}'s {} {:?}",
                //     table.get_by_index(bag_id, set_index),
                //     children[0],
                //     child_index,
                //     table.get_by_index(children[0], child_index),
                // );

                rec(
                    table,
                    children[0],
                    constr_table[bag_id][set_index].0.unwrap(),
                    &constr_table,
                    mis,
                    &node_relations,
                )
            }

            // Find the maximum from the left and right child.
            2 => {
                let left = constr_table[bag_id][set_index].0.unwrap();
                let right = constr_table[bag_id][set_index].1.unwrap();
                // println!(
                //     "{bag_id}'s {set_index} {:?} was constructed by {}'s {} {:?} and {}'s {} {:?}",
                //     table.get_by_index(bag_id, set_index),
                //     children[0],
                //     constr_table[bag_id][set_index].0.unwrap(),
                //     table.get_by_index(children[0], left),
                //     children[1],
                //     constr_table[bag_id][set_index].1.unwrap(),
                //     table.get_by_index(children[1], right),
                // );

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

                mis = HashSet::from_iter(left.union(&right).into_iter().copied());
                mis
            }

            _ => panic!("Unreachable"),
        }
    }

    // We find all subsets with the largest number in the root entry.
    let root_sets = table.get_max_root_set_indices(root_id);
    println!("MIS size according to table: {}", root_sets[0].1);

    // We try to reconstruction the independent sets until one is found.
    for (index, size) in root_sets.into_iter() {

        let mut result = HashSet::new();
        result = rec(
            table,
            root_id,
            index,
            &constr_table,
            result,
            &node_relations,
        );

        let connectes_vertices = find_connected_vertices(&result, &adjaceny_matrix);
        if connectes_vertices.len() == 0 {
            return result; // Return the found MIS.
        }
    }

    HashSet::new() // No MIS found.
}

/// Finds the maximum independent set as described in the
/// [wiki](https://github.com/manuEbg/thm-ptas/wiki/Maximum-Independent-Set-with-Dynamic-Programming-on-Nice-Tree-Decompositions.)
/// for this project.
/// The nice tree decomposition [`ntd`] is traversed in post order (left child, right child, parent) and
/// the independence is checked by the [`adjaceny_matrix`].
/// This implementation uses the [NormalDynTable].
pub fn find_mis(
    adjaceny_matrix: &Vec<Vec<bool>>,
    ntd: &NiceTreeDecomposition,
) -> Result<(HashSet<usize>, usize), FindMisError> {
    let mut table = NormalDynTable::default();

    // TODO: @speed Don't use dynamic vectors, instead compute the maximum size required with
    // `max_bag_size`? This should be at max something like the length of the spanning tree.
    // Idea: The i-th set was constructed by the j-th child set.
    let mut constr_table: ConstructionTable = vec![Vec::new(); ntd.td.bags().len()];

    for bag in PostOrderIter::new(&ntd.td) {
        let children = &ntd.relations.children[&bag.id];
        let mut entry = DynTableValue::default();

        match children.len() {
            0 => {
                // Leaf node.
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
                let child_entry = table.0.get(&child.id).unwrap();
                if let Some(&v) = bag.vertex_set.difference(&child.vertex_set).nth(0) {
                    // Introduce node.
                    for subset in SubsetIter::new(&bag.vertex_set) {
                        if !subset.contains(&v) {
                            let (i, size) = table.get(child.id, &subset);
                            println!(
                                "{v} notin {subset:?} => M[{}, {subset:?}] = M[{}, {subset:?}] = {size}",
                                bag.id, child.id
                            );
                            entry.sets.push(DynTableValueItem::new(subset, size));
                            constr_table[bag.id].push((Some(i), None)); // Reconstruction.
                        } else if is_independent(&adjaceny_matrix, v, &subset) {
                            // @speed This clone could be expensive.
                            let mut clone = subset.clone();
                            clone.remove(&v);
                            // println!(
                            //     "{v} in {subset:?} => M[{}, {subset:?}] = M[{}, {clone:?}] + 1 = {size} + 1",
                            //     bag.id, child.id
                            // );
                            let (i, size) = table.get(child.id, &clone);
                            entry
                                .sets
                                .push(DynTableValueItem::new(subset, size + MisSize::Valid(1)));
                            constr_table[bag.id].push((Some(i), None)); // Reconstruction.
                        } else {
                            println!(
                                "{subset:?} is not independent => M[{}, S] = -infinity",
                                bag.id
                            );
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

                        let without = table.get(child.id, &subset);
                        let with = table.get(child.id, &clone);

                        let (i, value) = std::cmp::max_by(with, without, |w, wo| w.1.cmp(&wo.1));

                        entry.sets.push(DynTableValueItem::new(subset, value));
                        constr_table[bag.id].push((Some(i), None)); // Reconstruction.
                    }
                } else if child.vertex_set == bag.vertex_set {
                    // Some weird edge case. Is this a problem from an earlier phase or can we handle
                    // it this way?
                    for subset in SubsetIter::new(&bag.vertex_set) {
                        let (child_set_index, size) = table.get(child.id, &subset);
                        entry.sets.push(DynTableValueItem::new(subset, size));
                        // table.put(bag.id, subset, size);
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
                    let (i, left_size) = table.get(left_child.id, &subset);
                    let (j, right_size) = table.get(right_child.id, &subset);
                    let len = MisSize::Valid(subset.len());

                    println!("M[{}, {subset:?}] = M[{}, S] + M[{}, S] - |S| = {left_size} + {right_size} - {len} = {}", bag.id, left_child.id, right_child.id, left_size + right_size - len);
                    entry
                        .sets
                        .push(DynTableValueItem::new(subset, left_size + right_size - len));
                    constr_table[bag.id].push((Some(i), Some(j))); // Reconstruction.
                }
            }

            _ => assert!(false, "Unreachable"),
        }

        table.0.insert(bag.id.clone(), entry);
    }

    let mut out = File::create("normal_table.txt").unwrap();
    write!(out, "{}", NtdAndNormalTable { ntd, table: &table });

    let mut constr_out = File::create("normal_constr_table.txt").unwrap();
    dump_construction_table(constr_out, &constr_table, &table, &ntd.relations);

    let result = reconstruct_mis(&table, ntd.td.root.unwrap(), &constr_table, &ntd.relations, &adjaceny_matrix);

    Ok((result.clone(), result.len()))
}

/// Checks whether `v` is independent from all vertices in `set` or not.
fn is_independent_fast(adjaceny_matrix: &Vec<Vec<bool>>, v: usize, set: &BitSet) -> bool {
    set.iter().all(|u| !adjaceny_matrix[u][v])
}

/// Finds the maximum independent set as described in the
/// [wiki](https://github.com/manuEbg/thm-ptas/wiki/Maximum-Independent-Set-with-Dynamic-Programming-on-Nice-Tree-Decompositions.)
/// for this project.
/// The nice tree decomposition [`ntd`] is traversed in post order (left child, right child, parent) and
/// the independence is checked by the [`adjaceny_matrix`].
/// This implementation uses the [FastDynTable].
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

    let mut table_out = File::create("fast_table.txt").unwrap();
    write!(table_out, "{}", NtdAndFastTable { ntd, table: &table });

    let mut constr_out = File::create("fast_constr_table.txt").unwrap();
    dump_construction_table(constr_out, &constr_table, &table, &ntd.relations);

    let result = reconstruct_mis(&table, ntd.td.root.unwrap(), &constr_table, &ntd.relations, &adjaceny_matrix);

    Ok((result.clone(), result.len()))
}

/// Finds the maximum independent set by checking all subsets of the graph for independence and
/// keeping the biggest.
pub fn find_mis_exhaustive(
    adjaceny_matrix: &Vec<Vec<bool>>,
) -> Result<(HashSet<usize>, usize), FindMisError> {
    let is_independent = |subset: &HashSet<usize>| {
        subset
            .iter()
            .all(|u| subset.iter().all(|v| !adjaceny_matrix[*u][*v]))
    };

    let mut max: HashSet<usize> = HashSet::new();
    let combinations = u32::pow(2, adjaceny_matrix.len() as u32);
    for (i, subset) in SubsetIter::new(&FxHashSet::from_iter(0..adjaceny_matrix.len())).enumerate() {
        if i % 100000 == 0 {
            println!(
                "{i}/{combinations}, {}%",
                f64::from(i as u32) / f64::from(combinations) * 100.0
            );
        }
        let subset2 = HashSet::from_iter(subset.into_iter());
        if is_independent(&subset2) && subset2.len() > max.len() {
            max = subset2;
        }
    }

    let len = max.len();
    Ok((max, len))
}

// TODO: If possible, merge the two `find_mis` algorithms.
// TODO: Would it be a nice idea to log the steps of the algorithm (print to a string buffer)?

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

/// Writes the construction table into a given file.
fn dump_construction_table<Set>(
    mut file: File,
    constr_table: &ConstructionTable,
    table: &dyn DynTable<Set>,
    node_relations: &NodeRelations,
) -> std::io::Result<()>
where
    Set: Eq + std::fmt::Debug + Clone + Default,
{
    for (bag_id, preds) in constr_table.iter().enumerate() {
        for (set_id, preds) in preds.iter().enumerate() {
            match preds {
                (None, None) => Ok(()),

                (Some(p), None) => {
                    let child_id = node_relations.children[&bag_id][0];
                    let (subset, size) = table.get_by_index(bag_id, set_id);
                    let (child_subset, child_size) = table.get_by_index(child_id, *p);
                    writeln!(
                        file,
                        "{bag_id}'s set {set_id} ({:?}, {}) from child {child_id}'s set {p} ({:?}, {})",
                        subset, size,
                        child_subset, child_size,
                    )
                }

                (Some(l), Some(r)) => {
                    let left_id = node_relations.children[&bag_id][0];
                    let right_id = node_relations.children[&bag_id][1];
                    writeln!(
                        file,
                        "{bag_id}'s set {set_id} {:?} from left child {left_id}'s set {l} {:?} and right child {right_id}'s set {r} {:?}",
                        table.get_by_index(bag_id, set_id).0,
                        table.get_by_index(left_id, *l).0,
                        table.get_by_index(right_id, *r).0,
                    )
                }

                _ => panic!("Unreachable"),
            };
        }
    }

    Ok(())
}

/// Finds the connected vertices of a set.
pub fn find_connected_vertices(
    set: &HashSet<usize>,
    adjaceny_matrix: &Vec<Vec<bool>>,
) -> Vec<usize> {
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
            mis_finder::{find_connected_vertices, find_mis, find_mis_fast},
            nice_tree_decomp::NiceTreeDecomposition,
            node_relations::NodeRelations,
            tree_decomposition::td_write_to_dot,
        },
        read_graph_file_into_dcel_builder,
    };
    use arboretum_td::tree_decomposition::TreeDecomposition;
    use fxhash::FxHashSet;
    use std::{fs::File, process::Command};

    use super::{find_mis_exhaustive, ConstructionTable};

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
        assert!(
            connected_vertices.len() == 0,
            "Set is not independet: {:?}",
            connected_vertices
        );
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
        assert!(
            connected_vertices.len() == 0,
            "Set is not independet: {:?}",
            connected_vertices
        );
    }

    #[test]
    fn exhaustive() {
        let mut dcel_b = read_graph_file_into_dcel_builder("data/problem.graph").unwrap();
        let mut dcel = dcel_b.build();
        let adjacency_matrix = dcel.adjacency_matrix();

        let result = find_mis_exhaustive(&adjacency_matrix);
        println!("Exhaustive result: {result:?}");
    }
}
