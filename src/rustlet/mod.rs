use std::cmp::min;
use std::error::Error;
use self::figfont::{FIGchar, FIGfont};

pub mod figfont;

macro_rules! cmp_return_other {
    ( $a:expr, $b:expr, $c:expr ) => {
        if $a == $b {
            return Some($c)
        }
        if $a == $c {
            return Some($b)
        }
    }
}

macro_rules! find_return_latter {
    ( $p1:expr, $p2:expr, $a:expr, $b:expr ) => {
        if $p1.find($a) != None && $p2.find($b) != None {
            return Some($b)
        }
        if $p1.find($b) != None && $p2.find($a) != None {
            return Some($a)
        }
    }
}

macro_rules! cmp_any_return {
    ( $c1:expr, $c2:expr, $a:expr, $b:expr, $r:expr ) => {
        if ($a == $c1 && $b == $c2) || ($a == $c2 && $b == $c1) {
            return Some($r)
        }
    }
}

macro_rules! cmp_return {
    ( $c1:expr, $c2:expr, $a:expr, $b:expr, $r:expr ) => {
        if $a == $c1 && $b == $c2 {
            return Some($r)
        }
    }
}

macro_rules! apply_rule {
    ( $cmd:expr ) => {
        match $cmd {
            Some(val) => return Some(val),
            None      => {},
        }
    }
}


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

        cmp_return_other!(' ', l, r);

        // Disallows overlapping if previous character or current character has a
        // width of 1 or zero
        if self.prev_width < 2 || self.curr_width < 2 {
            return None
        }

        smusher_smush_char(l, r, self.font.hardblank, self.right2left, self.mode)
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

fn smusher_smush_char(l: char, r: char, hardblank: char, right2left: bool, mode: u32) -> Option<char> {

    // Universal smushing simply overrides the sub-character from the earlier
    // FIGcharacter with the sub-character from the later FIGcharacter. This
    // produces an "overlapping" effect with some FIGfonts, wherin the latter
    // FIGcharacter may appear to be "in front".
    if mode == 0 {
        // Ensure overlapping preference to visible characters
        cmp_return_other!(hardblank, l, r);

        // Ensures that the dominant (foreground) fig-character for overlapping is
        // the latter in the user's text, not necessarily the rightmost character
        if right2left {
            return Some(l)
        }

        return Some(r)
    }

    // Rule 6: HARDBLANK SMUSHING (code value 32)
    // Smushes two hardblanks together, replacing them with a single hardblank.
    if l == hardblank && r == hardblank {
        if mode & figfont::SMUSH_HARDBLANK != 0 {
            return Some(l);
        } else {
            return None;
        }
    }

    apply_rule!(smush_rule_1(l, r, mode));
    apply_rule!(smush_rule_2(l, r, mode));
    apply_rule!(smush_rule_3(l, r, mode));
    apply_rule!(smush_rule_4(l, r, mode));
    apply_rule!(smush_rule_5(l, r, mode));

    None
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
        match smusher_smush_char(l, r, hardblank, false, mode) {
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

    match smusher_smush_char(l, r, hardblank, false, mode) {
        Some(c) => { res.push(c); res.push_str(&s2[1..]) },
        None    => res += s2,
    }

    Ok(res)
}

// Rule 1: EQUAL CHARACTER SMUSHING (code value 1)
// Two sub-characters are smushed into a single sub-character if they are the same (except
// hardblanks). 
fn smush_rule_1(l: char, r: char, mode: u32) -> Option<char> {
    if mode & figfont::SMUSH_EQUAL != 0 {
        if l == r {
            return Some(l)
        }
    }

    None
}

// Rule 2: UNDERSCORE SMUSHING (code value 2)
// An underscore ("_") will be replaced by any of: "|", "/", "\", "[", "]", "{", "}", "(",
// ")", "<" or ">".
fn smush_rule_2(l: char, r: char, mode: u32) -> Option<char> {
    if mode & figfont::SMUSH_UNDERLINE != 0 {
        find_return_latter!("_", r"|/\[]{}()<>", l, r);
    }

    None
}

// Rule 3: HIERARCHY SMUSHING (code value 4)
// A hierarchy of six classes is used: "|", "/\", "[]", "{}", "()", and "<>". When two
// smushing sub-characters are from different classes, the one from the latter class will
// be used.
fn smush_rule_3(l: char, r: char, mode: u32) -> Option<char> {
    if mode & figfont::SMUSH_HIERARCHY != 0 {
        find_return_latter!("|", r"/\[]{}()<>", l, r);
        find_return_latter!(r"/\", "[]{}()<>", l, r);
        find_return_latter!("[]", "{}()<>", l, r);
        find_return_latter!("{}", "()<>", l, r);
        find_return_latter!("()", "<>", l, r);
    }

    None
}


// Rule 4: OPPOSITE PAIR SMUSHING (code value 8)
// Smushes opposing brackets ("[]" or "]["), braces ("{}" or "}{") and parentheses ("()"
// or ")(") together, replacing any such pair with a vertical bar ("|").
fn smush_rule_4(l: char, r: char, mode: u32) -> Option<char> {
    if mode & figfont::SMUSH_PAIR != 0 {
        cmp_any_return!('[', ']', l, r, '|');
        cmp_any_return!('{', '}', l, r, '|');
        cmp_any_return!('(', ')', l, r, '|');
    }

    None
}

// Rule 5: BIG X SMUSHING (code value 16)
// Smushes "/\" into "|", "\/" into "Y", and "><" into "X". Note that "<>" is not smushed
// in any way by this rule. The name "BIG X" is historical; originally all three pairs
// were smushed into "X".
fn smush_rule_5(l: char, r: char, mode: u32) -> Option<char> {
    if mode & figfont::SMUSH_BIGX != 0 {
        cmp_return!('/', '\\', l, r, '|');
        cmp_return!('\\', '/', l, r, 'Y');
        cmp_return!('>', '<', l, r, 'X');
    }

    None
}


#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! vec_of_strings {
        ( $($x:expr),* ) => (vec![$($x.to_string()),*])
    }

    #[test]
    fn test_rule_1() {
        assert_eq!(smush_rule_1('-', '-', 0), None);
        assert_eq!(smush_rule_1('-', 'x', 0), None);
        assert_eq!(smush_rule_1('-', '-', figfont::SMUSH_EQUAL), Some('-'));
        assert_eq!(smush_rule_1('-', 'x', figfont::SMUSH_EQUAL), None);
    }

    #[test]
    fn test_rule_2() {
        for x in [ '|', '/', '\\', '[', ']', '{', '}', '(', ')', '<', '>' ].iter() {
            assert_eq!(smush_rule_2('_', *x, 0), None);
            assert_eq!(smush_rule_2(*x, '_', 0), None);
            assert_eq!(smush_rule_2('_', *x, figfont::SMUSH_UNDERLINE), Some(*x));
            assert_eq!(smush_rule_2(*x, '_', figfont::SMUSH_UNDERLINE), Some(*x));
        }
        assert_eq!(smush_rule_2('_', 'x', 0), None);
        assert_eq!(smush_rule_2('x', '_', 0), None);
        assert_eq!(smush_rule_2('_', 'x', figfont::SMUSH_UNDERLINE), None);
        assert_eq!(smush_rule_2('x', '_', figfont::SMUSH_UNDERLINE), None);
    }

    #[test]
    fn test_rule_3() {
        let list = [ '|', '/', '\\', '[', ']', '{', '}', '(', ')', '<', '>' ];

        for x in list.iter() {
            for y in list.iter() {
                assert_eq!(smush_rule_3(*x, *y, 0), None);
            }
        }

        assert_eq!(smush_rule_3('|', '|', figfont::SMUSH_HIERARCHY), None);
        for x in list[1..].iter() {
            assert_eq!(smush_rule_3('|', *x, figfont::SMUSH_HIERARCHY), Some(*x));
        }
        for i in 0..4 {
            let idx = 3 + i*2;
            for x in list[idx..idx+1].iter() {
                for y in list[idx+2..].iter() {
                    assert_eq!(smush_rule_3(*x, *y, figfont::SMUSH_HIERARCHY), Some(*y));
                    assert_eq!(smush_rule_3(*y, *x, figfont::SMUSH_HIERARCHY), Some(*y));
                }
            }
        }
    }

    #[test]
    fn test_rule_4() {
        assert_eq!(smush_rule_4('[', ']', 0), None);
        assert_eq!(smush_rule_4(']', '[', 0), None);
        assert_eq!(smush_rule_4('{', '}', 0), None);
        assert_eq!(smush_rule_4('}', '{', 0), None);
        assert_eq!(smush_rule_4('(', ')', 0), None);
        assert_eq!(smush_rule_4(')', '(', 0), None);
        assert_eq!(smush_rule_4('[', ']', figfont::SMUSH_PAIR), Some('|'));
        assert_eq!(smush_rule_4(']', '[', figfont::SMUSH_PAIR), Some('|'));
        assert_eq!(smush_rule_4('{', '}', figfont::SMUSH_PAIR), Some('|'));
        assert_eq!(smush_rule_4('}', '{', figfont::SMUSH_PAIR), Some('|'));
        assert_eq!(smush_rule_4('(', ')', figfont::SMUSH_PAIR), Some('|'));
        assert_eq!(smush_rule_4(')', '(', figfont::SMUSH_PAIR), Some('|'));
        assert_eq!(smush_rule_4('(', 'x', figfont::SMUSH_PAIR), None);
        assert_eq!(smush_rule_4('(', '}', figfont::SMUSH_PAIR), None);
        assert_eq!(smush_rule_4('(', ']', figfont::SMUSH_PAIR), None);
        assert_eq!(smush_rule_4('(', '(', figfont::SMUSH_PAIR), None);
    }

    #[test]
    fn test_rule_5() {
        assert_eq!(smush_rule_5('/', '\\', 0), None);
        assert_eq!(smush_rule_5('\\', '/', 0), None);
        assert_eq!(smush_rule_5('>', '<', 0), None);
        assert_eq!(smush_rule_5('<', '>', 0), None);
        assert_eq!(smush_rule_5('/', '\\', figfont::SMUSH_BIGX), Some('|'));
        assert_eq!(smush_rule_5('\\', '/', figfont::SMUSH_BIGX), Some('Y'));
        assert_eq!(smush_rule_5('>', '<', figfont::SMUSH_BIGX), Some('X'));
        assert_eq!(smush_rule_5('<', '>', figfont::SMUSH_BIGX), None);
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
