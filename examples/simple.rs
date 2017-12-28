extern crate rustlet;

use std::io::{self, BufRead};
use rustlet::{FIGfont, Smusher, Wrapper};

fn main() {
    match run() {
        Ok(_)  => {},
        Err(e) => println!("Error: {}", e),
    }
}

fn run() -> Result<(), rustlet::Error> {
    let path = env!("CARGO_MANIFEST_DIR").to_owned() + "/fonts/small.flf";
    let font = FIGfont::from_path(&path)?;
    let mut wr = Wrapper::new(Smusher::new(&font), 78);

    // Read input from stdin send it to our line wrapper
    let input = io::BufReader::new(io::stdin());
    for line in input.lines() {
        wr.clear();
        line?.split_whitespace().for_each(|x| {wr.wrap_str(x, &print_output);});
        print_output(&wr.get());
    }
    Ok(())
}

fn print_output(v: &Vec<String>) {
    v.iter().for_each(|x| println!("{}", x));
}


