extern crate extra;

use std::env::args;
use std::io::BufRead;
use std::io::{self};
use std::process::exit;

static MAN_PAGE: &str = /* @MANSTART{mtxt} */ r#"
NAME
    mtxt - a simple tool to manipulate text from standard input.

SYNOPSIS
    mtxt [-h | --help | -u | --to-uppercase-w | -l | --to-lowercase | -a | --strip-non-ascii]

DESCRIPTION
    This utility will manipulate UTF-8 encoded text from the standard input. Unicode is supported.
    The flags are composable, unless otherwise stated.

OPTIONS
    -h
    --help
        Print this manual page.

    -u
    --to-uppercase
        Convert the input to UPPERCASE. Works correctly with Unicode. Incompatible with '-l'.

    -l
    --lowercase
        Convert the input to lowercase. Works correctly with Unicode.

    -a
    --strip-non-ascii
        Strip the input for non-ASCII bytes, outputting a valid ASCII string.

EXAMPLES
    $ echo Δ | mtxt -l
    > δ
    $ echo we got deltas Δ right | mtxt -a
    > we got deltas  right
    $ echo Japanese scripts do not have case thus 山will stay unchanged | mtxt -u
    > JAPANESE SCRIPTS DO NOT HAVE CASE THUS 山WILL STAY UNCHANGED

AUTHOR
    This program was written by Ticki for Redox OS. Bugs, issues, or feature requests should be
    reported in the Gitlab repository, 'redox-os/extrautils'.

COPYRIGHT
    Copyright (c) 2016 Ticki

    Permission is hereby granted, free of charge, to any person obtaining a copy of this software
    and associated documentation files (the "Software"), to deal in the Software without
    restriction, including without limitation the rights to use, copy, modify, merge, publish,
    distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the
    Software is furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all copies or
    substantial portions of the Software.

    Do you actually read the license? wat?
    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING
    BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
    NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
    DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
"#; /* @MANEND */

fn main() {
    let stdin = io::stdin();
    let stdin = stdin.lock();

    // These are the options.
    let mut to_uppercase = false;
    let mut to_lowercase = false;
    let mut strip_non_ascii = false;

    for arg in args().skip(1) {
        match arg.as_str() {
            "-u" | "--to-uppercase" => to_uppercase = true,
            "-l" | "--to-lowercase" => to_lowercase = true,
            "-a" | "--strip-non-ascii" => strip_non_ascii = true,
            "-h" | "--help" => {
                print!("{}", MAN_PAGE);
            }
            a => {
                eprintln!("Error: unknown argument: {}", a);
                exit(1);
            }
        }
    }

    if to_lowercase && to_uppercase {
        eprintln!("-u and -l are incompatible. Aborting.");
        exit(1);
    }

    for u8_line in stdin.lines() {
        match u8_line {
            Err(err) => {
                eprintln!("{}", err);
                exit(1);
            }
            Ok(line) => {
                for c in line.chars() {
                    if !strip_non_ascii || c.is_ascii() {
                        if to_uppercase {
                            print!("{}", c.to_uppercase().to_string());
                        } else if to_lowercase {
                            print!("{}", c.to_lowercase().to_string());
                        } else {
                            print!("{}", c);
                        }
                    }
                }
                println!();
            }
        }
    }
}
