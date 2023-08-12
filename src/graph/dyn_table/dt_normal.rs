use crate::graph::mis_finder::{DynTable, MisSize};
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

    fn get_max_root_set_index(&self, root_id: usize) -> usize {
        self.0[&root_id]
            .sets
            .iter()
            .enumerate()
            .max_by(|(_, l), (_, r)| l.size.cmp(&r.size))
            .unwrap()
            .0
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
