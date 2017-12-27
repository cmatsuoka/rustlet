extern crate rustlet;

macro_rules! new_smusher {
    ( $a: ident, $b: expr ) => {
        let path = env!("CARGO_MANIFEST_DIR").to_owned() + "/tests/" + $b;
        let mut font = rustlet::FIGfont::new();
        assert!(!font.load(path).is_err());
        let $a = rustlet::Smusher::new(&font);
    }
}

fn dummy(_: &Vec<String>) {
}

#[test]
fn line_full() {
    new_smusher!(sm, "test.flf");
    let mut wr = rustlet::Wrapper::new(sm, 8);
    assert!(!wr.push_str("this").is_err());
    assert!(!wr.push_str(" ").is_err());
    assert!(!wr.push_str("is").is_err());
    assert!(!wr.push_str(" ").is_err());
    assert!(wr.push_str("a").is_err());
    assert_eq!(wr.get(), vec!["this is "]);
}

#[test]
fn line_wrap() {
    new_smusher!(sm, "test.flf");
    let mut wr = rustlet::Wrapper::new(sm, 8);
    [ "this", " ", "is", " ", "a", " ", "test" ].iter().for_each(|x| wr.wrap_str(&x, &dummy));
    assert_eq!(wr.get(), vec!["a test"]);
}

#[test]
fn alignment_left() {
    new_smusher!(sm, "test.flf");
    let mut wr = rustlet::Wrapper::new(sm, 12);
    wr.align = rustlet::Align::Left;
    [ "this", " ", "is", " ", "a", " ", "new", " ", "test" ].iter().for_each(|x| wr.wrap_str(&x, &dummy));
    assert_eq!(wr.get(), vec!["new test"]);
}

#[test]
fn alignment_center() {
    new_smusher!(sm, "test.flf");
    let mut wr = rustlet::Wrapper::new(sm, 12);
    wr.align = rustlet::Align::Center;
    [ "this", " ", "is", " ", "a", " ", "new", " ", "test" ].iter().for_each(|x| wr.wrap_str(&x, &dummy));
    assert_eq!(wr.get(), vec!["  new test"]);
}

#[test]
fn alignment_right() {
    new_smusher!(sm, "test.flf");
    let mut wr = rustlet::Wrapper::new(sm, 12);
    wr.align = rustlet::Align::Right;
    [ "this", " ", "is", " ", "a", " ", "new", " ", "test" ].iter().for_each(|x| wr.wrap_str(&x, &dummy));
    assert_eq!(wr.get(), vec!["    new test"]);
}

