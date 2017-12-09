use std::error::Error;
use super::Smusher;

pub enum Align {
    Left,
    Right,
    Center,
}

/// Render smushed ASCII-art characters with word wrapping.
///
/// Wrapper receives string or character input and renders the corresponding
/// FIGcharacters if the output text fits inside the maximum width specified on
/// creation.
pub struct Wrapper<'a> {
    sm        : &'a mut Smusher<'a>,
    pub buffer: String,
    pub width : usize,
    pub align : Align,
}

impl<'a> Wrapper<'a> {

    /// Create a new wrapper using the specified Smusher and terminal width.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn foo() -> Result<(), Box<std::error::Error>> {
    /// // Create a smusher using the specified FIGfont
    /// let mut font = fig::FIGfont::new();
    /// font.load("small.flf")?;
    /// let mut sm = fig::Smusher::new(&font);
    ///
    /// // Create a line wrapper using our smusher and maximum width of 80 columns
    /// let mut wr = fig::Wrapper::new(&mut sm, 80);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(sm: &'a mut Smusher<'a>, width: usize) -> Self {
        Wrapper{
           sm,
           width,
           buffer: String::new(),
           align : Align::Left,
        }
    }

    /// Clear the output buffer.
    pub fn clear(&mut self) {
        self.sm.clear();
        self.buffer.clear();
    }


    /// Retrieve the output buffer lines.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn foo() -> Result<(), Box<std::error::Error>> {
    /// // Create a new wrapper
    /// let mut font = fig::FIGfont::new();
    /// font.load("small.flf")?;
    /// let mut sm = fig::Smusher::new(&font);
    /// let mut wr = fig::Wrapper::new(&mut sm, 80);
    ///
    /// // Add a string to the output buffer
    /// wr.push_str("hello")?;
    ///
    /// // Get and print the current output buffer contents
    /// for line in wr.get() {
    ///     println!("{}", line);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get(&self) -> Vec<String> {
        let l = self.width - self.len();
        let v = self.sm.get();

        match self.align {
            Align::Left   => v,
            Align::Center => align_center(v, l),
            Align::Right  => align_right(v, l),
        }
    }

    /// Get the length in sub-characters of the current output buffer.
    pub fn len(&self) -> usize {
        self.sm.len()
    }

    /// Add a string to the output buffer.
    ///
    /// # Errors
    ///
    /// If adding the string results in a line wider than the maximum number of columns,
    /// the string is not added to the output buffer and an error is returned.
    pub fn push_str(&mut self, s: &str) -> Result<(), Box<Error>> {
        let empty = self.sm.is_empty();

        if !empty {
            self.sm.push(' ');
        }
        self.sm.push_str(s);

        if self.sm.len() > self.width {
            let b = self.buffer.clone();
            self.sm.clear();
            self.sm.push_str(&b);
            return Err(From::from("line full".to_string()))
        }

        if !empty {
            self.buffer.push(' ');
        }
        self.buffer.push_str(s);
        Ok(())
    }

    /// Add a character to the output buffer.
    ///
    /// # Errors
    ///
    /// If adding the character results in a line wider than the maximum number of columns,
    /// the character is not added to the output buffer and an error is returned.
    pub fn push(&mut self, ch: char) -> Result<(), Box<Error>> {
        self.sm.push(ch);

        if self.sm.len() > self.width {
            let b = self.buffer.clone();
            self.sm.clear();
            self.sm.push_str(&b);
            return Err(From::from("line full".to_string()))
        }

        self.buffer.push(ch);
        Ok(())
    }

    pub fn push_nowrap(&mut self, ch: char) {
        self.sm.push(ch);
        self.buffer.push(ch);
    }

    pub fn wrap_str(&mut self, s: &str, flush: &Fn(&Vec<String>)) {
        match self.push_str(s) {
            Ok(_)  => {},
            Err(_) => {
                flush(&self.get());
                self.clear();
                match self.push_str(s) {
                    Ok(_)  => {},
                    Err(_) => self.wrap(s, flush),
                }
            }
        }
    }
    
    fn wrap(&mut self, word: &str, flush: &Fn(&Vec<String>)) {
        for c in word.chars() {
            match self.push(c) {
                Ok(_)  => {},
                Err(_) => {
                    if !self.buffer.is_empty() {
                        flush(&self.get());
                        self.clear();
                    }
                    self.push_nowrap(c);
                }
            }
        }
    }
}

fn pad(num: usize) -> String {
    (0..num).map(|_| " ").collect::<String>()
}

fn align_center(v: Vec<String>, width: usize) -> Vec<String> {
    v.iter().map(|x| pad((width - 1) / 2) + x).collect()
}

fn align_right(v: Vec<String>, width: usize) -> Vec<String> {
    v.iter().map(|x| pad(width - 1) + x).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! vec_of_strings {
        ( $($x:expr),* ) => (vec![$($x.to_string()),*])
    }

    #[test]
    fn test_align_center() {
        assert_eq!(align_center(vec_of_strings!("x", "x"), 5), vec_of_strings!("  x", "  x"));
    }

    #[test]
    fn test_align_right() {
        assert_eq!(align_right(vec_of_strings!("x", "x"), 5), vec_of_strings!("    x", "    x"));
    }
}
