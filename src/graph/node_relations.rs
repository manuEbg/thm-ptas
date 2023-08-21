use std::collections::{HashMap, VecDeque};

use arboretum_td::tree_decomposition::TreeDecomposition;

/// Represents the parent of a bag. The [`Fake`] value is used for the root since it has no real
/// parent.
#[derive(Clone, Copy)]
pub enum NodeParent {
    Fake,
    Real(usize),
}

// @speed The hash map could be replaced with a parent matrix.
/// Associates bag IDs with their parent and their children.
pub struct NodeRelations {
    pub parent: HashMap<usize, NodeParent>,
    pub children: HashMap<usize, Vec<usize>>,
}

impl NodeRelations {
    /// Creates a new relation struct by traversing the tree decomposition in breadth first search
    /// order.
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
