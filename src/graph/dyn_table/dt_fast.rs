/* New dynamic table idea: @speed
 * - Bit set as subset.
 */

use std::collections::HashMap;
use bit_set::BitSet;

/// This type is essentially a 2D matrix where the first value of the tuple is the bag ID and the
/// second index is a subset of vertices from the current subgraph (donut).
pub struct FastDynTable(HashMap<(usize, BitSet), usize>);

fn is_independent2(adjaceny_matrix: &Vec<Vec<bool>>, v: usize, set: &BitSet) -> bool {
    set.iter().all(|u| !adjaceny_matrix[v][u])
}

/*
#[cfg(test)]
pub mod tests {
    #[test]
    fn test_subsets() {
        let set = FxHashSet::from_iter(vec![1, 2, 3, 4].into_iter());
        for (i, subset) in SubsetIter::new(&set).enumerate() {
            println!("{i}. {:?}", subset);
        }
    }

    #[test]
    fn bitset() {
        let mut table: FastDynTable = HashMap::new();
        let mut set = BitSet::new();
        set.insert(1);
        set.insert(3);
        table.insert((2, set.clone()), 6);

        dbg!(&table);
        let size = table[&(2, set)];
        println!("Size = {size}");
    }
}
*/
