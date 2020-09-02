use std::env;
use std::process::{exit, Command};

static MAN_PAGE: &str = /* @MANSTART{man} */ r#"
NAME
    man - view a man page.

SYNOPSIS
    man [-h | --help] [page]

DESCRIPTION
    This utility launches less with a manual file.

OPTIONS
    --help, -h
        Print this manual page.

AUTHOR
    This program was written by Jeremy Soller for Redox OS. Bugs, issues, or feature requests
    should be reported in the Gitlab repository, 'redox-os/extrautils'.

COPYRIGHT
    Copyright (c) 2016 Jeremy Soller

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
    if let Some(arg) = env::args().nth(1) {
        match arg.as_str() {
            "--help" | "-h" => {
                // Print help.
                eprint!("{}", MAN_PAGE);
                exit(0);
            }
            page => {
                Command::new("less")
                    .arg(&("/ref/".to_owned() + page))
                    .spawn()
                    .unwrap()
                    .wait()
                    .unwrap();
            }
        }
    } else {
        eprintln!("Which manual page do you want?");
        exit(1);
    }
}
