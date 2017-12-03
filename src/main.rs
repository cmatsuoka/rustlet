use std::error::Error;
use std::path::PathBuf;
use rustlet::figfont::FIGfont;

mod rustlet;

fn main() {

    match process() {
        Err(e) => { println!("Error: {}", e) }
        Ok(_)  => {},
    }
}

fn process() -> Result<(), Box<Error>> {
    let mut font = FIGfont::new();
    try!(font.load(PathBuf::from("/usr/share/figlet/small.flf")));

    let mut sm = rustlet::Smusher::new(&font);
    sm.add_str(&"Hello world!");
    sm.print();

    Ok(())
}
