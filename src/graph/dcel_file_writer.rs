use std::fs::File;
use std::io::prelude::*;
use super::Dcel;
use super::dcel::*;

pub trait WebFileWriter {
    fn write_to_file(&self, file: &mut File, id: usize) -> std::io::Result<()>;
}


impl WebFileWriter for Vertex {
    fn write_to_file(&self, file: &mut File, id: usize) -> std::io::Result<()> {
        write!(file,"\t\t{{\"data\": {{ \"id\": \"{}\"}} }}",id)?;
        Ok(())
    }
}

impl WebFileWriter for Arc {
    fn write_to_file(&self, file: &mut File, id: usize) -> std::io::Result<()> {
        write!(*file,"\t\t{{\"data\": {{ \"id\": \"a{}\", \"source\": {}, \"target\": {} }} }}",id, self.src() , self.dst())
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
        self.append_vertices(self.dcel.num_vertices()).unwrap();
        self.append_arcs(self.dcel).unwrap();
        self.end().unwrap();
    }

    fn beginning(&mut self) -> std::io::Result<()> {
        self.file.write_all(b"let data = {\n")?;
        Ok(())
    }

    fn append_vertices(&mut self, n : usize) -> std::io::Result<()>{
        
        write!(self.file,"\t\"vertices\": [\n")?;
        for i in 0..n {
            self.dcel.get_vertex(i).write_to_file(&mut self.file, i)?;
            if i < n-1 {
                write!(self.file, ",\n")?;
            } else {
                write!(self.file, "\n")?;
            }
        }
        
        write!(self.file, "],\n")?;
        
        Ok(())
    }

    fn append_arcs(&mut self, dcel : &Dcel) -> std::io::Result<()>{
        write!(self.file, "\t\"arcs\": [\n")?;
        let mut i = 0;
        for a in dcel.get_arcs() {
            i += 1;
            a.write_to_file(&mut self.file, i)?;
            if i < dcel.num_arcs() * 2 {
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

