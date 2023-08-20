use crate::graph::{
    iterators::{post_order::PostOrderIter, subset::SubsetIter},
    mis_finder::{DynTable, MisSize},
    nice_tree_decomp::NiceTreeDecomposition,
};
use fxhash::FxHashSet;
use std::collections::{HashMap, HashSet};

pub struct NormalDynTable(pub HashMap<usize, DynTableValue>);

impl Default for NormalDynTable {
    fn default() -> Self {
        NormalDynTable(HashMap::new())
    }
}

impl<'a> DynTable<'a, FxHashSet<usize>> for NormalDynTable {
    fn get(&self, bag_id: usize, subset: &FxHashSet<usize>) -> (usize, MisSize) {
        let entry = &self.0[&bag_id];
        find_child_size(&entry, &subset)
    }

    fn get_by_index(&self, bag_id: usize, subset_index: usize) -> (&FxHashSet<usize>, MisSize) {
        let item = &self.0[&bag_id].sets[subset_index];
        (&item.mis, item.size.clone())
    }

    fn get_max_root_set_indices(&self, root_id: usize) -> Vec<(usize, MisSize)> {
        let max_size = self.0[&root_id]
            .sets
            .iter()
            .max_by(|l, r| l.size.cmp(&r.size))
            .unwrap()
            .size;
        // println!("Normal max size in root: {max_size}");
        self.0[&root_id]
            .sets
            .iter()
            .enumerate()
            .filter(|e| e.1.size == max_size)
            .map(|(i, s)| (i, s.size))
            .collect::<Vec<_>>()
    }

    fn put<'b: 'a>(&'a mut self, bag_id: usize, subset: FxHashSet<usize>, size: MisSize) {
        if self.0.get(&bag_id).is_none() {
            self.0.insert(bag_id, DynTableValue::default());
        }
        self.0
            .get_mut(&bag_id)
            .unwrap()
            .add(DynTableValueItem::new(subset, size));
    }

    fn add_to_mis(&self, bag_id: usize, subset_index: usize, mis: &mut HashSet<usize>) {
        self.0[&bag_id].sets[subset_index]
            .mis
            .iter()
            .for_each(|&v| {
                mis.insert(v);
            });
    }
}

#[derive(Debug)]
pub struct DynTableValue {
    pub sets: Vec<DynTableValueItem>,
}

impl DynTableValue {
    pub fn add(&mut self, item: DynTableValueItem) {
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
                    write!(f, "({}: {})", index, set)?;

                    if index < self.sets.len() - 1 {
                        write!(f, ", ")?;
                    }

                    Ok(())
                })
            })
    }
}

pub struct NtdAndNormalTable<'a> {
    pub ntd: &'a NiceTreeDecomposition,
    pub table: &'a NormalDynTable,
}

impl std::fmt::Display for NtdAndNormalTable<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for bag in PostOrderIter::new(&self.ntd.td) {
            for subset in SubsetIter::new(&bag.vertex_set) {
                let mut sorted_subset = Vec::from_iter(subset.iter().copied());
                sorted_subset.sort();
                writeln!(
                    f,
                    "M[{}, {}] = {}",
                    bag.id,
                    format_vec_as_set(sorted_subset),
                    self.table.get(bag.id, &subset).1
                )?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

fn format_vec_as_set(vec: Vec<usize>) -> String {
    let vec_str = format!("{:?}", vec);
    let inner_str = String::from(&vec_str[1..vec_str.len() - 1]);
    format!("{{{inner_str}}}")
}

// TODO: Maybe change to a tuple struct.
#[derive(Debug)]
pub struct DynTableValueItem {
    pub mis: FxHashSet<usize>,
    pub size: MisSize,
}

impl DynTableValueItem {
    pub fn new(mis: FxHashSet<usize>, size: MisSize) -> Self {
        DynTableValueItem { mis, size }
    }
}

impl std::fmt::Display for DynTableValueItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}, {}", self.mis, self.size)
    }
}

impl std::fmt::Display for NormalDynTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (key, val) in self.0.iter() {
            writeln!(f, "M[{key}] = {val}")?;
        }
        Ok(())
    }
}

fn find_child_size(entry: &DynTableValue, set: &FxHashSet<usize>) -> (usize, MisSize) {
    entry
        .sets
        .iter()
        .enumerate()
        .find(|tup| tup.1.mis == *set)
        .map(|(i, item)| (i, item.size))
        .unwrap()
}
