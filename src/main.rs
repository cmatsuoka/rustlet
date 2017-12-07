extern crate getopts;
extern crate rustlet;

use std::env;
use std::error::Error;
use std::path::{self, PathBuf};
use getopts::Options;
use rustlet::FIGfont;

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
        Some(name) => fontpath = find_font(fontpath, name),
        None       => fontpath.push(DEFAULT_FONT),
    }

    match process(&fontpath.into_os_string().into_string().unwrap()) {
        Err(e) => { println!("Error: {}", e) }
        Ok(_)  => {},
    }
}

fn find_font(mut fontpath: PathBuf, mut name: String) -> PathBuf {
    if !name.ends_with(".flf") {
        name = format!("{}.flf", name);
    }

    if name.starts_with(path::MAIN_SEPARATOR) {
        return PathBuf::from(&name);
    }

    fontpath.push(&name);
    if fontpath.exists() {
        return fontpath;
    }

    PathBuf::from(&name)
}

fn process(fontname: &str) -> Result<(), Box<Error>> {
    let mut font = FIGfont::new();
    try!(font.load(fontname));

    let mut sm = rustlet::Smusher::new(&font);
    let mut wr = rustlet::Wrapper::new(&mut sm, 80);

    let s = "The quick brown fox jumps over the lazy dog";

    write_line(&mut wr, &s);

    Ok(())
}

fn write_line(wr: &mut rustlet::Wrapper, s: &str) {

    let v: Vec<&str> = s.split(' ').collect();
    for word in v {
        match wr.push_str(word) {
            Ok(_)  => {},
            Err(_) => {
                print_output(&wr);
                wr.clear();
                wr.push_str(word);
            }
        }
    }
    print_output(&wr);
}

fn print_output(wr: &rustlet::Wrapper) {
    for line in &wr.get() {
        println!("{}", line);
    }
}
