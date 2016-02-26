#![deny(warnings)]
#![feature(io)]
#![feature(unicode)]

extern crate coreutils;

use std::ascii::AsciiExt;
use std::env::args;
use std::io::{Write, Read, self};
use std::process::exit;

use coreutils::extra::OptionalExt;

static HELP: &'static str = r#"
    NAME
        mtxt - a simple tool to manipulate text from standard input.
    SYNOPSIS
        mtxt [-h | --help | -u | --to-uppercase-w | -l | --to-lowercase | -a | --strip-non-ascii]
    DESCRIPTION
        This utility will manipulate UTF-8 encoded text from the standard input. Unicode is supported.

        The flags are not compatible (for performance reasons). For combining the flags, piping is recommended.
    OPTIONS
        -h
        --help
            Print this manual page.
        -u
        --to-uppercase
            Convert the input to UPPERCASE. Works correctly with Unicode.
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
    let mut args = args().skip(1);
    let stdin = io::stdin();
    let stdin = stdin.lock();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    let arg = args.next().fail("no argument given.", &mut stdout);

    match arg.as_str() {
        "-u" | "--to-uppercase" => {
            for i in stdin.chars().flat_map(|x| x.unwrap().to_uppercase()) {
                let mut buf = [0; 4];
                let n = i.encode_utf8(&mut buf).try(&mut stdout);
                stdout.write(&buf[..n]).try(&mut stdout);
            }
        },
        "-l" | "--to-lowercase" => {
            for i in stdin.chars().flat_map(|x| x.unwrap().to_lowercase()) {
                let mut buf = [0; 4];
                let n = i.encode_utf8(&mut buf).try(&mut stdout);
                stdout.write(&buf[..n]).try(&mut stdout);
            }
        },
        "-a" | "--strip-non-ascii" => {
            for i in stdin.bytes().map(|x| x.unwrap()).filter(|x| x.is_ascii()) {
                stdout.write(&[i]).try(&mut stdout);
            }
        },
        "-h" | "--help" => {
            stdout.write(HELP.as_bytes()).try(&mut stdout);
        },
        a => {
            stdout.write(b"error: unknown argument, ").try(&mut stdout);
            stdout.write(a.as_bytes()).try(&mut stdout);
            stdout.write(b".\n").try(&mut stdout);
            stdout.flush().try(&mut stdout);
            exit(1);
        }
    }
}
