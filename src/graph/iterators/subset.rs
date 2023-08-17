use bit_set::BitSet;
use fxhash::FxHashSet;
use std::hash::Hash;

pub struct SubsetIter<T: Eq + Hash + Copy> {
    set: Vec<T>, // We want to get the elements one by another, so a vector is useful.
    element_index: usize,
    subsets: Vec<FxHashSet<T>>,
    subset_index: usize,
}

impl<T: Eq + Hash + Copy> SubsetIter<T> {
    pub fn new(set: &FxHashSet<T>) -> Self {
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

pub struct SubBitSetIter {
    set: Vec<usize>, // We want to get the elements one by another, so a vector is useful.
    element_index: usize,
    subsets: Vec<BitSet>,
    subset_index: usize,
}

impl SubBitSetIter {
    pub fn new(set: &FxHashSet<usize>) -> Self {
        let items = set.iter().copied().collect::<Vec<usize>>();
        SubBitSetIter {
            set: items,
            element_index: 0,
            subsets: vec![BitSet::from_iter(Vec::new().into_iter())],
            subset_index: 0,
        }
    }
}

impl Iterator for SubBitSetIter {
    type Item = BitSet;

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
            .collect::<Vec<BitSet>>();

        new_subsets
            .into_iter()
            .for_each(|set| self.subsets.push(set));

        self.element_index += 1;

        self.next()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use fxhash::FxHashSet;

    #[test]
    fn sub_bitsets() {
        println!("Sub bitsets!");
        let set = FxHashSet::from_iter(vec![1, 3, 10]);

        for (i, subset) in SubBitSetIter::new(&set).enumerate() {
            println!("{i}. Subset: {:?}", subset);
        }
    }
}
