#![deny(warnings)]

extern crate extra;

use std::ascii::AsciiExt;
use std::env::args;
use std::io::{self, Write, Read};
use std::process::exit;

use extra::option::OptionalExt;
use extra::io::{fail, WriteExt};

static HELP: &'static str = r#"
    NAME
        mtxt - a simple tool to manipulate text from standard input.
    SYNOPSIS
        mtxt [-h | --help | -u | --to-uppercase-w | -l | --to-lowercase | -a | --strip-non-ascii]
    DESCRIPTION
        This utility will manipulate UTF-8 encoded text from the standard input. Unicode is supported. The flags are composable, unless otherwise stated.
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
        This program was written by Ticki for Redox OS. Bugs, issues, or feature requests should be reported in the Github repository, 'redox-os/extrautils'.
    COPYRIGHT
        Copyright (c) 2016 Ticki

        Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

        The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

        Do you actually read the license? wat?
        THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
"#;

fn main() {
    let stdin = io::stdin();
    let stdin = stdin.lock();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();

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
                // The help page.
                stdout.write(HELP.as_bytes()).try(&mut stderr);
            },
            a => {
                // The argument, a, is unknown.
                stderr.write(b"error: unknown argument, ").try(&mut stderr);
                stderr.write(a.as_bytes()).try(&mut stderr);
                stderr.write(b".\n").try(&mut stderr);
                stderr.flush().try(&mut stderr);
                exit(1);
            },
        }
    }

    if to_lowercase && to_uppercase {
        // Fail, since -u and -l are incompatible.
        fail("-u and -l are incompatible. Aborting.", &mut stderr);
    }

    // My eyes bleed.
    //
    // Anyone?
    // ...
    //
    // Silence.
    //
    // If you see this, rewrite the code below. Now, you can't say no. Too late.
    // Muhahahaha.
    for i in stdin.chars() {
        let i = i.try(&mut stderr);

        // TODO handle -a more efficient

        // If -u is set, convert to uppercase
        if to_uppercase {
            for uppercase in i.to_uppercase().filter(|x| !strip_non_ascii || x.is_ascii() ) {
                stdout.write_char(uppercase).try(&mut stderr);
            }
        // If -l is set, convert to lowercase
        } else if to_lowercase {
            for lowercase in i.to_lowercase().filter(|x| !strip_non_ascii || x.is_ascii()) {
                stdout.write_char(lowercase).try(&mut stderr);
            }
        // If -a is set, strip non-ASCII.
        } else if !strip_non_ascii || strip_non_ascii && i.is_ascii() {
            stdout.write_char(i).try(&mut stderr);
        }
    }
}
