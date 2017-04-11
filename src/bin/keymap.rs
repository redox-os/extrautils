extern crate extra;

use std::{io, str, env};
use std::io::Write;
use std::fs::File;
use std::process::exit;

use extra::option::OptionalExt;
use extra::io::WriteExt;

static MAN_PAGE: &'static str = /* @MANSTART{grep} */ r#"
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
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut stderr = io::stderr();

    let arg  = match args.next() {
        Some(arg) => arg,
        None => {
            // TODO Write explanation
            println!("Needs one argument.");
            exit(1);
        }
    };
    let path = if arg.starts_with("-") {
        match arg.as_str() {
            "-h" | "--help" => {
                stdout.writeln(MAN_PAGE.as_bytes()).try(&mut stderr);
            },
            "-l" | "--list" => {
                // TODO list keymaps
            },
            _ => {
                stderr.write(b"Unknown option: ").try(&mut stderr);
                stderr.write(arg.as_bytes()).try(&mut stderr);
                stderr.write(b"\n").try(&mut stderr);
                let _ = stderr.flush();
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
            println!("keymap: failed to open display: {}", err);
            exit(1);
        }
    }
}
