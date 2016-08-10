#![deny(warnings)]

extern crate extra;

use std::env;
use std::io::{stdout, stderr, Write};
use std::process::{Command, exit};

use extra::option::OptionalExt;

const MAN_PAGE: &'static str = /* @MANSTART{dmesg} */ r#"
NAME
    dmesg - display the system message buffer

SYNOPSIS
    dmesg [ -h | --help]

DESCRIPTION
    Displays the contents of the system message buffer.

OPTIONS
    -h
    --help
        display this help and exit
"#; /* @MANEND */

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();

    for arg in env::args().skip(1){
        if arg.as_str() == "-h" || arg.as_str() == "--help" {
            stdout.write(MAN_PAGE.as_bytes()).try(&mut stderr);
            stdout.flush().try(&mut stderr);
            exit(0);
        }
    }

    Command::new("less").arg("sys:/log").spawn().unwrap().wait().unwrap();
}
