#![deny(warnings)]

extern crate coreutils;

use std::env::args;
use std::io::{self, Write};
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

use coreutils::extra::OptionalExt;

static HELP: &'static str = r#"
    NAME
        rem - set a count-down.
    SYNOPSIS
        rem [-h | --help] [-m N | --minutes N] [-H N | --hours N] [-s N | --seconds N] [-M N | --milliseconds N] [-n | --len]
    DESCRIPTION
        This utility lets you set a count-down with a progress bar. The options can be given in combination, adding together the durations given.
    OPTIONS
        -h
        --help
            Print this manual page.
        -m N
        --minutes N
            Wait N minutes.
        -H N
        --hours N
            Wait N hours.
        -s N
        --seconds N
            Wait N seconds.
        -M N
        --milliseconds N
            Wait N milliseconds.
        -n
        --len
            Length of the progress bar.
    AUTHOR
        This program was written by Ticki for Redox OS. Bugs, issues, or feature requests should be reported in the Github repository, 'redox-os/extrautils'.
    COPYRIGHT
        Copyright (c) 2016 Ticki

        Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

        The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

        THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
"#;

fn main() {
    let mut args = args().skip(1);
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();

    let mut ms = 0u64;
    let mut len = 20;

    // Loop over the arguments.
    loop {
        let arg = if let Some(x) = args.next() {
            x
        } else {
            break;
        };

        match arg.as_str() {
            "-h" | "--help" => {
                // Print help.
                stdout.write(HELP.as_bytes()).try(&mut stderr);
                return;
            },
            "-n" | "--len" => len = args.next().fail("no number after -n.", &mut stderr).parse().try(&mut stderr),
            t => {
                // Find number input.
                let num: u64 = args.next().unwrap_or_else(|| {
                    stderr.write(b"error: incorrectly formatted number.\
                                   Please input a positive integer.").try(&mut stderr);
                    stderr.flush().try(&mut stderr);
                    exit(1);
                }).parse().try(&mut stderr);
                ms += num * match t {
                    "-m" | "--minutes" => 1000 * 60,
                    "-H" | "--hours" => 1000 * 60 * 60,
                    "-s" | "--seconds" => 1000,
                    "-M" | "--milliseconds" => 1,
                    _ => {
                        // Unknown argument.
                        stderr.write(b"error: unknown argument, ").try(&mut stderr);
                        stderr.write(t.as_bytes()).try(&mut stderr);
                        stderr.write(b".\n").try(&mut stderr);
                        stderr.flush().try(&mut stderr);
                        exit(1);
                    },

                };
            },
        }
    }

    // Hide the cursor.
    stdout.write(b"\x1b[?25l").try(&mut stderr);
    // Draw the empty progress bar.
    for _ in 0..len + 1 {
        stdout.write(b" ").try(&mut stderr);
    }
    stdout.write(b"]").try(&mut stderr);

    stdout.write(b"\r[").try(&mut stderr);

    // As time goes, update the progress bar.
    for _ in 0..len {
        stdout.write(b"#").try(&mut stderr);
        stdout.flush().try(&mut stderr);
        // Sleep.
        sleep(Duration::from_millis(ms / len));
    }

    // Show the cursor again.
    stdout.write(b"\x1b[?25h").try(&mut stderr);
    stdout.write(b"\n").try(&mut stderr);
}
