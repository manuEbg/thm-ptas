use crate::graph::approximated_td::ApproximatedTD;
use arboretum_td::tree_decomposition::{Bag, TreeDecomposition};
use fxhash::FxHashSet;
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

// TODO: Split this file into multiple and a separate directory.

// TODO: @cleanup This can probably be removed.
/// This function is used to create a tree decomposition on one of the rings
/// in a DCEL data structure.
impl From<&ApproximatedTD<'_>> for TreeDecomposition {
    fn from(atd: &ApproximatedTD) -> Self {
        let mut result = TreeDecomposition {
            bags: vec![],
            root: None,
            max_bag_size: 0,
        };

        let mut max_bag_size = 0;

        atd.bags().iter().for_each(|bag| {
            if bag.len() > max_bag_size {
                max_bag_size = bag.len();
            }
            result
                .add_bag(FxHashSet::from_iter(bag.iter().copied()));
        });

        for i in 0..atd.adjacent().len() {
            let neighbors = &atd.adjacent()[i];
            for n in neighbors {
                result.add_edge(i, *n);
            }
        }

        result.max_bag_size = max_bag_size;
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

    let ins_len = intersection.len();

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

struct NTD {
    td: TreeDecomposition,
    relations: NodeRelations,
    bag_types: usize,
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

struct PostOrderIter<'a> {
    td: &'a TreeDecomposition,
    stack: Vec<usize>,  // Just bag IDs.
    visited: Vec<bool>, // Improvement: Use a bitset.
}

impl<'a> PostOrderIter<'a> {
    pub fn new(td: &'a TreeDecomposition) -> Self {
        PostOrderIter {
            td,
            stack: vec![td.root.unwrap()],
            visited: vec![false; td.bags.len()],
        }
    }

    /// Recursively traverse the subtrees.
    /// We push the child nodes first from left to right.
    /// It is assumed that the sub root is already on the stack.
    fn traverse_subtrees(&mut self, sub_root: &Bag) {
        for &child_id in sub_root.neighbors.iter() {
            if !self.visited[child_id] {
                let child = &self.td.bags[child_id];
                self.stack.push(child_id);
                self.visited[child_id] = true;
                self.traverse_subtrees(child);
            }
        }
    }
}

impl<'a> Iterator for PostOrderIter<'a> {
    type Item = &'a Bag;

    fn next(&mut self) -> Option<Self::Item> {
        // Traverse subtrees until we find a leaf node to return.
        while let Some(&current_id) = self.stack.last() {
            let current = &self.td.bags[current_id];
            if current.neighbors.len() == 0 || self.visited[current_id] {
                self.stack.pop();
                return Some(current);
            }

            self.visited[current_id] = true;
            self.traverse_subtrees(current);
        }
        None
    }
}

struct SubsetIter<T: Eq + Hash + Copy> {
    set: Vec<T>, // We want to get the elements one by another, so a vector is useful.
    element_index: usize,
    subsets: Vec<FxHashSet<T>>,
    subset_index: usize,
}

impl<T: Eq + Hash + Copy> SubsetIter<T> {
    fn new(set: &FxHashSet<T>) -> Self {
        let items = set.iter().copied().collect::<Vec<T>>();
        SubsetIter {
            set: items,
            element_index: 0,
            subsets: vec![FxHashSet::from_iter(Vec::new().into_iter())],
            subset_index: 0,
        }
    }
}

impl<T: Eq + Hash + Copy> Iterator for SubsetIter<T> {
    type Item = FxHashSet<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.subset_index < self.subsets.len() {
            self.subset_index += 1;
            return self
                .subsets
                .get(self.subset_index - 1)
                .map(|subset| subset.clone());
        }

        if self.element_index >= self.set.len() {
            return None;
        }

        let new_subsets = self
            .subsets
            .iter()
            .map(|subset| {
                let mut clone = subset.clone();
                clone.insert(self.set[self.element_index]);
                clone
            })
            .collect::<Vec<FxHashSet<T>>>();

        new_subsets
            .into_iter()
            .for_each(|set| self.subsets.push(set));

        self.element_index += 1;

        self.next()
    }
}

/* New dynamic table idea: @speed
 * - Bit set as subset.
 */

#[derive(Debug)]
struct DynTableValue {
    sets: Vec<DynTableValueItem>,
}

impl DynTableValue {
    fn add(&mut self, item: DynTableValueItem) {
        self.sets.push(item);
    }
}

impl Default for DynTableValue {
    fn default() -> Self {
        DynTableValue { sets: Vec::new() }
    }
}

impl std::fmt::Display for DynTableValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.sets
            .iter()
            .enumerate()
            .fold(Ok(()), |result, (index, set)| {
                result.and_then(|()| {
                    write!(f, "{}", set)?;

                    if index < self.sets.len() - 1 {
                        write!(f, ", ")?;
                    }

                    Ok(())
                })
            })
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum MisSize {
    Invalid,
    Valid(usize),
}

impl MisSize {
    fn get(&self) -> usize {
        match self {
            MisSize::Invalid => panic!("Mis size is invalid!"),
            MisSize::Valid(s) => *s,
        }
    }
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

// TODO: Maybe change to a tuple struct.
#[derive(Debug)]
struct DynTableValueItem {
    mis: FxHashSet<usize>,
    size: MisSize,
}

impl DynTableValueItem {
    fn new(mis: FxHashSet<usize>, size: MisSize) -> Self {
        DynTableValueItem { mis, size }
    }
}

impl std::fmt::Display for DynTableValueItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}, {}", self.mis, self.size)
    }
}

fn print_dyn_table(table: &HashMap<usize, DynTableValue>) {
    for (key, val) in table.iter() {
        println!("M[{key}] = {val}");
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

fn find_child_size(entry: &DynTableValue, set: &FxHashSet<usize>) -> MisSize {
    entry
        .sets
        .iter()
        .find(|item| item.mis == *set)
        .unwrap()
        .size
}

fn is_independent(adjaceny_matrix: &Vec<Vec<bool>>, v: usize, set: &FxHashSet<usize>) -> bool {
    set.iter().all(|u| !adjaceny_matrix[v][*u])
}

// TODO:
// 1. Maybe change data structure for the nice tree decomposition.
// 2. Add an adjaceny matrix/list to check whether two vertices are connected or not.
pub fn find_mis(
    adjaceny_matrix: Vec<Vec<bool>>,
    ntd: &TreeDecomposition,
    node_relations: &NodeRelations,
) -> Result<(FxHashSet<usize>, usize), FindMisError> {
    type DynTable = HashMap<usize, DynTableValue>;
    let mut dyn_table: DynTable = HashMap::new();

    for bag in PostOrderIter::new(&ntd) {
        let children = &node_relations.children[&bag.id];
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
                let child = &ntd.bags[children[0]];
                let child_entry = dyn_table.get(&child.id).unwrap();
                if let Some(&v) = bag.vertex_set.difference(&child.vertex_set).nth(0) {
                    // Introduce node.
                    println!(
                        "Introduce {v}: B{} -> B{} = {:?} -> {:?}",
                        &child.id, &bag.id, &child.vertex_set, &bag.vertex_set
                    );

                    for subset in SubsetIter::new(&bag.vertex_set) {
                        if !subset.contains(&v) {
                            let value = find_child_size(&child_entry, &subset);
                            entry.sets.push(DynTableValueItem::new(subset, value));
                        } else if is_independent(&adjaceny_matrix, v, &subset) {
                            // @speed This clone could be expensive.
                            let mut clone = subset.clone();
                            clone.remove(&v);
                            println!("{subset:?} + 1");
                            entry.sets.push(DynTableValueItem::new(
                                subset,
                                find_child_size(&child_entry, &clone) + MisSize::Valid(1),
                            ));
                        } else {
                            println!("{subset:?} is not independent.");
                            entry
                                .sets
                                .push(DynTableValueItem::new(subset, MisSize::Invalid));
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

                        let without_value = find_child_size(&child_entry, &subset);
                        let with_value = find_child_size(&child_entry, &clone);

                        // Reconstruction: Take the set that has a greater size.
                        entry.sets.push(DynTableValueItem::new(
                            subset,
                            std::cmp::max(without_value, with_value),
                        ));
                    }
                }
            }

            2 => {
                // Join node.
                // forall subsets of bag: M[bag, subset] = M[lc, subset] + M[rc, subset] - |subset|

                let left_child = &ntd.bags[children[0]];
                let right_child = &ntd.bags[children[1]];
                let left_entry = dyn_table.get(&left_child.id).unwrap();
                let right_entry = dyn_table.get(&right_child.id).unwrap();

                for subset in SubsetIter::new(&bag.vertex_set) {
                    let left_value = find_child_size(&left_entry, &subset);
                    let right_value = find_child_size(&right_entry, &subset);
                    let len = MisSize::Valid(subset.len());

                    println!("{left_value} + {right_value} - {len}");
                    entry.sets.push(DynTableValueItem::new(
                        subset,
                        left_value + right_value - len,
                    ));
                }
            }

            _ => assert!(false, "Unreachable"),
        }

        dyn_table.insert(bag.id.clone(), entry);
    }

    print_dyn_table(&dyn_table);

    fn find_largest_mis_in_bag<'a>(entry: &'a DynTableValue) -> &'a DynTableValueItem {
        entry
            .sets
            .iter()
            .max_by(|&x, &y| x.size.cmp(&y.size))
            .unwrap()
    }

    // TODO: Reconstruction
    // 1. Remember for each set from what it was constructed.
    // 2. Find the entry with the largest value and then walk the constructions bag.
    let mut result: Option<&DynTableValueItem> = None;
    for entry in dyn_table.values() {
        match result {
            None => {
                result = Some(find_largest_mis_in_bag(&entry));
            }
            Some(e) => {
                let max = find_largest_mis_in_bag(&entry);
                if max.size > e.size {
                    result = Some(max);
                }
            }
        }
    }

    result
        .ok_or(FindMisError::NoMisFound)
        .map(|i| (i.mis.clone(), i.size.get()))
}

fn validate_nice_td(
    otd: &TreeDecomposition,
    ntd: &TreeDecomposition,
    relations: &NodeRelations,
) -> bool {
    let mut present = vec![false; otd.bags.len()]; // Check if all original bags are present.
    ntd.bags.iter().all(|bag| {
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
    }) && present.iter().all(|&v| v)
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

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::graph::approximated_td::{ApproximatedTD, TDBuilder};
    use crate::graph::dcel::spanning_tree::SpanningTree;
    use crate::read_graph_file_into_dcel_builder;
    use std::io::Error;
    use std::process::Command;

    #[test]
    pub fn test_tree_decomposition() {
        let mut dcel_b = read_graph_file_into_dcel_builder("data/exp.graph").unwrap();
        let dcel = dcel_b.build();
        let mut spanning_tree = SpanningTree::new(&dcel);
        spanning_tree.build(0);
        let mut td_builder = TDBuilder::new(&spanning_tree);
        let atd = ApproximatedTD::from(&mut td_builder);
        let tree_decomposition = TreeDecomposition::from(&atd);

        println!("Normal tree decomposition:");
        for bag in tree_decomposition.bags.iter() {
            println!("{:?}", bag);
        }

        assert_eq!(tree_decomposition.bags.len(), 4);
    }

    #[test]
    pub fn test_nice_tree_decomposition() -> Result<(), Error> {
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
        assert!(validate_nice_td(&td, &nice_td, &ntd_rels));

        let ntd_path = "ntd.dot";
        let mut ntd_out = File::create(ntd_path)?;
        td_write_to_dot("ntd", &mut ntd_out, &nice_td, &ntd_rels)?;
        Command::new("dot")
            .args(["-Tpdf", ntd_path, "-o", "ntd.pdf"])
            .spawn()
            .expect("dot command did not work.");

        Ok(())
    }

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

    #[test]
    fn test_subsets() {
        let set = FxHashSet::from_iter(vec![1, 2, 3, 4].into_iter());
        for (i, subset) in SubsetIter::new(&set).enumerate() {
            println!("{i}. {:?}", subset);
        }
    }
}
