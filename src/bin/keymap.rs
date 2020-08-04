extern crate extra;

use std::{io, str, env};
use std::io::Write;
use std::fs::File;
use std::process::exit;

use extra::option::OptionalExt;
use extra::io::WriteExt;

static MAN_PAGE: &str = /* @MANSTART{keymap} */ r#"
NAME
    keymap - change the keymap

SYNOPSIS
    keymap [-h | --help] [-l --list] NAME

DESCRIPTION
    Changes the keymap.
OPTIONS
    -h
    --help
        Print this manual page.

    -l
    --list
        List available keymaps.
"#; /* @MANEND */


fn main() {
    let mut args = env::args().skip(1);

    let arg  = match args.next() {
        Some(arg) => arg,
        None => {
            eprintln!("Must specify keymap name.");
            exit(1);
        }
    };
    let path = if arg.starts_with("-") {
        match arg.as_str() {
            "-h" | "--help" => {
                print!("{}", MAN_PAGE);
            },
            "-l" | "--list" => {
                // TODO list keymaps
            },
            _ => {
                eprintln!("Unknown option: {}", arg);
                exit(1);
            }
        }
        exit(0);
    } else {
        arg
    };

    match File::open("display:keymap") {
        Ok(mut file) => {
            match file.write(path.as_bytes()) {
                Err(e) => println!("keymap: could not change keymap: {}", e),
                _ => {}
            }

        },
        Err(err) => {
            eprintln!("keymap: failed to open display: {}", err);
            exit(1);
        }
    }
}
