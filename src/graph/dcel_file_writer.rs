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
    fn write_to_file(&self, file: &mut File, id: usize, dcel: &Dcel) -> std::io::Result<()>;
}

impl WebFileWriter for usize {
    fn write_to_file(&self, file: &mut File, id: usize, dcel: &Dcel) -> std::io::Result<()> {
        write!(*file, "{}", self)
    }
}

impl WebFileWriter for &str {
    fn write_to_file(&self, file: &mut File, id: usize, dcel: &Dcel) -> std::io::Result<()> {
        write!(*file, "\"{}\"", self)
    }
}

impl<'a, T: WebFileWriter> WebFileWriter for Array<'a, T> {
    fn write_to_file(&self, file: &mut File, id: usize, dcel: &Dcel) -> std::io::Result<()> {
        write!(*file, "[")?;
        for (idx, item) in self.vec.iter().enumerate() {
            item.write_to_file(file, idx, dcel)?;

            if idx == self.vec.len() - 1 {
                break;
            }

            write!(*file, ", ")?;
        }
        write!(*file, "]")
    }
}

impl<'a, T: WebFileWriter> WebFileWriter for Object<'a, T> {
    fn write_to_file(&self, file: &mut File, id: usize, dcel: &Dcel) -> std::io::Result<()> {
        write!(*file, "{{\n")?;
        self.item.write_to_file(file, id, dcel)?;
        write!(*file, "\n}}")
    }
}

impl<'a> WebFileWriter for Value<'a> {
    fn write_to_file(&self, file: &mut File, id: usize, dcel: &Dcel) -> std::io::Result<()> {
        write!(file, "\"{}\": ", self.name)?;
        self.value.write_to_file(file, id, dcel)
    }
}

impl<'a> WebFileWriter for Values<'a> {
    fn write_to_file(&self, file: &mut File, id: usize, dcel: &Dcel) -> std::io::Result<()> {
        for (i, v) in self.values.iter().enumerate() {
            v.write_to_file(file, id, dcel)?;

            if i == self.values.len() - 1 {
                break;
            }

            write!(*file, ",\n")?;
        }
        Ok(())
    }
}

impl WebFileWriter for Vertex {
    fn write_to_file(&self, file: &mut File, id: usize, dcel: &Dcel) -> std::io::Result<()> {
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
        .write_to_file(file, id, dcel)?;
        Ok(())
    }
}

impl WebFileWriter for Arc {
    fn write_to_file(&self, file: &mut File, id: usize, dcel: &Dcel) -> std::io::Result<()> {
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
        .write_to_file(file, id, dcel)
    }
}

impl WebFileWriter for Face {
    fn write_to_file(&self, file: &mut File, id: usize, dcel: &Dcel) -> std::io::Result<()> {
        let arcs = self.walk_face(dcel);
        Array { vec: &arcs }.write_to_file(file, id, dcel)
    }
}

impl<'a> WebFileWriter for SpanningTree<'a> {
    fn write_to_file(&self, file: &mut File, id: usize, dcel: &Dcel) -> std::io::Result<()> {
        Value::new(
            "spantree",
            &Array {
                vec: self.get_arcs(),
            },
        )
        .write_to_file(file, id, dcel)
    }
}

impl<'a> WebFileWriter for DualGraph<'a> {
    fn write_to_file(&self, file: &mut File, id: usize, dcel: &Dcel) -> std::io::Result<()> {
        write!(*file, "\t\"dual_graph\": {{ \n\t\t\"")
    }
}

impl WebFileWriter for Dcel {
    fn write_to_file(&self, file: &mut File, id: usize, dcel: &Dcel) -> std::io::Result<()> {
        let v = self.get_vertices();
        let st = self.spanning_tree(0);
        let s = st.get_arcs();
        let a = self.get_arcs();
        let f = self.get_faces();
        file.write_all(b"let data = ")?;
            Object {
                item: &Values {
                    values: vec![
                        Value::new("vertices", &Array { vec: v }),
                        Value::new("arcs", &Array { vec: a }),
                        Value::new("faces", &Array { vec: &f }),
                        Value::new("spantree", &Array { vec: &s }),
                    ],
                },
            }
        .write_to_file(file, id, dcel)
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
        self.dcel
            .write_to_file(&mut self.file, 0, self.dcel)
            .unwrap();
        // self.beginning().unwrap();
        //
        // self.append_vertices().unwrap();
        // self.append_arcs().unwrap();
        // self.append_faces().unwrap();
        // self.dcel
        //     .spanning_tree(0)
        //     .write_to_file(&mut self.file, 0, self.dcel)
        //     .unwrap();
        // self.end().unwrap();
    }

    fn beginning(&mut self) -> std::io::Result<()> {
        self.file.write_all(b"let data = {\n")?;
        Ok(())
    }

    fn append_faces(&mut self) -> std::io::Result<()> {
        write!(self.file, "\t\"faces\": [\n")?;
        Value::new(
            "faces",
            &Array {
                vec: self.dcel.get_faces(),
            },
        )
        .write_to_file(&mut self.file, 0, self.dcel)
        // for (i, f) in self.dcel.get_faces().iter().enumerate() {
        //     f.write_to_file(&mut self.file, i, self.dcel)?;
        //     if i < self.dcel.num_faces() - 1 {
        //         write!(self.file, ",\n")?;
        //     } else {
        //         write!(self.file, "\n")?;
        //     }
        // }
        // write!(self.file, "\t],\n")
    }

    fn append_vertices(&mut self) -> std::io::Result<()> {
        write!(self.file, "\t\"vertices\": [\n")?;
        for i in 0..self.dcel.num_vertices() {
            Object {
                item: self.dcel.get_vertex(i),
            }
            .write_to_file(&mut self.file, i, self.dcel)?;
            if i < self.dcel.num_vertices() - 1 {
                write!(self.file, ",\n")?;
            } else {
                write!(self.file, "\n")?;
            }
        }

        write!(self.file, "\t],\n")?;

        Ok(())
    }

    fn append_arcs(&mut self) -> std::io::Result<()> {
        write!(self.file, "\t\"arcs\": [\n")?;
        for (i, a) in self.dcel.get_arcs().iter().enumerate() {
            Object { item: a }.write_to_file(&mut self.file, i, self.dcel)?;
            if i < self.dcel.num_arcs() - 1 {
                write!(self.file, ",\n")?;
            } else {
                write!(self.file, "\n")?;
            }
        }
        write!(self.file, "\t],\n")?;
        Ok(())
    }

    fn end(&mut self) -> std::io::Result<()> {
        self.file.write_all(b"}")?;
        Ok(())
    }
}
