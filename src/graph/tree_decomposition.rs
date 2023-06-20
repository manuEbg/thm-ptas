use crate::graph::dual_graph::DualGraph;
use arboretum_td::tree_decomposition::TreeDecomposition;
use fxhash::FxHashSet;

/// This function is used to create a tree decomposition on one of the rings
/// in a DCEL data structure.
impl From<&DualGraph<'_>> for TreeDecomposition {
    fn from(dual_graph: &DualGraph) -> Self {
        let dcel = dual_graph.get_dcel();
        let faces = &dcel.get_faces();
        let mut result = TreeDecomposition {
            bags: vec![],
            root: None,
            max_bag_size: faces
                .iter()
                .map(|face| face.walk_face(dcel).len())
                .fold(0, |max, v| Ord::max(max, v)),
        };

        for face in *faces {
            let mut vertices: FxHashSet<usize> = FxHashSet::default();
            for arc in face.walk_face(&dcel) {
                vertices.insert(dcel.get_arc(arc).get_src());
            }
            result.add_bag(vertices);
        }

        for i in 0..dual_graph.get_adjacent().len() {
            let neighbors = &dual_graph.get_adjacent()[i];
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
            let vertices = bag.vertex_set.iter().map(|v| *v).collect::<Vec<usize>>();

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
