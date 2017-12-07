use std::error::Error;
use super::Smusher;

pub struct Wrapper<'a> {
    sm       : &'a mut Smusher<'a>,
    buffer   : String,
    pub width: usize,
}

impl<'a> Wrapper<'a> {
    pub fn using(sm: &'a mut Smusher<'a>, width: usize) -> Self {
        Wrapper{
           sm,
           width,
           buffer: String::new(),
        }
    }

    pub fn clear(&mut self) {
        self.sm.clear();
        self.buffer.clear();
    }

    pub fn get(self) -> Vec<String> {
        self.sm.get()
    }

    pub fn push_str(&mut self, s: &str) -> Result<(), Box<Error>> {
        try!(self.sm.push(' '));
        try!(self.sm.push_str(s));

        if self.sm.len() > self.width {
            let b = self.buffer.clone();
            try!(self.sm.push_str(&b));
            self.sm.clear();
            return Err(From::from("line full".to_string()))
        }

        self.buffer.push(' ');
        self.buffer.push_str(s);
        Ok(())
    }

    pub fn push(&mut self, ch: char) -> Result<(), Box<Error>> {
        try!(self.sm.push(ch));

        if self.sm.len() > self.width {
            let b = self.buffer.clone();
            try!(self.sm.push_str(&b));
            self.sm.clear();
            return Err(From::from("line full".to_string()))
        }

        self.buffer.push(ch);
        Ok(())
    }
}
