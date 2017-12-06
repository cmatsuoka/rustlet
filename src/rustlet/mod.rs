use std::cmp::min;
use std::error::Error;
use self::figfont::{FIGchar, FIGfont};

pub mod figfont;
pub mod charsmush;


#[derive(Debug)]
pub struct Smusher<'a> {
    prev_width: isize,
    curr_width: isize,
    mode      : u32,
    right2left: bool,
    font      : &'a FIGfont,
    output    : Vec<String>,
}


impl<'a> Smusher<'a> {

    pub fn new(font: &'a FIGfont) -> Self {
        let mut sm = Smusher{
            font,
            prev_width: 0,
            curr_width: 0,
            mode      : font.layout,
            right2left: false,
            output    : Vec::new(),
        };
        for _ in 0..sm.font.height {
            sm.output.push("".to_string());
        }
        sm
    }

    pub fn amount(self, c: FIGchar) -> usize {
        smusher_amount(&self.output, &c, self.font.hardblank, self.mode)
    }

    pub fn print(self) {
        for p in self.output {
            println!("{}", p);
        }
    }

    pub fn add_str(&mut self, s: &str) -> Result<(), Box<Error>> {
        for c in s.chars() {
            try!(self.add_char(&c));
        }

        Ok(())
    }

    pub fn add_char(&mut self, ch: &char) -> Result<(), Box<Error>> {
        let fc = self.font.get(ch);
        self.output = try!(smusher_smush(&self.output, fc, self.font.hardblank, self.mode));
        Ok(())
    }

    fn smush_char(self, l: char, r: char) -> Option<char> {

        //cmp_return_other!(' ', l, r);

/*
        // Disallows overlapping if previous character or current character has a
        // width of 1 or zero
        if self.prev_width < 2 || self.curr_width < 2 {
            return None
        }
*/
        charsmush::smush(l, r, self.font.hardblank, self.right2left, self.mode)
    }
}

fn smusher_amount(output: &Vec<String>, c: &FIGchar, hardblank: char, mode: u32) -> usize {
    let mut amt = 9999;
    for i in 0..output.len() {
        amt = min(amt, output[i].smush_amount(&c.lines[i], hardblank, mode));
    }
    amt
}

fn smusher_smush(output: &Vec<String>, fc: &FIGchar, hardblank: char, mode: u32) -> Result<Vec<String>, Box<Error>> {

    let amt = smusher_amount(&output, fc, hardblank, mode);
println!("> amt={}", amt);
    let mut res = Vec::new();

    for i in 0..output.len() {
        res.push(smush_str(&output[i], &fc.lines[i], amt, hardblank, false, mode)?);
    }

    Ok(res)
}

trait Smush {
    fn smush_amount(self, &str, char, u32) -> usize;
}

impl<'a> Smush for &'a str {

    // Compute the number of characters a string can be smushed into another string.
    fn smush_amount(self, s: &str, hardblank: char, mode: u32) -> usize {
    
        let a1 = self.len() - match self.rfind(|x| { let y:char = x; !y.is_whitespace() }) {
            Some(val) => val + 1,
            None      => 0,
        };
    
        let a2 = match s.find(|x| { let y:char = x; !y.is_whitespace() }) {
            Some(val) => val,
            None      => s.len(),
        };

println!(">> s1={:?} s2={:?} a1={} a2={}", self, s, a1, a2);
        let amt = a1 + a2;

        // Retrieve character pair and see if they're smushable
        let (l, r) = match smush_chars(self, s, amt + 1) {
            Some(pair) => pair,
            None       => { return amt; }
        };
        match charsmush::smush(l, r, hardblank, false, mode) {
            Some(_) => { amt + 1 },
            None    => { amt },
        }
    }
}

fn smush_chars(s1: &str, s2: &str, amt: usize) -> Option<(char, char)> {
    if s1.len() == 0 || s2.len() == 0 {
        return None;
    }

    let overlap = min(amt, s2.len());

    for i in (0..overlap).rev() {
        if s1.len() + i < amt {
            return None;
        }
        let l = s1.chars().nth(s1.len() + i - amt).unwrap();
        let r = s2.chars().nth(i).unwrap();
        if l != ' ' && r != ' ' {
            return Some((l, r));
        }
    }

    None
}

fn smush_str(s1: &str, s2: &str, mut amt: usize, hardblank: char, right2left: bool,
             mode: u32) -> Result<String, Box<Error>> {

    /*let a2 = match s2.find(|x| { let y:char = x; !y.is_whitespace() }) {
        Some(val) => val,
        None      => s2.len(),
    };

    let s2 = s2.trim_left();
    if a2 > amt {
        amt = 0;
    } else {
        amt -= a2;
    }*/
    let mut limit: usize;
    if amt > s1.len() {
        limit = 1;
    } else {
        limit = s1.len() - amt;
    }
    let mut res = s1[..limit].to_string();
println!("s1={:?} s2={:?} amt={} res={:?}", s1, s2, amt, res);

    let (l, r) = match smush_chars(s1, s2, amt) {
        Some(pair) => pair,
        None       => { res.push_str(s2); return Ok(res) },
    };

    match charsmush::smush(l, r, hardblank, false, mode) {
        Some(c) => { res.push(c); res.push_str(&s2[1..]) },
        None    => res += s2,
    }

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! vec_of_strings {
        ( $($x:expr),* ) => (vec![$($x.to_string()),*])
    }

    #[test]
    fn test_smush_chars() {
        assert_eq!(smush_chars("    ", "    ", 2), None);
        assert_eq!(smush_chars("abc ", " xyz", 1), None);
        assert_eq!(smush_chars("abc ", " xyz", 2), None);
        assert_eq!(smush_chars("abc ", " xyz", 3), Some(('c', 'x')));
        assert_eq!(smush_chars("abc   ", " x", 5), Some(('c', 'x')));
        assert_eq!(smush_chars("a ", " xyzwt", 3), Some(('a', 'x')));
        assert_eq!(smush_chars("a", "      x", 6), None);
        assert_eq!(smush_chars("a", "      x", 7), Some(('a', 'x')));
        assert_eq!(smush_chars("", "       x", 7), None);
    }

    #[test]
    fn test_smush_amount_str() {
        assert_eq!("".smush_amount("", '$', 0xbf), 0);

        assert_eq!("".smush_amount("    ", '$', 0xbf), 4);
        assert_eq!("".smush_amount("   y", '$', 0xbf), 3);

        assert_eq!("    ".smush_amount("    ", '$', 0xbf), 8);
        assert_eq!("x   ".smush_amount("    ", '$', 0xbf), 7);
        assert_eq!("xx  ".smush_amount("    ", '$', 0xbf), 6);
        assert_eq!("xxx ".smush_amount("    ", '$', 0xbf), 5);
        assert_eq!("xxxx".smush_amount("    ", '$', 0xbf), 4);

        assert_eq!("    ".smush_amount("   y", '$', 0xbf), 7);
        assert_eq!("x   ".smush_amount("   y", '$', 0xbf), 6);
        assert_eq!("xx  ".smush_amount("   y", '$', 0xbf), 5);
        assert_eq!("xxx ".smush_amount("   y", '$', 0xbf), 4);
        assert_eq!("xxxx".smush_amount("   y", '$', 0xbf), 3);

        assert_eq!("    ".smush_amount("  yy", '$', 0xbf), 6);
        assert_eq!("x   ".smush_amount("  yy", '$', 0xbf), 5);
        assert_eq!("xx  ".smush_amount("  yy", '$', 0xbf), 4);
        assert_eq!("xxx ".smush_amount("  yy", '$', 0xbf), 3);
        assert_eq!("xxxx".smush_amount("  yy", '$', 0xbf), 2);

        assert_eq!("    ".smush_amount(" yyy", '$', 0xbf), 5);
        assert_eq!("x   ".smush_amount(" yyy", '$', 0xbf), 4);
        assert_eq!("xx  ".smush_amount(" yyy", '$', 0xbf), 3);
        assert_eq!("xxx ".smush_amount(" yyy", '$', 0xbf), 2);
        assert_eq!("xxxx".smush_amount(" yyy", '$', 0xbf), 1);

        assert_eq!("    ".smush_amount("yyyy", '$', 0xbf), 4);
        assert_eq!("x   ".smush_amount("yyyy", '$', 0xbf), 3);
        assert_eq!("xx  ".smush_amount("yyyy", '$', 0xbf), 2);
        assert_eq!("xxx ".smush_amount("yyyy", '$', 0xbf), 1);
        assert_eq!("xxxx".smush_amount("yyyy", '$', 0xbf), 0);

        assert_eq!("x".smush_amount("y", '$', 0xbf), 0);
        assert_eq!("x".smush_amount("x", '$', 0xbf), 1);     // rule 1
        assert_eq!("<".smush_amount(">", '$', 0xbf), 0);
        assert_eq!("_".smush_amount("/", '$', 0xbf), 1);     // rule 2
        assert_eq!("/".smush_amount("_", '$', 0xbf), 1);     // rule 2
        assert_eq!("[".smush_amount("{", '$', 0xbf), 1);     // rule 3
        assert_eq!("[".smush_amount("]", '$', 0xbf), 1);     // rule 4
        assert_eq!(">".smush_amount("<", '$', 0xbf), 1);     // rule 5
        assert_eq!("[ ".smush_amount(" {", '$', 0xbf), 3);   // rule 3 + spacing
    }

    #[test]
    fn test_smusher_amount() {
        let output = vec_of_strings![ "", "", "", "" ];
        let fc = FIGchar{ lines: vec_of_strings![ "   ", "  x", " xx", "xx " ] };
        assert_eq!(smusher_amount(&output, &fc, '$', 0xbf), 0);

        let output = vec_of_strings![ "", "", "", "" ];
        let fc = FIGchar{ lines: vec_of_strings![ "   ", "  x", " xx", "   " ] };
        assert_eq!(smusher_amount(&output, &fc, '$', 0xbf), 1);

        let output = vec_of_strings![ "xxx ", "xx  ", "x   ", "    " ];
        let fc = FIGchar{ lines: vec_of_strings![ "   y", "  yy", " yyy", "yyyy" ] };
        assert_eq!(smusher_amount(&output, &fc, '$', 0xbf), 4);

        let output = vec_of_strings![  "xxxx ", "xxx  ", "xx   ", "x    " ];
        let fc = FIGchar{ lines: vec_of_strings![ "   x", "  xx", " xxx", "xxxx" ] };
        assert_eq!(smusher_amount(&output, &fc, '$', 0xbf), 5);
    }

    #[test]
    fn test_smush_str() {
        assert_eq!(smush_str("123! ", "xy", 1, '$', false, 0xbf).ok(), Some("123!xy".to_string()));
        assert_eq!(smush_str("123> ", "<y", 2, '$', false, 0xbf).ok(), Some("123Xy".to_string()));
        assert_eq!(smush_str("123! ", "   xy", 5, '$', false, 0xbf).ok(), Some("123xy".to_string()));
        assert_eq!(smush_str("123/ ", "   /y", 5, '$', false, 0xbf).ok(), Some("123/y".to_string()));
        assert_eq!(smush_str("", "   y", 3, '$', false, 0xbf).ok(), Some("y".to_string()));
        assert_eq!(smush_str("", "      ", 1, '$', false, 0xbf).ok(), Some("".to_string()));
    }
}
