extern crate rustlet;
use rustlet::{FIGfont, Smusher};

fn main() {
    match run() {
        Ok(msg) => msg.iter().for_each(|x| println!("{}", x)),
        Err(e)  => println!("Error: {}", e),
    }
}

fn run() -> Result<Vec<String>, rustlet::Error> {
    let path = env!("CARGO_MANIFEST_DIR").to_owned() + "/fonts/standard.flf";
    let font = FIGfont::from_file(&path)?;
    let mut sm = Smusher::new(&font);
    sm.push_str("Hello world");
    Ok(sm.get())
}
