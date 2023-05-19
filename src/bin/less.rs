extern crate extra;
extern crate pager;
extern crate termion;

use std::env::args;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use extra::option::OptionalExt;
use termion::raw::IntoRawMode;

static MAN_PAGE: &str = /* @MANSTART{less} */ r#"
NAME
    less - view a text file.

SYNOPSIS
    less [-h | --help] [input]

DESCRIPTION
    This utility views text files. If no input file is specified as an argument, standard input is
    used.

OPTIONS
    --help, -h
        Print this manual page.

AUTHOR
    This program was written by MovingtoMars and Ticki for Redox OS. Bugs, issues, or feature
    requests should be reported in the Gitlab repository, 'redox-os/extrautils'.

COPYRIGHT
    Copyright (c) 2016 MovingtoMars

    Permission is hereby granted, free of charge, to any person obtaining a copy of this software
    and associated documentation files (the "Software"), to deal in the Software without
    restriction, including without limitation the rights to use, copy, modify, merge, publish,
    distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the
    Software is furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all copies or
    substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING
    BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
    NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
    DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
"#; /* @MANEND */

fn main() {
    let mut args = args().skip(1).peekable();
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut stderr = io::stderr();

    if let Some(x) = args.peek() {
        if x == "--help" || x == "-h" {
            print!("{}", MAN_PAGE);
            return;
        }
    } else {
        let mut terminal = termion::get_tty().try(&mut stderr);
        run("-", &mut stdin, &mut terminal, io::stdout()).try(&mut stderr);
    };

    for filename in args {
        let file = File::open(Path::new(filename.as_str()));
        match file {
            Ok(mut open_file) => {
                if let Err(err) = run(filename.as_str(), &mut open_file, &mut stdin, io::stdout()) {
                    eprintln!("{}: {}", filename, err);
                }
            }
            Err(err) => {
                eprintln!("{}: {}", filename, err);
            }
        }
    }
}

// Run pager on a single file.
fn run<W: IntoRawMode>(
    path: &str,
    file: &mut dyn Read,
    controls: &mut dyn Read,
    stdout: W,
) -> std::io::Result<()> {
    let mut string = String::new();
    file.read_to_string(&mut string)?;
    let string2 = string.replace('\x1B', "?");

    pager::start(controls, stdout, path, &string2)
}
