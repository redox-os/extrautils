extern crate extra;

use std::env;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::exit;

static MAN_PAGE: &str = /* @MANSTART{grep} */ r#"
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
        Flags {
            line_numbers: false,
            invert_match: false,
            count: false,
        }
    }
}

fn main() {
    let args = env::args().skip(1);
    let stdin = io::stdin();
    let stdin = stdin.lock();

    let mut flags = Flags::new();
    let mut pattern = String::new();
    let mut files = Vec::with_capacity(args.len());
    for arg in args {
        if arg.starts_with('-') {
            match arg.as_str() {
                "-h" | "--help" => {
                    print!("{}", MAN_PAGE);
                    exit(0);
                }
                "-n" | "--line-number" => flags.line_numbers = true,
                "-v" | "--invert-match" => flags.invert_match = true,
                "-c" | "--count" => flags.count = true,
                _ => {
                    eprintln!("Error, unknown option: {}", arg);
                    exit(2);
                }
            }
        } else if pattern.is_empty() {
            pattern = arg.clone();
        } else {
            files.push(arg)
        }
    }

    if pattern.is_empty() {
        eprintln!("You must provide a pattern");
        exit(2);
    }

    let mut found = false;
    let mut error = false;
    if files.is_empty() {
        found = do_simple_search(BufReader::new(stdin), &pattern, flags);
    } else {
        for path in files {
            match File::open(&Path::new(&path)) {
                Ok(f) => {
                    found |= do_simple_search(BufReader::new(f), &pattern, flags);
                }
                Err(err) => {
                    eprintln!("Error opening {}: {}", path, err);
                    error = true;
                }
            }
        }
    }
    if error {
        exit(2);
    }
    if !found {
        exit(1);
    }
}

fn do_simple_search<T: BufRead>(reader: T, pattern: &str, flags: Flags) -> bool {
    let mut count = 0;
    for (line_num, result) in reader.lines().enumerate() {
        if let Ok(line) = result {
            let mut is_match = line.contains(pattern);
            if flags.invert_match {
                is_match = !is_match
            }
            if is_match {
                count += 1;
                if !flags.count {
                    if flags.line_numbers {
                        print!("{}: ", line_num + 1);
                    }
                    println!("{}", line);
                }
            }
        }
    }

    if flags.count {
        println!("{}", count);
    }

    return count > 0;
}
