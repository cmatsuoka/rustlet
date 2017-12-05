use std::cmp::min;
use self::figfont::FIGfont;

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

    pub fn print(self) {
        for p in self.output {
            println!("{}", p);
        }
    }

    pub fn add_str(&mut self, s: &str) {
        for c in s.chars() {
            self.add_char(&c);
        }
    }

    pub fn add_char(&mut self, ch: &char) {
         let fc = self.font.get(ch);
         for i in 0..self.font.height {
             let s = str::replace(&fc.lines[i], self.font.hardblank, " ");
             self.output[i] += &s;
         }
    }

    fn smush(self, l: char, r: char) -> Option<char> {

        cmp_return_other!(' ', l, r);

        // Disallows overlapping if previous character or current character has a
        // width of 1 or zero
        if self.prev_width < 2 || self.curr_width < 2 {
            return None
        }

        Self::do_smush(l, r, self.font.hardblank, self.right2left, self.mode)
    }

    fn do_smush(l: char, r: char, hardblank: char, right2left: bool, mode: u32) -> Option<char> {

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

}

trait Smush {
    fn smush_amount(self, &str) -> usize;
}

impl<'a> Smush for &'a str {

    // Compute the number of characters a string can be smushed into another string.
    fn smush_amount(self, s: &str) -> usize {
    
        let a1 = self.len() - match self.rfind(|x| { let y:char = x; !y.is_whitespace() }) {
            Some(val) => val + 1,
            None      => 0,
        };
    
        let a2 = match s.find(|x| { let y:char = x; !y.is_whitespace() }) {
            Some(val) => val,
            None      => s.len(),
        };
    
        min(self.len(), a1 + a2 + 1)
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
            assert_eq!(smush_rule_2('_', *x, figfont::SMUSH_UNDERLINE), Some(*x));
        }
        assert_eq!(smush_rule_2('_', 'x', 0), None);
        assert_eq!(smush_rule_2('_', 'x', figfont::SMUSH_UNDERLINE), None);
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
        assert_eq!("    ".smush_amount("    "), 4);
        assert_eq!("x   ".smush_amount("    "), 4);
        assert_eq!("xx  ".smush_amount("    "), 4);
        assert_eq!("xxx ".smush_amount("    "), 4);
        assert_eq!("xxxx".smush_amount("    "), 4);

        assert_eq!("    ".smush_amount("   x"), 4);
        assert_eq!("x   ".smush_amount("   x"), 4);
        assert_eq!("xx  ".smush_amount("   x"), 4);
        assert_eq!("xxx ".smush_amount("   x"), 4);
        assert_eq!("xxxx".smush_amount("   x"), 4);

        assert_eq!("    ".smush_amount("  xx"), 4);
        assert_eq!("x   ".smush_amount("  xx"), 4);
        assert_eq!("xx  ".smush_amount("  xx"), 4);
        assert_eq!("xxx ".smush_amount("  xx"), 4);
        assert_eq!("xxxx".smush_amount("  xx"), 3);

        assert_eq!("    ".smush_amount(" xxx"), 4);
        assert_eq!("x   ".smush_amount(" xxx"), 4);
        assert_eq!("xx  ".smush_amount(" xxx"), 4);
        assert_eq!("xxx ".smush_amount(" xxx"), 3);
        assert_eq!("xxxx".smush_amount(" xxx"), 2);

        assert_eq!("    ".smush_amount("xxxx"), 4);
        assert_eq!("x   ".smush_amount("xxxx"), 4);
        assert_eq!("xx  ".smush_amount("xxxx"), 3);
        assert_eq!("xxx ".smush_amount("xxxx"), 2);
        assert_eq!("xxxx".smush_amount("xxxx"), 1);
    }
}
