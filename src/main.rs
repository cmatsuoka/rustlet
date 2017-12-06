extern crate getopts;
extern crate rustlet;

use std::env;
use std::error::Error;
use std::path::PathBuf;
use getopts::{Matches, Options};
use rustlet::figfont::FIGfont;

const FONT_DIR    : &'static str = "/usr/share/figlet";
const DEFAULT_FONT: &'static str = "standard.flf";

fn main() {

    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();

    opts.optflag("c", "center", "center the output horizontally");
    opts.optopt("d", "dir", "set the default font directory", "dir");
    opts.optopt("f", "font", "specify the figfont to use", "name");
    opts.optflag("h", "help", "display usage information and exit");
    opts.optflag("k", "kern", "use kerning mode to display characters");
    opts.optflag("l", "left", "left-align the output");
    opts.optopt("m", "mode", "override the font layout mode", "num");
    opts.optflag("r", "right", "right-align the output");
    opts.optflag("S", "smush", "use smushing mode to display characters");
    opts.optflag("W", "full-width", "display characters in full width");
    opts.optopt("w", "width", "set the output width", "cols");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    if matches.opt_present("h") {
        let brief = format!("Usage: {} [options] message", args[0]);
        print!("{}", opts.usage(&brief));
        return;
    }

    let mut fontpath = PathBuf::from(match matches.opt_str("d") {
        Some(dir) => dir,
        None      => FONT_DIR.to_string(),
    });

    match matches.opt_str("f") {
        Some(name) => fontpath.push(name),
        None       => fontpath.push(DEFAULT_FONT),
    }

    match process(&fontpath.into_os_string().into_string().unwrap()) {
        Err(e) => { println!("Error: {}", e) }
        Ok(_)  => {},
    }
}

fn process(fontname: &str) -> Result<(), Box<Error>> {
    let mut font = FIGfont::new();
    try!(font.load(fontname));

    let mut sm = rustlet::Smusher::new(&font);
    try!(sm.push_str(&"Hello world!"));
    sm.print();

    Ok(())
}
