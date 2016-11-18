#![deny(warnings)]

extern crate extra;
use extra::option::OptionalExt;

use std::env::args;
use std::fs::File;
use std::io::{Write, Read, stdin, stdout, stderr};

static HELP: &'static str = /* @MANSTART{hexdump} */ r#"
NAME
    hexdump - hexadecimal dump

SYNOPSIS
    hexdump [-h | --help] [FILE 1] [FILE 2]...

DESCRIPTION
    hexdump outputs files in hexadecimal with byte counts

OPTIONS
    -h
    --help
        Print this manual page.
"#; /* @MANEND */

/// Convert hex to ascii
#[inline]
pub fn hex_to_ascii(b: u8) -> u8 {
    match b {
        0...9 => b'0' + b,
        _ => b'a' - 10 + b,
    }
}

fn main() {
    let args = args();
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();

    // The buffer. Bytes will be read to this, and afterwards chehexdumpmed.
    let mut buf = Vec::new();

    let mut file_given = false;
    for i in args.skip(1) {
        match i.as_str() {
            // Print the help page.
            "-h" | "--help" => {
                stdout.write(HELP.as_bytes()).try(&mut stderr);
            }
            // Read from stdin.
            "-" => {
                stdin().read_to_end(&mut buf).try(&mut stderr);
                file_given = true;
            }
            file => {
                File::open(file).try(&mut stderr).read_to_end(&mut buf).try(&mut stderr);
                file_given = true;
            },
        }
    }

    if ! file_given {
        stdin().read_to_end(&mut buf).try(&mut stderr);
    }

    // Print the hexadecimal hash to stdout.
    for (i, chunk) in buf.chunks(0x10).enumerate() {
        stdout.write(&[
            hex_to_ascii((i >> 24) as u8 & 0xf),
            hex_to_ascii((i >> 20) as u8 & 0xf),
            hex_to_ascii((i >> 16) as u8 & 0xf),
            hex_to_ascii((i >> 12) as u8 & 0xf),
            hex_to_ascii((i >> 8) as u8 & 0xf),
            hex_to_ascii((i >> 4) as u8 & 0xf),
            hex_to_ascii(i as u8 & 0xf),
            b'0',
            b':'
        ]).try(&mut stderr);
        for b in chunk.iter() {
            stdout.write(&[
                b' ',
                hex_to_ascii(b >> 4),
                hex_to_ascii(b & 0xf)
            ]).try(&mut stderr);
        }
        stdout.write(b"\n").try(&mut stderr);
    }

    // Trailing newline.
    stdout.write(b"\n").try(&mut stderr);
}
