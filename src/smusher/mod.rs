
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

macro_rules! matches_return_other {
    ( $p1:expr, $p2:expr, $a:expr, $b:expr ) => {
        if $p1.find($a) != None && $p2.find($b) != None {
            return Some($b)
        }
        if $p1.find($b) != None && $p2.find($a) != None {
            return Some($a)
        }
    }
}

macro_rules! cmp_return {
    ( $c1:expr, $c2:expr, $a:expr, $b:expr, $r:expr ) => {
        if ($a == $c1 && $b == $c2) || ($b == $c1 && $a == $c2) {
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

        if self.mode & SM_HARDBLANK != 0 {
            if l == self.hardblank || r == self.hardblank {
                return Some(l)
            }
        }

        if l == self.hardblank || r == self.hardblank {
            return None
        }

        if self.mode & SM_EQUAL != 0 {
            if l == r {
                return Some(l)
            }
        }

        if self.mode & SM_LOWLINE != 0 {
            matches_return_other!("_", r"|/\[]{}()<>", l, r);
        }

        if self.mode & SM_HIERARCHY != 0 {
            matches_return_other!("|", r"|/\[]{}()<>", l, r);
            matches_return_other!(r"/\", r"|/\[]{}()<>", l, l);
            matches_return_other!("[]", "{}()<>", l, r);
            matches_return_other!("{}", "()<>", l, r);
            matches_return_other!("()", "<>", l, r);
        }

        if self.mode & SM_PAIR != 0 {
            cmp_return!('[', ']', l, r, '|');
            cmp_return!('{', '}', l, r, '|');
            cmp_return!('(', '}', l, r, '|');
        }

        if self.mode & SM_BIGX != 0 {
            cmp_return!('/', '\\', l, r, 'X');
            if l == '>' && r == '<' {
                return Some('X');
            }
        }

        return None
    }
}
