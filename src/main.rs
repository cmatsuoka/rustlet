extern crate getopts;
extern crate fig;

use std::env;
use std::io::{self, BufRead};
use std::error::Error;
use std::path::{self, Path, PathBuf};
use getopts::{Matches, Options};
use fig::wrapper::Align;

const FONT_DIR     : &'static str = "/usr/share/figlet";
const DEFAULT_FONT : &'static str = "standard.flf";
const DEFAULT_WIDTH: usize = 80;


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

    let msg = matches.free.join(" ");
    match process(&fontpath, &msg, &matches) {
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

fn process(path: &Path, msg: &str, matches: &Matches) -> Result<(), Box<Error>> {
    let mut font = fig::FIGfont::new();
    try!(font.load(path));

    let mut sm = fig::Smusher::new(&font);

    if matches.opt_present("k") {
        sm.mode = fig::SMUSH_KERN;
    } else if matches.opt_present("W") {
        sm.full_width = true;
    }

    let width = match matches.opt_str("w") {
        Some(s) => try!(s.parse::<usize>()),
        None    => DEFAULT_WIDTH,
    };

    let mut wr = fig::Wrapper::new(&mut sm, width);

    if matches.opt_present("c") {
        wr.align = Align::Center;
    } else if matches.opt_present("r") {
        wr.align = Align::Right;
    }

    if msg.len() > 0 {
        // read message from command line parameters
        write_line(&mut wr, &msg)
    } else {
        // read message from stdin
        let input = io::BufReader::new(io::stdin());
        for line in input.lines() {
            write_line(&mut wr, &line.unwrap())
        }
    }

    Ok(())
}

fn write_line(wr: &mut fig::Wrapper, s: &str) {
    wr.clear();
    for word in s.split_whitespace() {
        wr.wrap_str(word, &print_output);
    }
    print_output(&wr.get());
}

fn print_output(v: &Vec<String>) {
    for line in v {
        println!("{}", line);
    }
}
