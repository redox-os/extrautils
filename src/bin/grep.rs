extern crate arg_parser;
extern crate extra;

use arg_parser::ArgParser;
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
    grep [--help] [-chHinqv] [-m NUM] PATTERN [FILE...]

DESCRIPTION
    grep searches the named input FILEs for lines containing a match to the given PATTERN. If no
    files are specified, grep searches the standard input. grep prints the matching lines.

OPTIONS
    -c
    --count
        Print count of matching lines, instead of those lines.

    -H
    --with-filename
        Include filename header with each match (default for multiple files).

    -h
    --no-filename
        Never include filename header (default for single files or stdin).

    --help
        Print this manual page.

    -i
    --ignore-case
        Make matching case insensitive.

    -m NUM
    --max-count=NUM
        Stop searching after NUM matches.

    -n
    --line-number
        Prefix each line of output with the line number of the match.

    -q
    --quiet
        Suppress normal output and stop searching as soon as a match is found.

    -v
    --invert-match
        Invert matching.
"#; /* @MANEND */

#[derive(Copy, Clone)]
struct Flags {
    count: bool,
    ignore_case: bool,
    invert_match: bool,
    line_numbers: bool,
    quiet: bool,
    with_filenames: bool,
    without_filenames: bool,
    max_count: Option<u32>,
}

impl Flags {
    fn new() -> Flags {
        Flags {
            count: false,
            ignore_case: false,
            invert_match: false,
            line_numbers: false,
            quiet: false,
            with_filenames: false,
            without_filenames: false,
            max_count: None,
        }
    }
}

fn main() {
    let stdin = io::stdin();
    let stdin = stdin.lock();

    let mut flags = Flags::new();
    let mut parser = ArgParser::new(8)
        .add_flag(&["help"])
        .add_flag(&["c", "count"])
        .add_flag(&["H", "with-filename"])
        .add_flag(&["h", "no-filename"])
        .add_flag(&["i", "ignore-case"])
        .add_flag(&["n", "line-number"])
        .add_flag(&["q", "quiet"])
        .add_flag(&["v", "invert-match"])
        .add_opt("m", "max-count");
    parser.parse(env::args());

    if parser.found("help") {
        print!("{}", MAN_PAGE);
        exit(0);
    }
    flags.count |= parser.found("count");
    flags.with_filenames |= parser.found("with-filename");
    flags.without_filenames |= parser.found("no-filename");
    flags.ignore_case |= parser.found("ignore-case");
    flags.line_numbers |= parser.found("line-number");
    flags.quiet |= parser.found("quiet");
    flags.invert_match |= parser.found("invert-match");

    if let Some(mstr) = parser.get_opt("max-count") {
        flags.max_count = match mstr.parse::<u32>() {
            Ok(0) => exit(1),
            Ok(m) => Some(m),
            Err(e) => {
                eprintln!("Invalid max count {}: {}", mstr, e);
                exit(2);
            }
        };
    }

    if let Err(e) = parser.found_invalid() {
        eprint!("{}", e);
        exit(2);
    }
    if parser.args.is_empty() {
        eprintln!("You must provide a pattern");
        exit(2);
    }

    let mut pattern = parser.args[0].clone();
    let files = &parser.args[1..];

    if !flags.without_filenames && files.len() > 1 {
        flags.with_filenames = true;
    } else if flags.with_filenames && flags.without_filenames {
        // FIXME: Unfortunately, with ArgParser we don't have a way to tell
        // which order flags were received in. If a user has, say, an alias
        // like `grep -H` but then runs `grep -h` at the command line, we see
        // `grep -H -h` but can't distinguish it from `grep -h -H`. The last
        // flag should win, since that's clearly the user's intent.
        eprintln!("WARNING: filename flag overrides not yet supported");
    }
    if flags.ignore_case {
        pattern = pattern.to_lowercase();
    }

    let mut found = false;
    let mut error = false;
    if files.is_empty() {
        found = do_simple_search(BufReader::new(stdin), "(standard input)", &pattern, flags);
    } else {
        for path in files {
            match File::open(&Path::new(&path)) {
                Ok(f) => {
                    found |= do_simple_search(BufReader::new(f), &path, &pattern, flags);
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

fn do_simple_search<T: BufRead>(reader: T, path: &str, pattern: &str, flags: Flags) -> bool {
    let mut count = 0;
    for (line_num, result) in reader.lines().enumerate() {
        if let Ok(line) = result {
            let mut is_match = if flags.ignore_case {
                line.to_lowercase().contains(pattern)
            } else {
                line.contains(pattern)
            };
            if flags.invert_match {
                is_match = !is_match
            }
            if is_match {
                if flags.quiet {
                    return true;
                }
                count += 1;
                if !flags.count {
                    if flags.with_filenames {
                        print!("{}:", path);
                    }
                    if flags.line_numbers {
                        print!("{}:", line_num + 1);
                    }
                    println!("{}", line);
                }
                if let Some(m) = flags.max_count {
                    if count >= m {
                        break;
                    }
                }
            }
        }
    }

    if flags.count && !flags.quiet {
        if flags.with_filenames {
            print!("{}:", path);
        }
        println!("{}", count);
    }

    count > 0
}
