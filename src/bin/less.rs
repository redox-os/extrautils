#![deny(warnings)]

// TODO support reading from standard input

extern crate coreutils;
extern crate termion;

use std::env::args;
use std::fs::File;
use std::io::{self, Write, Read, StdoutLock, Stderr};
use std::path::Path;

use coreutils::extra::OptionalExt;

use termion::{terminal_size, TermRead, TermWrite, IntoRawMode, Color, Key, RawTerminal};

static LONG_HELP: &'static str = r#"
    NAME
        less - view a text file.
    SYNOPSIS
        less [-h | --help] [input]
    DESCRIPTION
        This utility views text files. If no input file is specified as an argument, standard input is used.
    OPTIONS
        --help, -h
            Print this manual page.
    AUTHOR
        This program was written by MovingtoMars for Redox OS. Bugs, issues, or feature requests should be reported in the Github repository, 'redox-os/extrautils'.
    COPYRIGHT
        Copyright (c) 2016 MovingtoMars

        Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

        The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

        THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
"#;

fn main() {
    let mut args = args().skip(1);
    let mut stdout = io::stdout().lock();
    let mut stdin = io::stdin().lock();
    let mut stderr = io::stderr();

    if let Some(x) = args.next() {
        match x.as_str() {
            "--help" | "-h" => {
                // Print help.
                stdout.write(LONG_HELP.as_bytes()).try(&mut stderr);
                return;
            },
            filename => {
                let mut file = File::open(Path::new(filename)).try(&mut stderr);
                run(&mut file, &mut stdin, &mut stdout, &mut stderr);
            }
        }

        if let Some(x) = args.next() {
            let mut file = File::open(Path::new(x.as_str())).try(&mut stderr);
            run(&mut file, &mut stdin, &mut stdout, &mut stderr);
        }
    } else {
        writeln!(stderr, "Readin from stdin is not yet supported").try(&mut stderr);
        //run(&mut stdin, &mut stdin, &mut stdout, &mut stderr);
    };
}

struct Buffer {
    lines: Vec<String>,
    y_off: u16,
}

impl Buffer {
    fn new() -> Buffer {
        Buffer{
            lines: Vec::new(),
            y_off: 0,
        }
    }

    fn read(&mut self, from: &mut Read) -> std::io::Result<usize> {
        let mut tmp = String::new();
        let res = from.read_to_string(&mut tmp);
        try!(res);

        self.lines.append(&mut tmp
            .as_str()
            .split("\n")
            .map(|x| {String::from(x)})
            .collect());

        return res;
    }

    fn draw(&self, to: &mut RawTerminal<&mut StdoutLock>, w: u16, h: u16) -> std::io::Result<usize> {
        try!(to.goto(0, 0));

        let mut y = 0;
        let mut bytes_written = 0;

        let lines = if self.lines.len() <= h as usize {
            &self.lines
        } else if self.y_off as usize >= self.lines.len() {
            &self.lines[0..0]
        } else {
            &self.lines[self.y_off as usize..]
        };

        for line in lines {
            if line.len() > w as usize {
                bytes_written += try!(to.write(line[..w as usize].as_bytes()));
            } else {
                bytes_written += try!(to.write(line.as_bytes()));
            }

            y += 1;
            try!(to.goto(0, y));
            if y >= h {
                break;
            }
        }

        Ok(bytes_written)
    }

    fn scroll_up(&mut self) {
        if self.y_off > 0 {
            self.y_off -= 1;
        }
    }

    fn scroll_down(&mut self, height: u16) {
        if ((self.y_off + height) as usize) <= self.lines.len() {
            self.y_off += 1;
        }
    }
}

fn run(file: &mut Read, controls: &mut Read, stdout: &mut StdoutLock, stderr: &mut Stderr) {
    let mut stdout = stdout.into_raw_mode().try(stderr);

    let (w, h) = {
        let (w, h) = terminal_size().try(stderr);
        (w as u16, h as u16)
    };

    stdout.clear().try(stderr);
    stdout.reset().try(stderr);

    let mut buffer = Buffer::new();
    buffer.read(file).try(stderr);

    buffer.draw(&mut stdout, w, h - 1).try(stderr);
    stdout.goto(0, h - 1).try(stderr);
    stdout.bg_color(Color::White).try(stderr);
    stdout.color(Color::Black).try(stderr);
    stdout.write(b"Press q to exit.").try(stderr);
    stdout.reset().try(stderr);
    stdout.flush().try(stderr);

    for c in controls.keys() {
        match c {
            Key::Char('q') => {
                stdout.clear().try(stderr);
                stdout.reset().try(stderr);
                stdout.goto(0, 0).try(stderr);
                return
            },
            Key::Up => buffer.scroll_up(),
            Key::Down => buffer.scroll_down(h),
            _ => {},
        }

        stdout.clear().try(stderr);
        stdout.reset().try(stderr);
        buffer.draw(&mut stdout, w, h - 1).try(stderr);
        stdout.goto(0, h - 1).try(stderr);
        stdout.bg_color(Color::White).try(stderr);
        stdout.color(Color::Black).try(stderr);
        stdout.write(b"Press q to exit.").try(stderr);
        stdout.reset().try(stderr);
        stdout.flush().try(stderr);
    }
}
