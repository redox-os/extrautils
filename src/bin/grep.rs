#![deny(warnings)]
extern crate coreutils;

use std::io;
use std::io::{Write, BufRead, BufReader, Stderr};
use std::env;
use std::fs::File;
use std::path::Path;
use std::error::Error;

use coreutils::extra::{OptionalExt, WriteExt, fail};

static MAN_PAGE: &'static str = r#"
    NAME
        grep - print lines matching a pattern
    SYNOPSIS
        wc [-h | --help] [-n --line-number] PATTERN [FILE...]
    DESCRIPTION
        grep searches the named input FILEs for lines containing a match to the given PATTERN. If no files are specified, grep searches the standard input. grep prints the matching lines.

    OPTIONS
        -h
        --help
            Print this manual page.

        -n
        --line-number
            Prefix each line of output with the line number of the match.
"#;

#[derive(Copy, Clone)]
struct Flags {
    line_numbers: bool,
}

impl Flags {
    fn new() -> Flags {
        Flags { line_numbers: false }
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
    let mut files = Vec::with_capacity(args.len()-1);
    for arg in args {
        if arg.starts_with("-") {
            match arg.as_str() {
                "-h" | "--help" => {
                    stdout.writeln(MAN_PAGE.as_bytes()).try(&mut stderr);
                },
                "-n" | "--line-number" => flags.line_numbers = true,
                _ => {
                    fail(&format!("Unkown option: {}", &arg), &mut stderr);
                }
            }
        } else if &pattern == "" {
            pattern = arg.clone();
        } else {
            match File::open(&Path::new(&arg)) {
                Ok(f) => files.push(f),
                Err(e) => {
                    fail(&format!("Error opening {}: {}", &arg, e.description()), &mut stderr);
                }
            }
        }
    }

    if &pattern == "" {
        fail("You must provide a pattern", &mut stderr);
    }

    if files.is_empty() {
        do_simple_search(stdin, &pattern, &mut stdout, &mut stderr, flags);
    } else {
        for f in files {
            do_simple_search(BufReader::new(f), &pattern, &mut stdout, &mut stderr, flags);
        }
    }
}

fn do_simple_search<T: BufRead, O: Write + WriteExt>(reader: T, pattern: &str, out: &mut O, stderr: &mut Stderr, flags: Flags) {
    let mut line_num = 0;
    for result in reader.lines() {
        line_num += 1;
        if let Ok(line) = result {
            if line.contains(pattern) {
                if flags.line_numbers {
                    out.write_all((line_num.to_string() + ": ").as_bytes()).try(stderr);
                }
                out.writeln(line.as_bytes()).try(stderr);
            }
        }
    }
}
