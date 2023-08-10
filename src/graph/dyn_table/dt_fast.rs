/* New dynamic table idea: @speed
 * - Bit set as subset.
 */

use bit_set::BitSet;
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::graph::mis_finder::{DynTable, MisSize};

/// This type is essentially a 2D matrix where the first value of the tuple is the bag ID and the
/// second index is a subset of vertices from the current subgraph (donut).
#[derive(Debug)]
pub struct FastDynTable {
    map: HashMap<(usize, BitSet), MisSize>,
    set_ids: HashMap<(usize, BitSet), usize>,
    set_count: HashMap<usize, usize>,
    // subsets: HashSet<BitSet>, // This could be a cache but it caused borrow checker errors.
}

impl FastDynTable {
    pub fn new(subset_count: usize) -> Self {
        FastDynTable {
            map: HashMap::default(),
            set_ids: HashMap::default(),
            set_count: HashMap::default(),
            // subsets: HashSet::with_capacity(subset_count),
        }
    }
}

/* The problem with the implementation that uses referenes:
table.put(1, set.clone(), MisSize::Valid(6));
----- mutable borrow occurs here

let (set_index, size) = table.get(2, &set);
^^^^^
|
immutable borrow occurs here
mutable borrow later used here
*/
impl<'a> DynTable<'a, BitSet> for FastDynTable {
    fn get(&self, bag_id: usize, subset: &BitSet) -> (usize, MisSize) {
        // TODO: @speed Get rid of these copies.
        (self.set_ids[&(bag_id, subset.clone())], self.map[&(bag_id, subset.clone())])
    }

    fn put<'b: 'a>(&'a mut self, bag_id: usize, subset: BitSet, size: MisSize) {
        let set_count = self.set_count.entry(bag_id).or_insert(0);
        *set_count += 1;
        // TODO: @speed Get rid of these copies.
        self.map.insert((bag_id, subset.clone()), size);
        self.set_ids.insert((bag_id, subset), set_count.clone());
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
    }
}
