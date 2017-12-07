use std::cmp::min;
use std::error::Error;
use self::figfont::{FIGchar, FIGfont};

pub mod figfont;
mod charsmush;
mod strsmush;


#[derive(Debug)]
pub struct Smusher<'a> {
    pub width : usize,
    buffer    : String,
    last      : String,
    mode      : u32,
    right2left: bool,
    wrap      : bool,
    font      : &'a FIGfont,
    output    : Vec<String>,
}


impl<'a> Smusher<'a> {

    pub fn new(font: &'a FIGfont) -> Self {
        let mut sm = Smusher{
            width     : 80,
            buffer    : String::new(),
            last      : String::new(),
            font,
            mode      : font.layout,
            right2left: false,
            wrap      : true,
            output    : Vec::new(),
        };
        for _ in 0..sm.font.height {
            sm.output.push("".to_string());
        }
        sm
    }

    pub fn set_wrap(mut self, width: usize) {
        self.width = width;
        self.wrap = width > 0;
    }

    pub fn amount(self, c: FIGchar) -> usize {
        amount(&self.output, &c, self.font.hardblank, self.mode)
    }

    pub fn get(&self) -> Vec<String> {
        let mut res: Vec<String> = Vec::new();
        for line in &self.output {
            res.push(line.replace(&self.font.hardblank.to_string(), " "));
        }
        res
    }

    pub fn print(self) {
        for p in self.get() {
            println!("{}", p);
        }
    }

    pub fn clear(&mut self) {
        self.clear_output();
        self.buffer.clear();
    }

    fn clear_output(&mut self) {
        for i in 0..self.output.len() {
            self.output[i].clear();
        }
    }

    pub fn push_word(&mut self, word: &str) -> Result<(), Box<Error>> {
        try!(self.push(' '));
        try!(self.push_str(word));

        if self.wrap && (self.len() > self.width) {
            let b = self.buffer.clone();
            try!(self.push_str(&b));
            self.clear_output();
            return Err(From::from("line full".to_string()))
        }

        self.buffer.push(' ');
        self.buffer.push_str(word);
        Ok(())
    }

    pub fn push_char(&mut self, ch: char) -> Result<(), Box<Error>> {
        try!(self.push(ch));

        if self.wrap && (self.len() > self.width) {
            let b = self.buffer.clone();
            try!(self.push_str(&b));
            self.clear_output();
            return Err(From::from("line full".to_string()))
        }

        self.buffer.push(ch);
        Ok(())
    }

    fn push_str(&mut self, s: &str) -> Result<(), Box<Error>> {
        for c in s.chars() {
            try!(self.push(c));
        }
        Ok(())
    }

    fn push(&mut self, ch: char) -> Result<(), Box<Error>> {
        let fc = self.font.get(ch);
        self.output = try!(smush(&self.output, fc, self.font.hardblank, self.mode));
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.output[0].len()
    }
}

fn amount(output: &Vec<String>, c: &FIGchar, hardblank: char, mode: u32) -> usize {
    let mut amt = 9999;
    for i in 0..output.len() {
        amt = min(amt, strsmush::amount(&output[i], &c.lines[i], hardblank, mode));
    }
    amt
}

fn smush(output: &Vec<String>, fc: &FIGchar, hardblank: char, mode: u32) -> Result<Vec<String>, Box<Error>> {

    let amt = amount(&output, fc, hardblank, mode);
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
    fn test_amount() {
        let output = vec_of_strings![ "", "", "", "" ];
        let fc = FIGchar{ lines: vec_of_strings![ "   ", "  x", " xx", "xx " ] };
        assert_eq!(amount(&output, &fc, '$', 0xbf), 0);

        let output = vec_of_strings![ "", "", "", "" ];
        let fc = FIGchar{ lines: vec_of_strings![ "   ", "  x", " xx", "   " ] };
        assert_eq!(amount(&output, &fc, '$', 0xbf), 1);

        let output = vec_of_strings![ "xxx ", "xx  ", "x   ", "    " ];
        let fc = FIGchar{ lines: vec_of_strings![ "   y", "  yy", " yyy", "yyyy" ] };
        assert_eq!(amount(&output, &fc, '$', 0xbf), 4);

        let output = vec_of_strings![  "xxxx ", "xxx  ", "xx   ", "x    " ];
        let fc = FIGchar{ lines: vec_of_strings![ "   x", "  xx", " xxx", "xxxx" ] };
        assert_eq!(amount(&output, &fc, '$', 0xbf), 5);
    }
}
