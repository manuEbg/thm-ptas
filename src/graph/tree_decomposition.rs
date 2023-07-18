use crate::graph::approximated_td::ApproximatedTD;
use arboretum_td::tree_decomposition::TreeDecomposition;
use fxhash::FxHashSet;

/// This function is used to create a tree decomposition on one of the rings
/// in a DCEL data structure.
impl From<&ApproximatedTD<'_>> for TreeDecomposition {
    fn from(approx_td: &ApproximatedTD) -> Self {
        let dcel = approx_td.graph();
        let faces = &dcel.faces();
        let mut result = TreeDecomposition {
            bags: vec![],
            root: None,
            max_bag_size: faces
                .iter()
                .map(|face| face.walk_face(dcel).len())
                .fold(0, Ord::max),
        };

        for face in *faces {
            let mut vertices: FxHashSet<usize> = FxHashSet::default();
            for arc in face.walk_face(&dcel) {
                vertices.insert(dcel.arc(arc).src());
            }
            result.add_bag(vertices);
        }

        for i in 0..approx_td.num_bags() {
            let neighbors = &approx_td.adjacent()[i];
            for n in neighbors {
                result.add_edge(i, *n);
            }
        }

        result
    }
}

pub trait NiceTreeDecomposition {
    fn make_nice(&self) -> TreeDecomposition;
}

impl NiceTreeDecomposition for TreeDecomposition {
    fn make_nice(&self) -> TreeDecomposition {
        let mut result = self.clone();
        assert_eq!(self.bags.len(), result.bags.len());

        for bag in self.bags.iter() {
            let mut pred_id = bag.id;
            let vertices = bag.vertex_set.iter().copied().collect::<Vec<usize>>();

            for i in (0..bag.vertex_set.len()).rev() {
                let mut vs: FxHashSet<usize> = FxHashSet::default();
                for v2 in vertices[0..i].iter() {
                    vs.insert(*v2);
                }
                let new_id = result.add_bag(vs);
                result.add_edge(pred_id, new_id);
                pred_id = new_id;
            }
        }

        result
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::graph::approximated_td::TDBuilder;
    use crate::graph::dcel::spanning_tree::SpanningTree;
    use crate::read_graph_file;

    #[test]
    pub fn test_tree_decomposition() {
        let dcel = read_graph_file("data/exp.graph").unwrap();
        let mut spanning_tree = SpanningTree::new(&dcel);
        spanning_tree.build(0);
        let mut b = TDBuilder::new(&spanning_tree);
        let approx_td = ApproximatedTD::from(&mut b);
        let tree_decomposition = TreeDecomposition::from(&approx_td);

        println!("Normal tree decomposition:");
        for bag in tree_decomposition.bags.iter() {
            println!("{:?}", bag);
        }

        assert_eq!(tree_decomposition.bags.len(), 4);
    }

    #[test]
    pub fn test_nice_tree_decomposition() {
        let dcel = read_graph_file("data/exp.graph").unwrap();
        let mut spanning_tree = SpanningTree::new(&dcel);
        spanning_tree.build(0);
        let mut b = TDBuilder::new(&spanning_tree);
        let approx_td = ApproximatedTD::from(&mut b);
        let tree_decomposition = TreeDecomposition::from(&approx_td);

        let nice_td = tree_decomposition.make_nice();

        assert_eq!(nice_td.bags.len(), 18);
        let mut found = vec![false; 18];

        println!("Nice tree decomposition:");
        for bag in nice_td.bags.iter() {
            found[bag.id] = true;
            println!("{:?}", bag);
        }

        assert!(found.iter().all(|f| *f == true));
    }
}
