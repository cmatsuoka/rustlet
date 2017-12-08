use std::cmp::min;
pub use super::figfont::{FIGchar, FIGfont};
pub use super::wrapper::Wrapper;

mod charsmush;
mod strsmush;

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
        if self.output.len() == 0 {
            return true;
        }
        self.output[0].is_empty()
    }

    pub fn clear(&mut self) {
        for i in 0..self.output.len() {
            self.output[i].clear();
        }
    }

    pub fn push_str(&mut self, s: &str) {
        for c in s.chars() {
            self.push(c);
        }
    }

    pub fn push(&mut self, ch: char) {
        let fc = self.font.get(ch);
        self.output = smush(&self.output, fc, self.font.hardblank, self.full_width, self.mode);
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

fn smush(output: &Vec<String>, fc: &FIGchar, hardblank: char, full_width: bool, mode: u32) -> Vec<String> {

    let amt = match full_width {
        true  => 0,
        false => amount(&output, fc, hardblank, mode),
    };

    let mut res = Vec::new();

    for i in 0..output.len() {
        res.push(strsmush::smush(&output[i], &fc.lines[i], amt, hardblank, false, mode));
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
