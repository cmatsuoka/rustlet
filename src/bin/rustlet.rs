extern crate getopts;
extern crate rustlet;
extern crate regex;

use std::env;
use std::io::{self, BufRead};
use std::path::{self, Path, PathBuf};
use getopts::{Matches, Options};
use regex::Regex;
use rustlet::Error;

const FONT_DIR     : &'static str = "/usr/share/figlet";
const DEFAULT_FONT : &'static str = "standard.flf";
const DEFAULT_WIDTH: usize = 79;


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
    opts.optflag("o", "overlap", "use character overlapping mode");
    opts.optflag("p", "paragraph", "ignore mid-paragraph line breaks");
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
    match run(&fontpath, &msg, &matches) {
        Err(e) => { println!("Error: {}", e) }
        Ok(_)  => {},
    }
}

fn find_font(mut fontpath: PathBuf, mut name: String) -> PathBuf {
    if !name.ends_with(".flf") && !name.ends_with(".tlf") {
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

fn run(path: &Path, msg: &str, matches: &Matches) -> Result<(), Error> {
    let mut font = rustlet::FIGfont::new();
    try!(font.load(path));

    let mut sm = rustlet::Smusher::new(&font);

    if matches.opt_present("o") {
        sm.mode = 0;
    } else if matches.opt_present("k") {
        sm.mode = rustlet::SMUSH_KERN;
    } else if matches.opt_present("W") {
        sm.full_width = true;
    }

    let width = match matches.opt_str("w") {
        Some(s) => try!(s.parse::<usize>()),
        None    => DEFAULT_WIDTH,
    };

    let mut wr = rustlet::Wrapper::new(&mut sm, width);

    if matches.opt_present("c") {
        wr.align = rustlet::Align::Center;
    } else if matches.opt_present("r") {
        wr.align = rustlet::Align::Right;
    }

    let re = Regex::new(r"(\S+|\s+)").unwrap();

    if msg.len() > 0 {
        // read message from command line parameters
        write_line(&mut wr, &msg, &re)
    } else {
        // read message from stdin
        let input = io::BufReader::new(io::stdin());
        if matches.opt_present("p") {
            input.lines().for_each(|x| write_paragraph(&mut wr, &x.unwrap(), &re));
            print_output(&wr.get());
        } else {
            input.lines().for_each(|x| write_line(&mut wr, &x.unwrap(), &re));
        }
    }

    Ok(())
}

fn write_line(wr: &mut rustlet::Wrapper, s: &str, re: &Regex) {
    wr.clear();
    write_tokens(wr, s, re);
    print_output(&wr.get());
}

fn write_paragraph(wr: &mut rustlet::Wrapper, s: &str, re: &Regex) {
    if s.starts_with(char::is_whitespace) && !wr.is_empty() {
        print_output(&wr.get());
        wr.clear();
    }
    write_tokens(wr, s, re);
}

fn write_tokens(wr: &mut rustlet::Wrapper, s: &str, re: &Regex) {
    re.captures_iter(s).for_each(|x| match x.get(0) {
        Some(val) => wr.wrap_str(val.as_str(), &print_output),
        None      => {},
    })
}

fn print_output(v: &Vec<String>) {
    v.iter().for_each(|x| println!("{}", x));
}
