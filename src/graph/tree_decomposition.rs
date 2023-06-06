use crate::graph::Dcel;
use arboretum_td::tree_decomposition::TreeDecomposition;
use fxhash::FxHashSet;

impl From<&Dcel> for TreeDecomposition {
    fn from(dcel: &Dcel) -> Self {
        let faces = &dcel.get_faces();
        let mut result = TreeDecomposition {
            bags: vec![],
            root: None,
            max_bag_size: faces.len(),
        };

        for face in *faces {
            let mut vertices: FxHashSet<usize> = FxHashSet::default();
            for arc in face.walk_face(&dcel) {
                vertices.insert(arc);
            }
            result.add_bag(vertices);
        }

        // TODO: Add neighbours to the bags.
        // Do we need to use the spanning tree of the DCEL here?

        result
    }
}
