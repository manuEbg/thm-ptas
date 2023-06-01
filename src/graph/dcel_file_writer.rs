use super::dcel::*;
use super::dual_graph::DualGraph;
use super::Dcel;
use std::fs::File;
use std::io::prelude::*;

struct Value<'a> {
    name: &'a str,
    value: &'a dyn WebFileWriter,
}

impl<'a> Value<'a> {
    fn new(name: &'a str, value: &'a dyn WebFileWriter) -> Self {
        Value { name, value }
    }
}

struct Values<'a> {
    values: Vec<Value<'a>>,
}

struct Array<'a, T: WebFileWriter> {
    vec: &'a Vec<T>,
}

struct Object<'a, T: WebFileWriter> {
    item: &'a T,
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
    fn write_to_file(&self, file: &mut File, _id: usize, level: u32) -> std::io::Result<()> {
        write!(*file, "{}", self)
    }
}

impl WebFileWriter for &str {
    fn write_to_file(&self, file: &mut File, _id: usize, level: u32) -> std::io::Result<()> {
        write!(*file, "\"{}\"", self)
    }
}

impl<'a, T: WebFileWriter> WebFileWriter for Array<'a, T> {
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

impl<'a, T: WebFileWriter> WebFileWriter for Object<'a, T> {
    fn write_to_file(&self, file: &mut File, id: usize, level: u32) -> std::io::Result<()> {
        write!(*file, "{{\n")?;
        self.item.write_to_file(file, id, level+1)?;
        write!(*file, "\n")?;
        self.tab(file, level)?;
        write!(*file, "}}")
    }
}

impl<'a> WebFileWriter for Value<'a> {
    fn write_to_file(&self, file: &mut File, id: usize, level: u32) -> std::io::Result<()> {
        self.tab(file, level)?;
        write!(file, "\"{}\": ", self.name)?;
        self.value.write_to_file(file, id, level)
    }
}

impl<'a> WebFileWriter for Values<'a> {
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

impl WebFileWriter for Vertex {
    fn write_to_file(&self, file: &mut File, id: usize, level: u32) -> std::io::Result<()> {
        self.tab(file, level)?;
        Object {
            item: &Value::new(
                "data",
                &Object {
                    item: &Value {
                        name: "id",
                        value: &id,
                    },
                },
            ),
        }
        .write_to_file(file, id, level)?;
        Ok(())
    }
}

impl WebFileWriter for Arc {
    fn write_to_file(&self, file: &mut File, id: usize, level: u32) -> std::io::Result<()> {
        self.tab(file, level)?;
        Object {
            item: &Value::new(
                "data",
                &Object {
                    item: &Values {
                        values: vec![
                            Value::new("id", &id),
                            Value::new("source", &self.get_src()),
                            Value::new("target", &self.get_dst()),
                        ],
                    },
                },
            ),
        }
        .write_to_file(file, id, level)
    }
}

impl<'a> WebFileWriter for SpanningTree<'a> {
    fn write_to_file(&self, file: &mut File, id: usize, level: u32) -> std::io::Result<()> {
        self.tab(file, level)?;
        Value::new(
            "spantree",
            &Array {
                vec: self.get_arcs(),
            },
        )
        .write_to_file(file, id, level)
    }
}

// impl<'a> WebFileWriter for DualGraph<'a> {
//     fn write_to_file(&self, file: &mut File, _id: usize, level: u32) -> std::io::Result<()> {
//         self.tab(file, level)?;
//         Value::new("dualgraph", Object{ item: Values{ vec: &vec![
//             Value("vertices", self )
//         ]}})
//     }
// }

impl WebFileWriter for Dcel {
    fn write_to_file(&self, file: &mut File, id: usize, level: u32) -> std::io::Result<()> {
        let v = self.get_vertices();
        let st = self.spanning_tree(0);
        let s = st.get_arcs();
        let a = self.get_arcs();
        let f = self.get_faces();
        let faces: Vec<Vec<usize>> = f.iter().map(|face| face.walk_face(self)).collect();
        let ff = faces.iter().map(|f| Array { vec: &f }).collect();
        file.write_all(b"let data = ")?;
        Object {
            item: &Values {
                values: vec![
                    Value::new("vertices", &Array { vec: v }),
                    Value::new("arcs", &Array { vec: a }),
                    Value::new("faces", &Array { vec: &ff }),
                    Value::new("spantree", &Array { vec: &s }),
                ],
            },
        }
        .write_to_file(file, id, level)
    }
}

pub struct DcelWriter<'a> {
    file: File,
    dcel: &'a Dcel,
}

impl<'a> DcelWriter<'a> {
    pub fn new(filename: &str, dcel: &'a Dcel) -> Self {
        let file_result = File::create(filename);

        let file = match file_result {
            Ok(file) => file,
            Err(error) => panic!("Problem opening the file: {:?}", error),
        };

        DcelWriter { file, dcel }
    }

    pub fn write_dcel(&mut self) {
        self.dcel.write_to_file(&mut self.file, 0, 0).unwrap();
    }
}
