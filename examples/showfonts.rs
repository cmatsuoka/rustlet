extern crate rustlet;

use std::error::Error;
use std::fs;
use std::path::PathBuf;
use rustlet::{FIGfont, Smusher};

fn main() {
    match run() {
        Ok(_)  => {},
        Err(e) => println!("Error: {}", e),
    }
}

fn run() -> Result<(), Box<Error>> {
    let path = env!("CARGO_MANIFEST_DIR").to_owned() + "/fonts";
    let mut fonts: Vec<_> = fs::read_dir(&path)?.map(|x| x.unwrap().path()).collect();
    fonts.sort();
    for f in fonts {
        show_font(f, &path)?;
    }
    Ok(())
}

fn show_font(p: PathBuf, prefix: &str) -> Result<(), Box<Error>> {
    let font = FIGfont::from_file(p.to_str().unwrap())?;
    let name = p.strip_prefix(prefix)?.file_stem().unwrap().to_str().unwrap();
    println!("{}:", name); 
    let mut sm = Smusher::new(&font);
    sm.push_str(name);
    sm.get().iter().for_each(|x| println!("{}", x));
    println!("\n");
    Ok(())
}
