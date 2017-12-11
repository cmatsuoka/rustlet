
use std::char;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[allow(dead_code)] pub const SMUSH_EQUAL    : u32 = 1;
#[allow(dead_code)] pub const SMUSH_UNDERLINE: u32 = 2;
#[allow(dead_code)] pub const SMUSH_HIERARCHY: u32 = 4;
#[allow(dead_code)] pub const SMUSH_PAIR     : u32 = 8;
#[allow(dead_code)] pub const SMUSH_BIGX     : u32 = 16;
#[allow(dead_code)] pub const SMUSH_HARDBLANK: u32 = 32;
#[allow(dead_code)] pub const SMUSH_KERN     : u32 = 64;
#[allow(dead_code)] pub const SMUSH_ENABLE   : u32 = 128;

const ERR_INVALID: &'static str = "invalid font file";


#[derive(Debug, Default)]
pub struct FIGfont {
    version       : char,     // font standard version (currently 'a')
    pub hardblank : char,     // sub-character used to represent hardblanks
    pub height    : usize,
    baseline      : usize,    // number of lines from the baseline of a FIGcharacter
    max_length    : usize,    // maximum length of any line describing a FIGcharacter
    pub old_layout: i32,
    comment_lines : usize,    // number of comment lines at the start of the file
    right_to_left : bool,
    pub layout    : u32,
    count         : u32,      // number of code-tagged FIGcharacters in this FIGfont
    chars         : HashMap<char, FIGchar>, // actual FIGcharacter definitions for this font
}

impl FIGfont {
    pub fn new() -> Self {
        let mut font: FIGfont = Default::default();
        font.chars = HashMap::new();
        font
    }

    pub fn get(&self, ch: char) -> &FIGchar {
        match self.chars.get(&ch) {
            Some(k) => k,
            None    => self.get(' '),
        }
    } 

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<&Self, Box<Error>> {

        let file = try!(File::open(path));
        let mut f = BufReader::new(&file);

        let mut line = String::with_capacity(200);

        try!(f.read_line(&mut line));
        try!(self.parse_header(&line));

        // Skip comment lines
        for _ in 0..self.comment_lines {
            line.clear();
            try!(f.read_line(&mut line));
        }

        // Load the 95 required characters
        for i in 32..127{
            let mut c = FIGchar::new();
            try!(c.load(&mut f, self.height));
            self.chars.insert(char::from_u32(i).unwrap(), c);
        }

        // Load code-tagged characters
        // TODO

        Ok(self)
    }

    pub fn parse_header(&mut self, line: &String) -> Result<&Self, Box<Error>> {

        if !line.starts_with("flf2") && !line.starts_with("tlf2") {
            return Err(From::from(ERR_INVALID.to_string()));
        }

        let parms = line.split_whitespace().collect::<Vec<&str>>();

        if parms[0].len() < 6 {
            return Err(From::from(ERR_INVALID.to_string()));
        }

        self.version       = parms[0].chars().nth(4).unwrap();
        self.hardblank     = parms[0].chars().nth(5).unwrap();
        self.height        = try!(parms[1].parse());
        self.baseline      = try!(parms[2].parse());
        self.max_length    = try!(parms[3].parse());
        self.old_layout    = try!(parms[4].parse());
        self.comment_lines = try!(parms[5].parse());
        self.right_to_left = parms[6] == "1";
        self.layout        = try!(parms[7].parse());
        self.count         = try!(parms[8].parse());

        Ok(self)
    }
}


#[derive(Debug)]
pub struct FIGchar {
    pub lines: Vec<String>,
}

impl FIGchar {
    fn new() -> Self {
        FIGchar{
            lines: Vec::new(),
        }
    }

    fn load<R: BufRead>(&mut self, f: &mut R, height: usize) -> Result<&Self, Box<Error>> {
        let mut line = String::new();
        for _ in 0..height {
            line.clear();
            try!(f.read_line(&mut line));
            line = line.trim_right().to_string();
            if line.len() < 1 {
                return Err(From::from("invalid character length"));
            }
            let mark = line.pop().unwrap();
            self.lines.push(line.trim_right_matches(mark).to_string());
        }

        Ok(self)
    }
}

impl fmt::Display for FIGchar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();
        for l in &self.lines {
            s += l;
            s += "\n";
        }
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        let mut f = FIGchar::new();
        f.lines = vec![ "1".to_string(), " 2".to_string(), "  3".to_string() ];
        assert_eq!(format!("{}", f), "1\n 2\n  3\n");
    }
}
