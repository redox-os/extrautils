#![deny(warnings)]
#![feature(question_mark)]

// TODO support reading from standard input

extern crate extra;
extern crate termion;

use std::env::args;
use std::fs::File;
use std::io::{self, Write, Read, StdoutLock};
use std::path::Path;

use extra::option::OptionalExt;

use termion::{clear, cursor, style, terminal_size};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

static LONG_HELP: &'static str = /* @MANSTART{less} */ r#"
NAME
    less - view a text file.

SYNOPSIS
    less [-h | --help] [input]

DESCRIPTION
    This utility views text files. If no input file is specified as an argument, standard input is
    used.

OPTIONS
    --help, -h
        Print this manual page.

AUTHOR
    This program was written by MovingtoMars for Redox OS. Bugs, issues, or feature requests should
    be reported in the Github repository, 'redox-os/extrautils'.

COPYRIGHT
    Copyright (c) 2016 MovingtoMars

    Permission is hereby granted, free of charge, to any person obtaining a copy of this software
    and associated documentation files (the "Software"), to deal in the Software without
    restriction, including without limitation the rights to use, copy, modify, merge, publish,
    distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the
    Software is furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all copies or
    substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING
    BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
    NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
    DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
"#; /* @MANEND */

#[cfg(target_os = "redox")]
fn terminal_path() -> String {
    use std::env;
    env::var("TTY").unwrap()
}

#[cfg(not(target_os = "redox"))]
fn terminal_path() -> String {
    "/dev/tty".to_string()
}

fn main() {
    let mut args = args().skip(1).peekable();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut stderr = io::stderr();

    if let Some(x) = args.peek() {
        if x == "--help" || x == "-h" {
            // Print help.
            stdout.write(LONG_HELP.as_bytes()).try(&mut stderr);
            return;
        }
    } else {
        let mut terminal = File::open(terminal_path()).try(&mut stderr);
        run("-", &mut stdin, &mut terminal, &mut stdout).try(&mut stderr);
    };

    while let Some(filename) = args.next() {
        let mut file = File::open(Path::new(filename.as_str())).try(&mut stderr);
        run(filename.as_str(), &mut file, &mut stdin, &mut stdout).try(&mut stderr);
    }
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
        let res = from.read_to_string(&mut tmp)?;

        self.lines.append(&mut tmp
            .as_str()
            .split("\n")
            .map(|x| {String::from(x)})
            .collect());

        Ok(res)
    }

    fn draw(&self, to: &mut RawTerminal<&mut StdoutLock>, w: u16, h: u16) -> std::io::Result<usize> {
        write!(to, "{}{}{}", clear::All, style::Reset, cursor::Goto(1, 1))?;

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
                bytes_written += to.write(line[..w as usize].as_bytes())?;
            } else {
                bytes_written += to.write(line.as_bytes())?;
            }

            y += 1;
            write!(to, "{}", cursor::Goto(1, y + 1))?;
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

fn run(path: &str, file: &mut Read, controls: &mut Read, stdout: &mut StdoutLock) -> std::io::Result<()> {
    let mut stdout = stdout.into_raw_mode()?;

    let (w, h) = {
        let (w, h) = terminal_size()?;
        (w as u16, h as u16)
    };

    let mut buffer = Buffer::new();
    buffer.read(file)?;

    buffer.draw(&mut stdout, w, h - 1)?;

    write!(stdout, "{}{}{} Press q to exit.{}", cursor::Goto(1, h), style::Invert, path, style::NoInvert)?;

    stdout.flush()?;

    for c in controls.keys() {
        match c.unwrap() {
            Key::Char('q') => {
                write!(stdout, "{}{}{}", clear::All, style::Reset, cursor::Goto(1, 1))?;
                break;
            },
            Key::Char('b') | Key::PageUp => for _i in 1..h {
                buffer.scroll_up()
            },
            Key::Char(' ') | Key::PageDown => for _i in 1..h {
                buffer.scroll_down(h)
            },
            Key::Char('u') => for _i in 1..h/2 {
                buffer.scroll_up()
            },
            Key::Char('d') => for _i in 1..h/2 {
                buffer.scroll_down(h)
            },
            Key::Up | Key::Char('k') => buffer.scroll_up(),
            Key::Down | Key::Char('j') => buffer.scroll_down(h),
            _ => {},
        }

        buffer.draw(&mut stdout, w, h - 1)?;

        write!(stdout, "{}{}{} Press q to exit.{}", cursor::Goto(1, h), style::Invert, path, style::NoInvert)?;

        stdout.flush()?;
    }

    Ok(())
}
