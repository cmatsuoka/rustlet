extern crate rustlet;

use std::error::Error;
use rustlet::figfont::FIGfont;


fn main() {

    match process() {
        Err(e) => { println!("Error: {}", e) }
        Ok(_)  => {},
    }
}

fn process() -> Result<(), Box<Error>> {
    let mut font = FIGfont::new();
    try!(font.load("/usr/share/figlet/small.flf"));

    let mut sm = rustlet::Smusher::new(&font);
    try!(sm.push_str(&"Hello world!"));
    sm.print();

    Ok(())
}
