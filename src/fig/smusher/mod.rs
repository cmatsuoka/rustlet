use std::cmp::min;
pub use super::figfont::{FIGchar, FIGfont};
pub use super::wrapper::Wrapper;
pub use super::CharExt;

mod charsmush;
pub mod strsmush;

/// Creates a message written with ASCII-art characters.
///
/// The Smusher adds FIGcharacters to an output buffer and controls how they share
/// border sub-characters with the content that's already in the buffer. Details
/// of how exactly this smushing happens is given by its layout mode.
#[derive(Debug)]
pub struct Smusher<'a> {
    pub mode      : u32,          // the layout mode
    pub full_width: bool,
    pub right2left: bool,
    font          : &'a FIGfont,
    output        : Vec<String>,
}


impl<'a> Smusher<'a> {

    /// Create a new smusher using the specified FIGfont.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn foo() -> Result<(), Box<std::error::Error>> {
    /// // Load a FIGfont
    /// let mut font = fig::FIGfont::new();
    /// font.load("small.flf")?;
    ///
    /// // Create a smusher using the FIGfont
    /// let mut sm = fig::Smusher::new(&font);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(font: &'a FIGfont) -> Self {
        let mut sm = Smusher{
            font,
            mode      : font.layout,
            full_width: font.old_layout == -1,
            right2left: false,
            output    : Vec::new(),
        };
        for _ in 0..sm.font.height {
            sm.output.push("".to_string());
        }
        sm
    }

    /// Get the contents of the output buffer.
    pub fn get(&self) -> Vec<String> {
        let mut res: Vec<String> = Vec::new();
        for line in &self.output {
            res.push(line.replace(&self.font.hardblank.to_string(), " "));
        }
        res
    }

    pub fn is_empty(&self) -> bool {
        self.output.len() == 0 || self.output[0].is_empty()
    }

    pub fn clear(&mut self) {
        self.output.iter_mut().for_each(|x| x.clear());
    }

    pub fn push_str(&mut self, s: &str) {
        s.chars().for_each(|x| self.push(x));
    }

    pub fn push(&mut self, ch: char) {
        let fc = self.font.get(ch);
        self.output = smush(&self.output, fc, self.font.hardblank, self.full_width, self.mode);
    }

    pub fn len(&self) -> usize {
        let s: &str = &self.output[0];
        s.char_len()
    }

    pub fn trim(&mut self, width: usize) {
        self.output = self.output.iter().map(|line| {
            let s: &str = &line;
            s[..s.char_index(width)].to_string()
        }).collect()
    }
}

fn amount(output: &Vec<String>, c: &FIGchar, hardblank: char, mode: u32) -> usize {
    let mut amt = 9999;
    for (line, cline) in output.iter().zip(&c.lines) {
        amt = min(amt, strsmush::amount(&line, &cline, hardblank, mode));
    }
    amt
}

fn smush(output: &Vec<String>, c: &FIGchar, hardblank: char, full_width: bool, mode: u32) -> Vec<String> {

    let amt = match full_width {
        true  => 0,
        false => amount(&output, c, hardblank, mode),
    };

    let mut res = Vec::new();

    for (line, cline) in output.iter().zip(&c.lines) {
        res.push(strsmush::smush(&line, &cline, amt, hardblank, false, mode));
    }

    res
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
