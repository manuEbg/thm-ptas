use std::fs::File;
use std::io::prelude::*;
use super::Dcel;
use super::dcel::*;

pub trait WebFileWriter {
    fn write_to_file(&self, file: &mut File, id: usize, dcel: &Dcel) -> std::io::Result<()>;
}


impl WebFileWriter for Vertex {
    fn write_to_file(&self, file: &mut File, id: usize, dcel: &Dcel) -> std::io::Result<()> {
        write!(file,"\t\t{{\"data\": {{ \"id\": \"{}\"}} }}",id)?;
        Ok(())
    }
}

impl WebFileWriter for Arc {
    fn write_to_file(&self, file: &mut File, id: usize, dcel: &Dcel) -> std::io::Result<()> {
        write!(*file,"\t\t{{\"data\": {{ \"id\": \"a{}\", \"source\": {}, \"target\": {} }} }}",id, self.src() , self.dst())
    }
}

impl WebFileWriter for Face {
    fn write_to_file(&self, file: &mut File, id: usize, dcel: &Dcel) -> std::io::Result<()> {
        write!(*file, "\t\t{{\"face\": [")?;
        let arcs = self.walk_face(dcel); 
        let mut i = 0;
        for a in &arcs {
            write!(*file, "\"{}\"", *a)?;
            i += 1;
            if i < arcs.len(){
                write!(*file,",")?;
            }
        }
        write!(*file, "] }}")
    }
}


pub struct DcelWriter<'a> {
    file : File,
    dcel: &'a Dcel
}

impl<'a> DcelWriter<'a> {
    pub fn new(filename : &str, dcel: &'a Dcel) -> Self {
        let file_result = File::create(filename);
        
        let file = match file_result {
            Ok(file) => file,
            Err(error) => panic!("Problem opening the file: {:?}", error),
        };

        DcelWriter {
            file,
            dcel
        }
    }

    pub fn write_dcel(&mut self) {
        self.beginning().unwrap();
        self.append_vertices().unwrap();
        self.append_arcs().unwrap();
        self.append_faces().unwrap();
        self.end().unwrap();
    }

    fn beginning(&mut self) -> std::io::Result<()> {
        self.file.write_all(b"let data = {\n")?;
        Ok(())
    }

    fn append_faces(&mut self) -> std::io::Result<()> {
        write!(self.file, "\t\"faces\": [\n")?;
        let mut i = 0;
        for f in self.dcel.get_faces() {
            f.write_to_file(&mut self.file, i, self.dcel)?;
            i += 1; 
            if i < self.dcel.num_faces() {
                write!(self.file, ",\n")?;
            } else {
                write!(self.file, "\n")?;
            }
        }
        write!(self.file, "],\n")
    }
    
    fn append_vertices(&mut self) -> std::io::Result<()>{
        
        write!(self.file,"\t\"vertices\": [\n")?;
        for i in 0..self.dcel.num_vertices() {
            self.dcel.get_vertex(i).write_to_file(&mut self.file, i, self.dcel)?;
            if i < self.dcel.num_vertices()-1 {
                write!(self.file, ",\n")?;
            } else {
                write!(self.file, "\n")?;
            }
        }
        
        write!(self.file, "],\n")?;
        
        Ok(())
    }

    fn append_arcs(&mut self) -> std::io::Result<()>{
        write!(self.file, "\t\"arcs\": [\n")?;
        let mut i = 0;
        for a in self.dcel.get_arcs() {
            a.write_to_file(&mut self.file, i,self.dcel)?;
            i += 1;
            if i < self.dcel.num_arcs() {
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

