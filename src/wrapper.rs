use super::Error;
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
    /// let mut font = rustlet::FIGfont::new();
    /// font.load("small.flf")?;
    /// let mut sm = rustlet::Smusher::new(&font);
    ///
    /// // Create a line wrapper using our smusher and maximum width of 80 columns
    /// let mut wr = rustlet::Wrapper::new(&mut sm, 80);
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
    /// let mut font = rustlet::FIGfont::new();
    /// font.load("small.flf")?;
    /// let mut sm = rustlet::Smusher::new(&font);
    /// let mut wr = rustlet::Wrapper::new(&mut sm, 80);
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
    pub fn get(&mut self) -> Vec<String> {
        if self.len() > self.width {
            self.sm.trim(self.width);
        }

        let w = self.width - self.len();
        let v = self.sm.get();

        match self.align {
            Align::Left   => v,
            Align::Center => add_pad(v, w / 2),
            Align::Right  => add_pad(v, w),
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
    /// the string is not added to the output buffer and a LineFull error is returned.
    pub fn push_str(&mut self, s: &str) -> Result<(), Error> {
        let empty = self.sm.is_empty();

        if !empty {
            self.sm.push(' ');
        }
        self.sm.push_str(s);

        if self.sm.len() > self.width {
            self.sm.clear();
            self.sm.push_str(&self.buffer);
            return Err(Error::LineFull)
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
    /// the character is not added to the output buffer and a LineFull error is returned.
    pub fn push(&mut self, ch: char) -> Result<(), Error> {
        self.sm.push(ch);

        if self.sm.len() > self.width {
            self.sm.clear();
            self.sm.push_str(&self.buffer);
            return Err(Error::LineFull)
        }

        self.buffer.push(ch);
        Ok(())
    }

    /// Add a string to the output buffer, wrapping it if necessary.
    ///
    /// If the new string causes the output to be wider than the maximum width, the current
    /// buffer contents (if any) will be passed to the flush callback, the buffer will be
    /// cleared, and the new string will be added to the buffer. If the string is wider
    /// than the output buffer, it will be wrapped at character level.
    pub fn wrap_str(&mut self, s: &str, flush: &Fn(&Vec<String>)) {
        match self.push_str(s) {
            Ok(_)  => {},
            Err(_) => {
                if !self.buffer.is_empty() {
                    flush(&self.get());
                    self.clear();
                }
                match self.push_str(s) {
                    Ok(_)  => {},
                    Err(_) => self.wrap(s, flush),
                }
            }
        }
    }
    
    /// Add a character to the output buffer, wrapping it if necessary.
    ///
    /// If the new character causes the output to be wider than the maximum width, the current
    /// buffer contents (if any) will be passed to the flush callback, the buffer will be
    /// cleared, and the new character will be added to the buffer. If the character is wider
    /// than the maximum width, it will be added without any additional processing.
    pub fn wrap(&mut self, word: &str, flush: &Fn(&Vec<String>)) {
        for c in word.chars() {
            match self.push(c) {
                Ok(_)  => {},
                Err(_) => {
                    if !self.buffer.is_empty() {
                        flush(&self.get());
                        self.clear();
                    }
                    // don't wrap this character
                    self.sm.push(c);
                    self.buffer.push(c);
                }
            }
        }
    }
}


fn add_pad(v: Vec<String>, pad_size: usize) -> Vec<String> {
    fn pad(num: usize) -> String {
        (0..num).map(|_| " ").collect::<String>()
    }
    v.iter().map(|x| pad(pad_size) + x).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! vec_string {
        ( $($x:expr),* ) => (vec![$($x.to_string()),*])
    }

    #[test]
    fn test_padding() {
        assert_eq!(add_pad(vec_string!("x", "x"), 0), vec_string!("x", "x"));
        assert_eq!(add_pad(vec_string!("x", "x"), 4), vec_string!("    x", "    x"));
    }

    #[test]
    fn test_padding_utf8() {
        assert_eq!(add_pad(vec_string!("á", "á"), 0), vec_string!("á", "á"));
        assert_eq!(add_pad(vec_string!("á", "á"), 4), vec_string!("    á", "    á"));
    }
}
