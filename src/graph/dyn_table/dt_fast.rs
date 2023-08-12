/* New dynamic table idea: @speed
 * - Bit set as subset.
 */

use bit_set::BitSet;
use std::collections::{HashMap, HashSet};

use crate::graph::mis_finder::{DynTable, MisSize};

/* The problem with the implementation that uses referenes:
table.put(1, set.clone(), MisSize::Valid(6));
----- mutable borrow occurs here

let (set_index, size) = table.get(2, &set);
^^^^^
|
immutable borrow occurs here
mutable borrow later used here
*/
/// This type is essentially a 2D matrix where the first value of the tuple is the bag ID and the
/// second index is a subset of vertices from the current subgraph (donut).
#[derive(Debug)]
pub struct FastDynTable {
    /// (bag ID, subset index) -> size
    map: HashMap<(usize, usize), MisSize>,
    /// (bag ID, subset set) -> subset index
    set_indices: HashMap<(usize, BitSet), usize>,
    /// (bag ID, subset index) -> subset
    set_indices_back: HashMap<(usize, usize), BitSet>,
    /// bag ID -> number of subsets
    set_count: HashMap<usize, usize>,
    // subsets: HashSet<BitSet>, // This could be a cache but it caused borrow checker errors.
}

impl FastDynTable {
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

    fn get_max_root_set_index(&self, root_id: usize) -> usize {
        (0..self.set_count[&root_id])
            .fold((0 as usize, MisSize::Valid(0)), |result, set_index| {
                let set_size = self.map[&(root_id, set_index)];
                if result.1 < set_size {
                    (set_index, set_size)
                } else {
                    result
                }
            })
            .0
    }

    fn put<'b: 'a>(&'a mut self, bag_id: usize, subset: BitSet, size: MisSize) {
        let set_count = self.set_count.entry(bag_id).or_insert(0);
        self.map.insert((bag_id, set_count.clone()), size);
        self.set_indices
            .insert((bag_id, subset.clone()), set_count.clone());
        // TODO: @speed Get rid of this copy.
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
