#![deny(warnings)]

extern crate coreutils;
use coreutils::extra::OptionalExt;

use std::env::args;
use std::fs::File;
use std::hash::Hasher;
use std::io::{Write, Read, stdin, stdout, stderr};
use std::mem;

static HELP: &'static str = r#"
    NAME
        cksum - calculate the DJB2 checksum of the input.
    SYNOPSIS
        cksum [-h | --help] [-b | --binary] [FILE 1] [FILE 2]...
    DESCRIPTION
        This utility is used for calculating the checksum from one or more byte streams (files and/or standard input). The ordering of the arguments do matter.

        'cksum' differs from the Unix version in multiple ways. Most importantly: it uses DJB2, a streaming non-cryptographic hash function.

        'cksum' defaults to hexadecimal.

        NOTA BENE: This tool should **never** be used as a secure hash function, since DJB2 is non-cryptographic, and easily breakable.
    OPTIONS
        -h
        --help
            Print this manual page.
        -b
        --binary
            Print the output in base 256.
    AUTHOR
        This program was written by Ticki for Redox OS. Bugs, issues, or feature requests should be reported in the Github repository, 'redox-os/extrautils'.
    COPYRIGHT
        Copyright (c) 2016 Ticki

        Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

        The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

        THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
"#;

pub struct Djb2 {
    state: u64,
}

impl Default for Djb2 {
    fn default() -> Djb2 {
        Djb2 {
            state: 5381,
        }
    }
}

impl Hasher for Djb2 {
    fn finish(&self) -> u64 {
        self.state
    }

    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.state = (self.state << 5).wrapping_add(self.state).wrapping_add(b as u64);
        }
    }
}

/// Convert hex to ascii
#[inline]
pub fn hex_to_ascii(b: u8) -> u8 {
    match b {
        0...9 => b'0' + b,
        _ => b'a' - 10 + b,
    }
}

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let mut stderr = stderr();

    let mut buf = Vec::new();
    let mut binary_mode = false;
    let args = args();

    for i in args.skip(1) {
        match i.as_str() {
            "-h" | "--help" => {
                stdout.write(HELP.as_bytes()).try(&mut stderr);
            }
            "-b" | "--binary" => binary_mode = true,
            "-" => {
                stdin().read_to_end(&mut buf).try(&mut stderr);
            }
            file => {
                File::open(file).try(&mut stderr).read_to_end(&mut buf).try(&mut stderr);
            },
        }
    }

    let mut hasher: Djb2 = Default::default();
    hasher.write(&buf);
    let hash = unsafe { mem::transmute::<u64, [u8; 8]>(hasher.finish()) };

    if binary_mode {
        stdout.write(&hash).try(&mut stderr);
    } else {
        for i in hash.iter() {
            stdout.write(&[hex_to_ascii(i & 0b1111), hex_to_ascii(i >> 4)]).try(&mut stderr);
        }
    }

    stdout.write(b"\n").try(&mut stderr);
}
