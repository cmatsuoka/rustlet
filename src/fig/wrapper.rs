use std::error::Error;
use super::Smusher;

enum Align {
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
        }
    }

    /// Clear the output buffer.
    pub fn clear(&mut self) {
        self.sm.clear();
        self.buffer.clear();
    }

    pub fn smush_mode(&mut self, mode: u32) {
        self.sm.mode = mode;
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
        self.sm.get()
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
            try!(self.sm.push(' '));
        }
        try!(self.sm.push_str(s));

        if self.sm.len() > self.width {
            let b = self.buffer.clone();
            self.sm.clear();
            try!(self.sm.push_str(&b));
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
        try!(self.sm.push(ch));

        if self.sm.len() > self.width {
            let b = self.buffer.clone();
            self.sm.clear();
            try!(self.sm.push_str(&b));
            return Err(From::from("line full".to_string()))
        }

        self.buffer.push(ch);
        Ok(())
    }
}
