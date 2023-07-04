use super::{Arc, ArcId, Dcel};

pub type FaceId = usize;

#[derive(Debug)]
pub struct Face {
    start_arc: ArcId,
}

impl<'a> Face {
    pub fn new(start_arc: ArcId) -> Self {
        Face { start_arc }
    }

    pub fn walk_face(&self, dcel: &Dcel) -> Vec<ArcId> {
        let mut arcs = vec![];
        for (id, _) in self.iter(dcel) {
            arcs.push(id);
        }
        arcs
    }

    pub fn iter(&self, dcel: &'a Dcel) -> FaceIterator<'a> {
        FaceIterator::new(dcel, self.start_arc())
    }

    pub fn start_arc(&self) -> ArcId {
        self.start_arc
    }

    pub fn set_start_arc(&mut self, start_arc: ArcId) {
        self.start_arc = start_arc;
    }
}

pub struct FaceIterator<'a> {
    dcel: &'a Dcel,
    start: ArcId,
    next_arc: Option<ArcId>,
}

impl<'a> Iterator for FaceIterator<'a> {
    type Item = (ArcId, &'a Arc);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(a_id) = self.next_arc {
            let arc = self.dcel.arc(a_id);
            let next = arc.next();
            let item = Some((a_id, arc));

            if next != self.start {
                self.next_arc = Some(next);
            } else {
                self.next_arc = None;
            }
            return item;
        }
        None
    }
}

impl<'a> FaceIterator<'a> {
    pub fn new(dcel: &'a Dcel, start: ArcId) -> Self {
        let next_arc = Some(start);

        Self {
            dcel,
            start,
            next_arc,
        }
    }
}
