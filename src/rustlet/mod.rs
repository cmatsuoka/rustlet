use std::cmp::min;
use std::error::Error;
use self::figfont::{FIGchar, FIGfont};

pub mod figfont;
mod charsmush;
mod strsmush;


#[derive(Debug)]
pub struct Smusher<'a> {
    mode      : u32,
    right2left: bool,
    font      : &'a FIGfont,
    output    : Vec<String>,
}


impl<'a> Smusher<'a> {

    pub fn new(font: &'a FIGfont) -> Self {
        let mut sm = Smusher{
            font,
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
        amt = min(amt, strsmush::amount(&output[i], &c.lines[i], hardblank, mode));
    }
    amt
}

fn smusher_smush(output: &Vec<String>, fc: &FIGchar, hardblank: char, mode: u32) -> Result<Vec<String>, Box<Error>> {

    let amt = smusher_amount(&output, fc, hardblank, mode);
    let mut res = Vec::new();

    for i in 0..output.len() {
        res.push(strsmush::smush(&output[i], &fc.lines[i], amt, hardblank, false, mode)?);
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
}
