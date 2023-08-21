use super::approximated_td::ApproximatedTD;
use super::approximated_td::TDBuilder;
use super::dcel::spanning_tree::SpanningTree;
use super::dcel::vertex::VertexId;
use super::sub_dcel::SubDcel;
use super::Dcel;
use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;

#[derive(Clone)]
struct JsValue<'a> {
    name: &'a str,
    value: &'a dyn WebFileWriter,
}

impl<'a> JsValue<'a> {
    fn new(name: &'a str, value: &'a dyn WebFileWriter) -> Self {
        JsValue { name, value }
    }
}

struct JsValues<'a> {
    values: Vec<JsValue<'a>>,
}

impl<'a> JsValues<'a> {
    fn new(values: Vec<JsValue<'a>>) -> Self {
        Self { values }
    }
}

struct JsArray<'a, T: WebFileWriter> {
    vec: &'a Vec<T>,
}

impl<'a, T: WebFileWriter> JsArray<'a, T> {
    pub fn new(vec: &'a Vec<T>) -> Self {
        Self { vec }
    }
}

struct JsFace {
    id: usize,
    arcs: Vec<usize>,
    vertices: Vec<usize>,
}

impl JsFace {
    fn new(id: usize, arcs: Vec<usize>, vertices: Vec<usize>) -> Self {
        Self { id, arcs, vertices }
    }
}

struct JsVertex {
    id: usize,
}

impl JsVertex {
    pub fn new(id: usize) -> Self {
        Self { id }
    }
}

struct JsArc {
    id: usize,
    src: usize,
    dst: usize,
    is_added: bool,
    is_invalid: bool,
}

impl JsArc {
    fn new(id: usize, src: usize, dst: usize, is_added: bool, is_invalid: bool) -> Self {
        Self {
            id,
            src,
            dst,
            is_added,
            is_invalid,
        }
    }
}

struct JsObject<'a, T: WebFileWriter> {
    item: &'a T,
}

impl<'a, T: WebFileWriter> JsObject<'a, T> {
    fn new(item: &'a T) -> Self {
        Self { item }
    }
}

pub trait WebFileWriter {
    fn write_to_file(&self, file: &mut File, id: usize, level: u32) -> std::io::Result<()>;

    fn tab(&self, file: &mut File, tabs: u32) -> std::io::Result<()> {
        for _ in 0..tabs {
            write!(*file, "\t")?;
        }
        Ok(())
    }
}

impl WebFileWriter for bool {
    fn write_to_file(&self, file: &mut File, id: usize, level: u32) -> std::io::Result<()> {
        write!(*file, "{}", self)
    }
}

impl WebFileWriter for usize {
    fn write_to_file(&self, file: &mut File, _id: usize, _level: u32) -> std::io::Result<()> {
        write!(*file, "{}", self)
    }
}

impl WebFileWriter for &str {
    fn write_to_file(&self, file: &mut File, _id: usize, _level: u32) -> std::io::Result<()> {
        write!(*file, "\"{}\"", self)
    }
}

impl<'a, T: WebFileWriter> WebFileWriter for JsArray<'a, T> {
    fn write_to_file(&self, file: &mut File, _id: usize, level: u32) -> std::io::Result<()> {
        self.tab(file, level)?;
        write!(*file, "[")?;
        for (idx, item) in self.vec.iter().enumerate() {
            item.write_to_file(file, idx, level)?;

            if idx == self.vec.len() - 1 {
                break;
            }

            write!(*file, ", ")?;
        }
        write!(*file, "]")
    }
}

impl<'a, T: WebFileWriter> WebFileWriter for JsObject<'a, T> {
    fn write_to_file(&self, file: &mut File, id: usize, level: u32) -> std::io::Result<()> {
        write!(*file, "{{\n")?;
        self.item.write_to_file(file, id, level + 1)?;
        write!(*file, "\n")?;
        self.tab(file, level)?;
        write!(*file, "}}")
    }
}

impl<'a> WebFileWriter for JsValue<'a> {
    fn write_to_file(&self, file: &mut File, id: usize, level: u32) -> std::io::Result<()> {
        self.tab(file, level)?;
        write!(file, "\"{}\": ", self.name)?;
        self.value.write_to_file(file, id, level)
    }
}

impl<'a> WebFileWriter for JsValues<'a> {
    fn write_to_file(&self, file: &mut File, id: usize, level: u32) -> std::io::Result<()> {
        for (i, v) in self.values.iter().enumerate() {
            v.write_to_file(file, id, level)?;

            if i == self.values.len() - 1 {
                break;
            }

            write!(*file, ",\n")?;
        }
        Ok(())
    }
}

impl WebFileWriter for JsVertex {
    fn write_to_file(&self, file: &mut File, id: usize, level: u32) -> std::io::Result<()> {
        self.tab(file, level)?;
        JsObject {
            item: &JsValue::new("id", &self.id),
        }
        .write_to_file(file, id, level)?;
        Ok(())
    }
}

impl WebFileWriter for JsFace {
    fn write_to_file(&self, file: &mut File, id: usize, level: u32) -> std::io::Result<()> {
        self.tab(file, level)?;
        JsObject::new(&JsValues::new(vec![
            JsValue::new("id", &self.id),
            JsValue::new("arcs", &JsArray::new(&self.arcs)),
            JsValue::new("vertices", &JsArray::new(&self.vertices)),
        ]))
        .write_to_file(file, id, level)
    }
}

impl WebFileWriter for JsArc {
    fn write_to_file(&self, file: &mut File, id: usize, level: u32) -> std::io::Result<()> {
        if self.is_invalid {
            return Ok(());
        }
        self.tab(file, level)?;
        JsObject {
            item: &JsValues {
                values: vec![
                    JsValue::new("id", &id),
                    JsValue::new("source", &self.src),
                    JsValue::new("target", &self.dst),
                    JsValue::new("is_added", &self.is_added),
                ],
            },
        }
        .write_to_file(file, id, level)
    }
}

impl<'a> WebFileWriter for SpanningTree<'a> {
    fn write_to_file(&self, file: &mut File, id: usize, level: u32) -> std::io::Result<()> {
        self.tab(file, level)?;
        JsValue::new("spantree", &JsArray::new(self.arcs())).write_to_file(file, id, level)
    }
}

impl WebFileWriter for SubDcel {
    fn write_to_file(&self, file: &mut File, id: usize, level: u32) -> std::io::Result<()> {
        let mapped_arcs = self
            .get_untriangulated_arcs()
            .iter()
            .enumerate()
            .map(|(i, _)| *self.get_original_arc(i).unwrap())
            .collect::<Vec<_>>();

        let triangulated_arcs = self
            .get_triangulated_arcs()
            .iter()
            .map(|arc| {
                let original_dst = self.get_original_vertex(arc.dst()).unwrap();
                let original_src = self.get_original_vertex(arc.src()).unwrap();

                JsValues::new(vec![
                    JsValue::new("dst", original_dst),
                    JsValue::new("src", original_src),
                ])
            })
            .collect::<Vec<_>>();

        let objs = triangulated_arcs
            .iter()
            .map(|arcs| JsObject::new(arcs))
            .collect();

        JsObject::new(&JsValues::new(vec![
            JsValue::new("arcs", &JsArray::new(&mapped_arcs)),
            //JsValue::new("triangulated_arcs", &JsArray::new(&triangulated_arcs)),
            JsValue::new("triangulated_arcs", &JsArray::new(&objs)),
        ]))
        .write_to_file(file, id, level)
    }
}

impl<'a> WebFileWriter for HashSet<VertexId> {
    fn write_to_file(&self, file: &mut File, id: usize, level: u32) -> std::io::Result<()> {
        JsArray::new(
            &self
                .iter()
                .collect::<Vec<&VertexId>>()
                .iter()
                .map(|m| JsVertex::new(**m))
                .collect::<Vec<JsVertex>>(),
        )
        .write_to_file(file, id, level)
    }
}

impl<'a> WebFileWriter for ApproximatedTD<'a> {
    fn write_to_file(&self, file: &mut File, id: usize, level: u32) -> std::io::Result<()> {
        fn build_js_verticies(n: usize) -> Vec<JsVertex> {
            (0..n).map(|id| JsVertex::new(id)).collect()
        }
        fn build_js_arcs(dg: &ApproximatedTD) -> Vec<JsArc> {
            let mut arc_count: usize = 0;
            let mut arcs = vec![];

            for (v, adj) in dg.adjacent().iter().enumerate() {
                for u in adj.iter() {
                    arcs.push(JsArc::new(arc_count, v, *u, false, false));
                    arc_count += 1;
                }
            }
            arcs
        }

        JsObject::new(&JsValues::new(vec![
            JsValue::new(
                "vertices",
                &JsArray::new(&build_js_verticies(self.num_bags())),
            ),
            JsValue::new("arcs", &JsArray::new(&build_js_arcs(self))),
            JsValue::new("bags", &JsArray::new(self.bags())),
        ]))
        .write_to_file(file, id, level)
    }
}

impl WebFileWriter for Dcel {
    fn write_to_file(&self, file: &mut File, id: usize, level: u32) -> std::io::Result<()> {
        let v = self
            .vertices()
            .iter()
            .enumerate()
            .map(|(i, _v)| JsVertex::new(i))
            .collect();
        let st = self.spanning_tree(0);
        let mut b = TDBuilder::new(&st);
        let approx_td = ApproximatedTD::from(&mut b);
        let s = st.arcs();
        let a = self
            .arcs()
            .iter()
            .enumerate()
            .map(|(i, a)| {
                JsArc::new(
                    i,
                    a.src(),
                    a.dst(),
                    i >= self.pre_triangulation_arc_count(),
                    self.invalid_arcs[i],
                )
            })
            .collect();
        let faces = self.faces();
        let arcs_per_faces: Vec<Vec<usize>> =
            faces.iter().map(|face| face.walk_face(self)).collect();
        let verts_per_face: Vec<Vec<usize>> = arcs_per_faces
            .iter()
            .map(|arcs| arcs.iter().map(|arc| self.arc(*arc).src()).collect())
            .collect();
        let mut js_faces = vec![];
        for (i, _) in faces.iter().enumerate() {
            js_faces.push(JsFace::new(
                i,
                arcs_per_faces[i].clone(),
                verts_per_face[i].clone(),
            ));
        }

        let rings = &self.find_rings().unwrap();
        let ring_array = rings
            .iter()
            .map(|ring| {
                ring.sub
                    .arcs()
                    .iter()
                    .enumerate()
                    .map(|(i, _)| *ring.get_original_arc(i).unwrap())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let donuts = &self.find_donuts_for_k(1, 1, &st).unwrap();

        file.write_all(b"let data = ")?;
        JsObject {
            item: &JsValues {
                values: vec![
                    JsValue::new("vertices", &JsArray::new(&v)),
                    JsValue::new("arcs", &JsArray::new(&a)),
                    JsValue::new("faces", &JsArray::new(&js_faces)),
                    JsValue::new("spantree", &JsArray::new(&s)),
                    JsValue::new("dualgraph", &approx_td), // TODO rename JS entry
                    JsValue::new(
                        "rings",
                        &JsArray::new(&ring_array.iter().map(|ring| JsArray::new(&ring)).collect()),
                    ),
                    JsValue::new("donuts", &JsArray::new(&donuts)),
                ],
            },
        }
        .write_to_file(file, id, level)
    }
}

pub struct JsDataWriter<'a> {
    file: File,
    dcel: &'a Dcel,
}

impl<'a> JsDataWriter<'a> {
    pub fn new(filename: &str, dcel: &'a Dcel) -> Self {
        let file_result = File::create(filename);

        let file = match file_result {
            Ok(file) => file,
            Err(error) => panic!("Problem opening the file: {:?}", error),
        };

        JsDataWriter { file, dcel }
    }

    pub fn write_data(&mut self) {
        self.dcel.write_to_file(&mut self.file, 0, 0).unwrap();
    }
}
