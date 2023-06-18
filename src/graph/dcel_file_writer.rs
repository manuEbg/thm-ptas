use super::dcel::*;
use super::dcel::spanning_tree::SpanningTree;
use super::dual_graph::DualGraph;
use super::Dcel;
use std::fs::File;
use std::io::prelude::*;

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
}

impl JsArc {
    pub fn new(id: usize, src: usize, dst: usize) -> Self {
        Self { id, src, dst }
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
        self.tab(file, level)?;
        JsObject {
            item: &JsValues {
                values: vec![
                    JsValue::new("id", &id),
                    JsValue::new("source", &self.src),
                    JsValue::new("target", &self.dst),
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

impl<'a> WebFileWriter for DualGraph<'a> {
    fn write_to_file(&self, file: &mut File, id: usize, level: u32) -> std::io::Result<()> {
        fn build_js_vertecies(n: usize) -> Vec<JsVertex> {
            (0..n).map(|id| JsVertex::new(id)).collect()
        }
        fn build_js_arcs(dg: &DualGraph) -> Vec<JsArc> {
            let mut arc_count: usize = 0;
            let mut arcs = vec![];

            for (v, adj) in dg.get_adjacent().iter().enumerate() {
                for u in adj.iter() {
                    arcs.push(JsArc::new(arc_count, v, *u));
                    arc_count += 1;
                }
            }
            arcs
        }

        JsObject::new(&JsValues::new(vec![
            JsValue::new(
                "vertices",
                &JsArray::new(&build_js_vertecies(self.num_vertices())),
            ),
            JsValue::new("arcs", &JsArray::new(&build_js_arcs(self))),
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
        let mut dual_graph = DualGraph::new(&st);
        dual_graph.build();
        let s = st.arcs();
        let a = self
            .arcs()
            .iter()
            .enumerate()
            .map(|(i, a)| JsArc::new(i, a.src(), a.dst()))
            .collect();
        let faces = self.faces();
        let arcs_per_faces: Vec<Vec<usize>> = faces.iter().map(|face| face.walk_face(self)).collect();
        let verts_per_face: Vec<Vec<usize>> = arcs_per_faces
            .iter()
            .map(|arcs| {
                arcs.iter()
                    .map(|arc| self.arc(*arc).src())
                    .collect()
            })
            .collect();
        let mut js_faces = vec![];
        for (i, _) in faces.iter().enumerate() {
           js_faces.push(JsFace::new(i, arcs_per_faces[i].clone(), verts_per_face[i].clone()));
        }
        file.write_all(b"let data = ")?;
        JsObject {
            item: &JsValues {
                values: vec![
                    JsValue::new("vertices", &JsArray::new(&v)),
                    JsValue::new("arcs", &JsArray::new(&a)),
                    JsValue::new("faces", &JsArray::new(&js_faces)),
                    JsValue::new("spantree", &JsArray::new(&s)),
                    JsValue::new("dualgraph", &dual_graph),
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
