use std::error::Error;
use std::path::PathBuf;
use rustlet::figfont::FIGfont;

mod rustlet;

fn main() {

    match process() {
        Err(e) => { println!("Error: {}", e) }
        Ok(_)  => {},
    }

    println!("Hello, world!");
}

fn process() -> Result<(), Box<Error>> {
    let mut font = FIGfont::new();
    try!(font.load(PathBuf::from("/usr/share/figlet/small.flf")));
    println!("{}", font.get(&'A'));
    println!("{}", font.get(&'B'));
    println!("{}", font.get(&'C'));

    Ok(())
}
