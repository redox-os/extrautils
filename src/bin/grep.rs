extern crate extra;

use std::io;
use std::io::{Write, BufRead, BufReader, Stderr};
use std::env;
use std::fs::File;
use std::path::Path;
use std::error::Error;
use std::process::exit;

use extra::option::OptionalExt;
use extra::io::WriteExt;

static MAN_PAGE: &'static str = /* @MANSTART{grep} */ r#"
NAME
    grep - print lines matching a pattern

SYNOPSIS
    grep [-h | --help] [-n --line-number] PATTERN [FILE...]

DESCRIPTION
    grep searches the named input FILEs for lines containing a match to the given PATTERN. If no
    files are specified, grep searches the standard input. grep prints the matching lines.

OPTIONS
    -h
    --help
        Print this manual page.

    -v
    --invert-match
        Invert matching.

    -c
    --count
        Print count of matching lines, instead of those lines.

    -n
    --line-number
        Prefix each line of output with the line number of the match.
"#; /* @MANEND */

#[derive(Copy, Clone)]
struct Flags {
    line_numbers: bool,
    invert_match: bool,
    count: bool,
}

impl Flags {
    fn new() -> Flags {
        Flags { line_numbers: false, invert_match: false, count: false }
    }
}

fn main() {
    let args = env::args().skip(1);
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();
    let stdin = io::stdin();
    let stdin = stdin.lock();

    let mut flags = Flags::new();
    let mut pattern = String::new();
    let mut files = Vec::with_capacity(args.len());
    for arg in args {
        if arg.starts_with("-") {
            match arg.as_str() {
                "-h" | "--help" => {
                    stdout.writeln(MAN_PAGE.as_bytes()).try(&mut stderr);
                },
                "-n" | "--line-number" => flags.line_numbers = true,
                "-v" | "--invert-match" => flags.invert_match = true,
                "-c" | "--count" => flags.count = true,
                _ => {
                    stderr.write(b"Unknown option: ").try(&mut stderr);
                    stderr.write(arg.as_bytes()).try(&mut stderr);
                    stderr.write(b"\n").try(&mut stderr);
                    let _ = stderr.flush();
                    exit(1);
                }
            }
        } else if pattern.is_empty() {
            pattern = arg.clone();
        } else {
            match File::open(&Path::new(&arg)) {
                Ok(f) => files.push(f),
                Err(e) => {
                    stderr.write(b"Error opening ").try(&mut stderr);
                    stderr.write(arg.as_bytes()).try(&mut stderr);
                    stderr.write(b": ").try(&mut stderr);
                    stderr.write(e.description().as_bytes()).try(&mut stderr);
                    stderr.write(b"\n").try(&mut stderr);
                    let _ = stderr.flush();
                    exit(1);
                }
            }
        }
    }

    if pattern.is_empty() {
        stderr.write_all(b"You must provide a pattern\n").try(&mut stderr);
        exit(1);
    }

    if files.is_empty() {
        do_simple_search(BufReader::new(stdin), &pattern, &mut stdout, &mut stderr, flags);
    } else {
        for f in files {
            do_simple_search(BufReader::new(f), &pattern, &mut stdout, &mut stderr, flags);
        }
    }
}

fn do_simple_search<T: BufRead, O: Write + WriteExt>(reader: T, pattern: &str, out: &mut O, stderr: &mut Stderr, flags: Flags) {
    let mut line_num = 0;
    let mut count = 0;
    for result in reader.lines() {
        line_num += 1;
        if let Ok(line) = result {
            let is_match = if flags.invert_match {
                !line.contains(pattern)
            } else {
                line.contains(pattern)
            };
            if is_match && flags.count {
                count += 1;
            } else if is_match {
                if flags.line_numbers {
                    out.write_all((line_num.to_string() + ": ").as_bytes()).try(stderr);
                }
                out.writeln(line.as_bytes()).try(stderr);
            }
        }
    }

    if flags.count {
        println!("{}", count);
    }
}
