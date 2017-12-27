extern crate rustlet;
use rustlet::{FIGfont, Smusher};

fn main() {
    match run() {
        Ok(msg) => msg.iter().for_each(|x| println!("{}", x)),
        Err(e)  => println!("Error: {}", e),
    }
}

fn run() -> Result<Vec<String>, rustlet::Error> {
    let font = FIGfont::from_file("fonts/standard.flf")?;
    let mut sm = Smusher::new(&font);
    sm.push_str("Hello world");
    Ok(sm.get())
}
