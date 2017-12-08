use std::cmp::min;
use super::charsmush;


// Compute the number of characters a string can be smushed into another string.
pub fn amount(s1: &str, s2: &str, hardblank: char, mode: u32) -> usize {

    let a1 = s1.len() - match s1.rfind(|x| { let y:char = x; !y.is_whitespace() }) {
        Some(val) => val + 1,
        None      => 0,
    };

    let a2 = match s2.find(|x| { let y:char = x; !y.is_whitespace() }) {
        Some(val) => val,
        None      => s2.len(),
    };

    let amt = a1 + a2;

    // Retrieve character pair and see if they're smushable
    let (l, r) = match get_pair(s1, s2, amt + 1) {
        Some(pair) => pair,
        None       => { return amt; }
    };
    match charsmush::smush(l, r, hardblank, false, mode) {
        Some(_) => { amt + 1 },
        None    => { amt },
    }
}

pub fn smush(s1: &str, s2x: &str, mut amt: usize, hardblank: char, right2left: bool,
             mode: u32) -> String {

    let mut s2 = s2x;

    if amt > s1.len() {
        s2 = &s2[amt - s1.len()..];
        amt = s1.len();
    }

    let mut res = "".to_string();
    let m1 = s1.len() - amt;

    // part 1: only characters from s1
    for c in s1[..m1].chars() {
        res.push(c);
    }

    // part 2: s1 and s2 overlap
    for i in 0..s2.len() {
        let l = match s1.chars().nth(m1 + i) {
            Some(v) => v,
            None    => ' ',
        };
        let r = s2.chars().nth(i).unwrap();
        if l != ' ' && r != ' ' {
            match charsmush::smush(l, r, hardblank, false, mode) {
                Some(c) => res.push(c),
                None    => res.push(r),
            }
        } else {
            res.push(match l { ' ' => r, _ => l });
        }
    }

    // part 3: remainder of s1 after the end of s2
    let m2 = m1 + s2.len();
    if s1.len() > m2 {
        res.push_str(&s1[m2..]);
    }

    res
}

fn get_pair(s1: &str, s2: &str, amt: usize) -> Option<(char, char)> {
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_pair() {
        assert_eq!(get_pair("    ", "    ", 2), None);
        assert_eq!(get_pair("abc ", " xyz", 1), None);
        assert_eq!(get_pair("abc ", " xyz", 2), None);
        assert_eq!(get_pair("abc ", " xyz", 3), Some(('c', 'x')));
        assert_eq!(get_pair("abc   ", " x", 5), Some(('c', 'x')));
        assert_eq!(get_pair("a ", " xyzwt", 3), Some(('a', 'x')));
        assert_eq!(get_pair("a", "      x", 6), None);
        assert_eq!(get_pair("a", "      x", 7), Some(('a', 'x')));
        assert_eq!(get_pair("", "       x", 7), None);
    }

    #[test]
    fn test_amount() {
        assert_eq!(amount("", "", '$', 0xbf), 0);

        assert_eq!(amount("", "    ", '$', 0xbf), 4);
        assert_eq!(amount("", "   y", '$', 0xbf), 3);

        assert_eq!(amount("    ", "    ", '$', 0xbf), 8);
        assert_eq!(amount("x   ", "    ", '$', 0xbf), 7);
        assert_eq!(amount("xx  ", "    ", '$', 0xbf), 6);
        assert_eq!(amount("xxx ", "    ", '$', 0xbf), 5);
        assert_eq!(amount("XXXX", "    ", '$', 0xbf), 4);

        assert_eq!(amount("    ", "   y", '$', 0xbf), 7);
        assert_eq!(amount("x   ", "   y", '$', 0xbf), 6);
        assert_eq!(amount("xx  ", "   y", '$', 0xbf), 5);
        assert_eq!(amount("xxx ", "   y", '$', 0xbf), 4);
        assert_eq!(amount("xxxx", "   y", '$', 0xbf), 3);

        assert_eq!(amount("    ", "  yy", '$', 0xbf), 6);
        assert_eq!(amount("x   ", "  yy", '$', 0xbf), 5);
        assert_eq!(amount("xx  ", "  yy", '$', 0xbf), 4);
        assert_eq!(amount("xxx ", "  yy", '$', 0xbf), 3);
        assert_eq!(amount("xxxx", "  yy", '$', 0xbf), 2);

        assert_eq!(amount("    ", " yyy", '$', 0xbf), 5);
        assert_eq!(amount("x   ", " yyy", '$', 0xbf), 4);
        assert_eq!(amount("xx  ", " yyy", '$', 0xbf), 3);
        assert_eq!(amount("xxx ", " yyy", '$', 0xbf), 2);
        assert_eq!(amount("xxxx", " yyy", '$', 0xbf), 1);

        assert_eq!(amount("    ", "yyyy", '$', 0xbf), 4);
        assert_eq!(amount("x   ", "yyyy", '$', 0xbf), 3);
        assert_eq!(amount("xx  ", "yyyy", '$', 0xbf), 2);
        assert_eq!(amount("xxx ", "yyyy", '$', 0xbf), 1);
        assert_eq!(amount("xxxx", "yyyy", '$', 0xbf), 0);

        assert_eq!(amount("x", "y", '$', 0xbf), 0);
        assert_eq!(amount("x", "x", '$', 0xbf), 1);     // rule 1
        assert_eq!(amount("<", ">", '$', 0xbf), 0);
        assert_eq!(amount("_", "/", '$', 0xbf), 1);     // rule 2
        assert_eq!(amount("/", "_", '$', 0xbf), 1);     // rule 2
        assert_eq!(amount("[", "{", '$', 0xbf), 1);     // rule 3
        assert_eq!(amount("[", "]", '$', 0xbf), 1);     // rule 4
        assert_eq!(amount(">", "<", '$', 0xbf), 1);     // rule 5
        assert_eq!(amount("[ ", " {", '$', 0xbf), 3);   // rule 3 + spacing
    }

    #[test]
    fn test_smush() {
        assert_eq!(smush("123! ", "xy", 1, '$', false, 0xbf), "123!xy".to_string());
        assert_eq!(smush("123> ", "<y", 2, '$', false, 0xbf), "123Xy".to_string());
        assert_eq!(smush("123! ", "   xy", 5, '$', false, 0xbf), "123xy".to_string());
        assert_eq!(smush("123/ ", "   /y", 5, '$', false, 0xbf), "123/y".to_string());
        assert_eq!(smush("", "   y", 3, '$', false, 0xbf), "y".to_string());
        assert_eq!(smush("", "      ", 1, '$', false, 0xbf), "     ".to_string());
    }
}
