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
