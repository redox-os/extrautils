//#![deny(warnings)]
#![feature(io)]

extern crate extra;

use std::env::args;
use std::io::{self, Read, Write};
use std::process::exit;

use extra::option::OptionalExt;
use extra::io::fail;

static MAN_PAGE: &'static str = /* @MANSTART{mtxt} */r#"
NAME
    mtxt - a simple tool to manipulate text from standard input.

SYNOPSIS
    mtxt (-u | --to-uppercase) [(-a | --strip-non-ascii)]
    mtxt (-l | --to-lowercase) [(-a | --strip-non-ascii)]
    mtxt (-a | --strip-non-ascii)
    mtxt (-h | --help)

DESCRIPTION
    This utility will change UTF-8(Unicode) encoded text from the standard input
    according to passed arguments.

OPTIONS
    -h
    --help
        Print this manual page.

    -u
    --to-uppercase
        Convert the input to UPPERCASE. Works correctly with Unicode. Incompatible with '-l'.

    -l
    --lowercase
        Convert the input to lowercase. Works correctly with Unicode. Incompatible with '-u'.

    -a
    --strip-non-ascii
        Strip the input for non-ASCII bytes, outputting a valid ASCII string.
        Compatible with '-l' and '-u'.

EXAMPLES
    $ echo Δ | mtxt -l
    > δ
    $ echo we got deltas Δ right | mtxt -a
    > we got deltas  right
    $ echo Japanese scripts do not have case thus 山will stay unchanged | mtxt -u
    > JAPANESE SCRIPTS DO NOT HAVE CASE THUS 山WILL STAY UNCHANGED

AUTHOR
    This program was written by Ticki for Redox OS. Bugs, issues, or feature requests should be
    reported in the Github repository, 'redox-os/extrautils'.

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
    let stdout = io::stdout();

    let stdin = stdin.lock();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();

    let mut to_uppercase = false;
    let mut to_lowercase = false;
    let mut strip_non_ascii = false;

    for arg in args().skip(1) {
        match arg.as_str() {
            "-u" | "--to-uppercase" => to_uppercase = true,
            "-l" | "--to-lowercase" => to_lowercase = true,
            "-a" | "--strip-non-ascii" => strip_non_ascii = true,
            "-h" | "--help" => {
                stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
                exit(0);
            }
            _ => fail(&format!("error: unknown argument, {}.\n", arg), &mut stderr),
        }
    }

    if to_lowercase && to_uppercase {
        fail("-u and -l are incompatible. Aborting.", &mut stderr);
    }

    // Handle five separate cases to eliminate checks:
    // 1. -u
    // 2. -u -a
    // 3. -l
    // 4. -l -a
    // 5. -a
    if to_uppercase {
        if strip_non_ascii {
            // -u -a
            stdin
                .chars()
                .filter(|x| {
                    if let &Ok(c) = x {
                        c.is_ascii()
                    } else {
                        false
                    }
                })
                .fold((), |_, x| {
                    stdout
                        .write(x.unwrap().to_uppercase().to_string().as_bytes())
                        .try(&mut stderr);
                });
        } else {
            // -u
            stdin.chars().fold((), |_, x| {
                stdout
                    .write(x.unwrap().to_uppercase().to_string().as_bytes())
                    .try(&mut stderr);
            });
        }
    } else if to_lowercase {
        if strip_non_ascii {
            // -l -a
            stdin
                .chars()
                .filter(|x| {
                    if let &Ok(c) = x {
                        c.is_ascii()
                    } else {
                        false
                    }
                })
                .fold((), |_, x| {
                    stdout
                        .write(x.unwrap().to_lowercase().to_string().as_bytes())
                        .try(&mut stderr);
                });
        } else {
            // -l
            stdin.chars().fold((), |_, x| {
                stdout
                    .write(x.unwrap().to_lowercase().to_string().as_bytes())
                    .try(&mut stderr);
            });
        }
    } else if strip_non_ascii {
        // -a
        stdin.chars().fold((), |_, x| {
            stdout
                .write(x.unwrap().to_string().as_bytes())
                .try(&mut stderr);
        });
    }
}
