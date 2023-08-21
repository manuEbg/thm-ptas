/* New dynamic table idea: @speed
 * - Bit set as subset.
 */

use bit_set::BitSet;
use std::collections::{HashMap, HashSet};
use crate::graph::iterators::post_order::PostOrderIter;

use crate::graph::iterators::subset::SubBitSetIter;
use crate::graph::{mis_finder::{DynTable, MisSize}, nice_tree_decomp::NiceTreeDecomposition};

/* The problem with the implementation that uses referenes:
table.put(1, set.clone(), MisSize::Valid(6));
----- mutable borrow occurs here

let (set_index, size) = table.get(2, &set);
^^^^^
|
immutable borrow occurs here
mutable borrow later used here
*/
/// This table uses bitsets to for the subsets. It implements the [DynTable] trait but has several
/// internal values that store information.
#[derive(Debug)]
pub struct FastDynTable {
    /// Associates a bag ID and a subset index with the corresponding maximum independent set size.
    /// To get the index of a given subset, see [FastDynTable::set_indces].
    /// (bag ID, subset index) -> maximum independent set size.
    map: HashMap<(usize, usize), MisSize>,

    /// Associates a bag ID and a subset with its corresponding subset index.
    /// A subset gets its index when its inserted by the [DynTable::put] function and only these
    /// subsets are valid.
    /// (bag ID, subset set) -> subset index.
    set_indices: HashMap<(usize, BitSet), usize>,

    /// Associates a bag ID and a subset index with the corresponding subset.
    /// It is the inverse of [FastDynTable::set_indices] and only needed for internal bookkeeping.
    /// (bag ID, subset index) -> subset.
    set_indices_back: HashMap<(usize, usize), BitSet>,

    /// Stores the amount of subsets for all bag IDs.
    /// bag ID -> number of subsets.
    set_count: HashMap<usize, usize>,

    // subsets: HashSet<BitSet>, // This could be a cache but it caused borrow checker errors.
}

impl FastDynTable {
    /// Creates a new dynamic table that uses bitsets.
    /// Note: The [subset_count] is unused for now.
    pub fn new(subset_count: usize) -> Self {
        FastDynTable {
            map: HashMap::default(),
            set_indices: HashMap::default(),
            set_indices_back: HashMap::default(),
            set_count: HashMap::default(),
            // subsets: HashSet::with_capacity(subset_count),
        }
    }
}

impl<'a> DynTable<'a, BitSet> for FastDynTable {
    fn get(&self, bag_id: usize, subset: &BitSet) -> (usize, MisSize) {
        // TODO: @speed Get rid of this copy.
        let subset_index = self.set_indices[&(bag_id, subset.clone())];
        (subset_index, self.map[&(bag_id, subset_index)])
    }

    fn get_by_index(&self, bag_id: usize, subset_index: usize) -> (&BitSet, MisSize) {
        (
            &self.set_indices_back[&(bag_id, subset_index)],
            self.map[&(bag_id, subset_index)],
        )
    }

    fn get_max_root_set_indices(&self, root_id: usize) -> Vec<(usize, MisSize)> {
        let max_size: MisSize = (0..self.set_count[&root_id]).map(|i| self.map[&(root_id, i)]).max_by(|l, r| l.cmp(&r)).unwrap();
        let mut result = Vec::new();
        for set_index in 0..self.set_count[&root_id] {
            let set_size = self.map[&(root_id, set_index)];
            if set_size == max_size {
                result.push((set_index, set_size));
            }
        }
        result
    }

    fn put<'b: 'a>(&'a mut self, bag_id: usize, subset: BitSet, size: MisSize) {
        let set_count = self.set_count.entry(bag_id).or_insert(0);
        // TODO: @speed Get rid of this copies. -> Problem: Borrow checker and we need the nightly
        // compiler for this.
        self.map.insert((bag_id, set_count.clone()), size);
        self.set_indices
            .insert((bag_id, subset.clone()), set_count.clone());
        self.set_indices_back
            .insert((bag_id, set_count.clone()), subset);
        *set_count += 1;
    }

    fn add_to_mis(&self, bag_id: usize, subset_index: usize, mis: &mut HashSet<usize>) {
        self.set_indices_back[&(bag_id, subset_index)]
            .iter()
            .for_each(|v| {
                mis.insert(v);
            });
    }
}

/// A wrapper so that we can implement [std::fmt::Display] for a nice tree decomposition and its
/// dynamic table.
pub struct NtdAndFastTable<'a> {
    pub ntd: &'a NiceTreeDecomposition,
    pub table: &'a FastDynTable,
}

impl std::fmt::Display for NtdAndFastTable<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for bag in PostOrderIter::new(&self.ntd.td) {
            for subset in SubBitSetIter::new(&bag.vertex_set) {
                writeln!(f, "M[{}, {subset:?}] = {}", bag.id, self.table.get(bag.id, &subset).1)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use bit_set::BitSet;
    use fxhash::FxHashSet;

    use crate::graph::{
        dyn_table::dt_fast::FastDynTable,
        iterators::subset::SubsetIter,
        mis_finder::{DynTable, MisSize},
    };

    #[test]
    fn test_subsets() {
        let set = FxHashSet::from_iter(vec![1, 2, 3, 4].into_iter());
        for (i, subset) in SubsetIter::new(&set).enumerate() {
            println!("{i}. {:?}", subset);
        }
    }

    #[test]
    fn bitset() {
        let mut table: FastDynTable = FastDynTable::new(3);
        let mut set = BitSet::new();
        set.insert(1);
        set.insert(3);
        let set2 = set.clone();
        table.put(2, set, MisSize::Valid(6));

        dbg!(&table);
        let (set_index, size) = table.get(2, &set2);
        println!("Set = {set_index}, size = {size}");
        let (subset, size2) = table.get_by_index(2, 1);
        println!("Subset = {subset:?}, size = {size2}");
    }
}
