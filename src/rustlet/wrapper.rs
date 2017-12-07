use std::error::Error;
use super::Smusher;

/// Render FIGcharacters with word wrapping.
///
/// Wrapper receives string or character input and renders the corresponding
/// FIGcharacters if the output text fits inside the maximum width specified on
/// creation.
pub struct Wrapper<'a> {
    sm       : &'a mut Smusher<'a>,
    buffer   : String,
    pub width: usize,
}

impl<'a> Wrapper<'a> {

    /// Create a new wrapper using the specified Smusher and terminal width.
    ///
    /// # Examples
    ///
    /// ```
    /// // Create a smusher using the specified FIGfont
    /// let mut sm = rustlet::Smusher::new("small.flf");
    ///
    /// // Create a line wrapper using our smusher and maximum width of 80 columns
    /// let mut wr = rustlet::Wrapper::new(&mut sm, 80);
    /// ```
    pub fn new(sm: &'a mut Smusher<'a>, width: usize) -> Self {
        Wrapper{
           sm,
           width,
           buffer: String::new(),
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
    /// let mut sm = rustlet::Smusher::new("small.flf");
    /// let mut wr = rustlet::Wrapper::new(&mut sm, 80);
    ///
    /// try!(wr.push_str("hello"));
    ///
    /// for line in wr.get() {
    ///    println!("{}", line);
    /// }
    /// ```
    pub fn get(self) -> Vec<String> {
        self.sm.get()
    }

    /// Add a string to the output buffer. If adding the string results in a line
    /// wider than the maximum number of columns, the string is not added and an
    /// error is returned.
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

    /// Add a character to the output buffer. If adding the character results in a
    /// line wider than the maximum number of columns, the character is not added
    /// and an error is returned.
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
