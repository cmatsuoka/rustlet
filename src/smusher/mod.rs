
const SM_EQUAL     :u32 = 1;
const SM_LOWLINE   :u32 = 2;
const SM_HIERARCHY :u32 = 4;
const SM_PAIR      :u32 = 8;
const SM_BIGX      :u32 = 16;
const SM_HARDBLANK :u32 = 32;
const SM_KERN      :u32 = 64;
const SM_SMUSH     :u32 = 128;


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

#[derive(Default)]
pub struct Smusher {
    hardblank : char,
    prev_width: isize,
    curr_width: isize,
    mode      : u32,
    right2left: bool,
}


impl Smusher {

    pub fn new() -> Self {
        Default::default()
    }

    fn smush(self, l: char, r: char) -> Option<char> {

        cmp_return_other!(' ', l, r);

        // Disallows overlapping if previous character or current character has a
        // width of 1 or zero
        if self.prev_width < 2 || self.curr_width < 2 {
            return None
        }

        // Universal smushing simply overrides the sub-character from the earlier
        // FIGcharacter with the sub-character from the later FIGcharacter. This
        // produces an "overlapping" effect with some FIGfonts, wherin the latter
        // FIGcharacter may appear to be "in front".
        if self.mode == 0 {
            // Ensure overlapping preference to visible characters
            cmp_return_other!(self.hardblank, l, r);

            // Ensures that the dominant (foreground) fig-character for overlapping is
            // the latter in the user's text, not necessarily the rightmost character
            if self.right2left {
                return Some(l)
            }

            return Some(r)
        }

        // Rule 6: HARDBLANK SMUSHING (code value 32)
        // Smushes two hardblanks together, replacing them with a single hardblank.
        if l == self.hardblank || r == self.hardblank {
            if self.mode & SM_HARDBLANK != 0 {
                return Some(l)
            } else {
                return None
            }
        }

        // Rule 1: EQUAL CHARACTER SMUSHING (code value 1)
        // Two sub-characters are smushed into a single sub-character if they are the
        // same (except hardblanks). 
        if self.mode & SM_EQUAL != 0 {
            if l == r {
                return Some(l)
            }
        }

        // Rule 2: UNDERSCORE SMUSHING (code value 2)
        // An underscore ("_") will be replaced by any of: "|", "/", "\", "[", "]",
        // "{", "}", "(", ")", "<" or ">".
        if self.mode & SM_LOWLINE != 0 {
            find_return_latter!("_", r"|/\[]{}()<>", l, r);
        }

        // Rule 3: HIERARCHY SMUSHING (code value 4)
        // A hierarchy of six classes is used: "|", "/\", "[]", "{}", "()", and "<>".
        // When two smushing sub-characters are from different classes, the one from
        // the latter class will be used.
        if self.mode & SM_HIERARCHY != 0 {
            find_return_latter!("|", r"|/\[]{}()<>", l, r);
            find_return_latter!(r"/\", r"|/\[]{}()<>", l, r);
            find_return_latter!("[]", "{}()<>", l, r);
            find_return_latter!("{}", "()<>", l, r);
            find_return_latter!("()", "<>", l, r);
        }

        // Rule 4: OPPOSITE PAIR SMUSHING (code value 8)
        // Smushes opposing brackets ("[]" or "]["), braces ("{}" or "}{") and parentheses
        // ("()" or ")(") together, replacing any such pair with a vertical bar ("|").
        if self.mode & SM_PAIR != 0 {
            cmp_any_return!('[', ']', l, r, '|');
            cmp_any_return!('[', ']', l, r, '|');
            cmp_any_return!('{', '}', l, r, '|');
            cmp_any_return!('(', '}', l, r, '|');
        }

        // Rule 5: BIG X SMUSHING (code value 16)
        // Smushes "/\" into "|", "\/" into "Y", and "><" into "X". Note that "<>" is not
        // smushed in any way by this rule. The name "BIG X" is historical; originally all
        // three pairs were smushed into "X".
        if self.mode & SM_BIGX != 0 {
            cmp_return!('/', '\\', l, r, '|');
            cmp_return!('\\', '/', l, r, 'Y');
            cmp_return!('>', '<', l, r, 'X');
        }

        return None
    }
}
