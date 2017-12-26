use std::char;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use super::Error;

#[allow(dead_code)] pub const SMUSH_EQUAL    : u32 = 1;
#[allow(dead_code)] pub const SMUSH_UNDERLINE: u32 = 2;
#[allow(dead_code)] pub const SMUSH_HIERARCHY: u32 = 4;
#[allow(dead_code)] pub const SMUSH_PAIR     : u32 = 8;
#[allow(dead_code)] pub const SMUSH_BIGX     : u32 = 16;
#[allow(dead_code)] pub const SMUSH_HARDBLANK: u32 = 32;
#[allow(dead_code)] pub const SMUSH_KERN     : u32 = 64;
#[allow(dead_code)] pub const SMUSH_ENABLE   : u32 = 128;


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
            None    => self.get( if ch == '\t' { ' ' } else { '\0' }),
        }
    } 

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<&Self, Error> {

        let file = try!(File::open(path));
        let mut f = BufReader::new(&file);

        let mut line = String::new();

        try!(f.read_line(&mut line));
        try!(self.parse_header(&line));

        // Skip comment lines
        for _ in 0..self.comment_lines {
            line.clear();
            try!(f.read_line(&mut line));
        }

        // Define default 0-code character
        self.chars.insert('\0', FIGchar::with_lines(self.height));

        // Load required characters
        for i in (32..127).chain(vec![196, 215, 220, 228, 246, 252, 223]) {
            let mut c = FIGchar::new();
            try!(c.load(&mut f, self.height));
            self.chars.insert(char_from_u32(i).unwrap(), c);
        }

        // Load code-tagged characters
        loop {
            line.clear();
            if try!(f.read_line(&mut line)) == 0 {
                break
            }
            let code = match line.split_whitespace().next() {
                Some(val) => val,
                None      => break,
            };

            let mut c = FIGchar::new();
            try!(c.load(&mut f, self.height));
            self.chars.insert(char_from_u32(u32_from_str(code)?)?, c);
        }

        Ok(self)
    }

    pub fn parse_header(&mut self, line: &String) -> Result<&Self, Error> {

        if !line.starts_with("flf2") && !line.starts_with("tlf2") {
            return Err(Error::FontFormat("unsupported font format"));
        }

        let parms = line.split_whitespace().collect::<Vec<&str>>();

        if parms[0].len() < 6 {
            return Err(Error::FontFormat("unsupported font format"));
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

fn char_from_u32(num: u32) -> Result<char, Error> {
    match char::from_u32(num) {
        Some(c) => Ok(c),
        None    => Err(Error::CodeTag(num)),
    }
}

// See https://github.com/rust-lang/rfcs/issues/1098
fn u32_from_str(s: &str) -> Result<u32, Error> {
    let mut s = s.trim();
    let mut radix = 10;

    // return an unused character for translation tables
    if s.starts_with("-") {
        return Ok(1);
    }

    if s.starts_with("0x") || s.starts_with("0X") {
        radix = 16;
        s = &s[2..];
    }

    Ok(u32::from_str_radix(s, radix)?)
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

    fn with_lines(num: usize) -> Self {
        let mut c = Self::new();
        (0..num).for_each(|_| c.lines.push("".to_string()));
        c
    }

    fn load<R: BufRead>(&mut self, f: &mut R, height: usize) -> Result<&Self, Error> {
        let mut line = String::new();
        for _ in 0..height {
            line.clear();
            try!(f.read_line(&mut line));
            line = line.trim_right().to_string();
            if line.len() < 1 {
                return Err(Error::FontFormat("invalid character length"));
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

    #[test]
    fn test_char_from_u32() {
        assert_eq!(char_from_u32(0x0041).unwrap(), 'A');
        assert_eq!(char_from_u32(0x00C1).unwrap(), 'Á');
    }

    #[test]
    fn test_u32_from_str() {
        assert!(matches!(u32_from_str("0x0041"), Ok(0x41)));
        assert!(matches!(u32_from_str("0x00C1"), Ok(0xC1)));
        assert!(matches!(u32_from_str("  0x41"), Ok(0x41)));
        assert!(matches!(u32_from_str("0X0041"), Ok(0x41)));
        assert!(matches!(u32_from_str("-0x100"), Ok(1)));
        assert!(matches!(u32_from_str("-5"), Ok(1)));
        assert!(u32_from_str("foobar").is_err());
        assert!(u32_from_str("").is_err());
    }
}
