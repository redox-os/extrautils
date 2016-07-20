#![deny(warnings)]

// TODO support reading from standard input

extern crate extra;
extern crate termion;

use std::env::args;
use std::fs::File;
use std::io::{self, Write, Read, StdoutLock, Stderr};
use std::path::{Path, PathBuf};
use std::str::Chars;

use extra::option::OptionalExt;

use termion::{terminal_size, TermRead, TermWrite, IntoRawMode, Color, Key, RawTerminal, Style};

static LONG_HELP: &'static str = /* @MANSTART{mdless} */ r#"
NAME
    mdless- view a markdown file.

SYNOPSIS
    mdless [-h | --help] [input]

DESCRIPTION
    This utility views md files. If no input file is specified as an argument, standard input is
    used.

OPTIONS
    --help, -h
        Print this manual page.

AUTHOR
    This program was written by MovingtoMars and Jeremy Soller for Redox OS. Bugs, issues, or feature requests should
    be reported in the Github repository, 'redox-os/extrautils'.

COPYRIGHT
    Copyright (c) 2016 MovingtoMars, Jeremy Soller

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

fn main() {
    let mut args = args().skip(1);
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
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
                run(filename, &mut file, &mut stdin, &mut stdout, &mut stderr);
            }
        }

        if let Some(x) = args.next() {
            let mut file = File::open(Path::new(x.as_str())).try(&mut stderr);
            run(x.as_str(), &mut file, &mut stdin, &mut stdout, &mut stderr);
        }
    } else {
        writeln!(stderr, "Readin from stdin is not yet supported").try(&mut stderr);
        //run(&mut stdin, &mut stdin, &mut stdout, &mut stderr);
    };
}

//TODO: String in Text
enum Block {
    Text(char),
    Bold(Vec<Block>),
    Italic(Vec<Block>),
    Code(Vec<Block>),
    Link(Vec<Block>, String),
}

impl Block {
    fn parse_bold(s: &mut Chars) -> Block {
        let mut blocks = Vec::new();

        while let Some(c) = s.next() {
            if c == '*' && s.as_str().chars().next() == Some('*') {
                let _ = s.next();
                break;
            } else {
                blocks.push(Block::Text(c))
            }
        }

        Block::Bold(blocks)
    }

    fn parse_italic(s: &mut Chars) -> Block {
        let mut blocks = Vec::new();

        while let Some(c) = s.next() {
            if c == '*' {
                break;
            } else {
                blocks.push(Block::Text(c))
            }
        }

        Block::Italic(blocks)
    }

    fn parse_code(s: &mut Chars) -> Block {
        let mut blocks = Vec::new();

        while let Some(c) = s.next() {
            if c == '`' {
                break;
            } else {
                blocks.push(Block::Text(c))
            }
        }

        Block::Code(blocks)
    }

    fn parse_link(s: &mut Chars) -> Block {
        let mut blocks = Vec::new();
        let mut link = String::new();

        while let Some(c) = s.next() {
            match c {
                ']' => break,
                _ => blocks.push(Block::Text(c))
            }
        }

        if s.as_str().chars().next() == Some('(') {
            while let Some(c) = s.next() {
                match c {
                    '(' => (),
                    ')' => break,
                    _ => link.push(c)
                }
            }
        }

        Block::Link(blocks, link)
    }

    fn parse(s: &mut Chars) -> Vec<Block> {
        let mut blocks = Vec::new();

        while let Some(c) = s.next() {
            match c {
                '*' => if s.as_str().chars().next() == Some('*') {
                    let _ = s.next();
                    blocks.push(Block::parse_bold(s));
                } else {
                    blocks.push(Block::parse_italic(s));
                },
                '`' => blocks.push(Block::parse_code(s)),
                '[' => blocks.push(Block::parse_link(s)),
                _ => blocks.push(Block::Text(c))
            }
        }

        blocks
    }

    fn draw(&self, to: &mut RawTerminal<&mut StdoutLock>) -> std::io::Result<usize> {
        let mut res = 0;

        match *self {
            Block::Text(c) => {
                res += try!(to.write(&[c as u8]));
            },
            Block::Bold(ref blocks) => {
                try!(to.style(Style::Bold));
                for block in blocks.iter() {
                    res += try!(block.draw(to));
                }
                try!(to.style(Style::NoBold));
            },
            Block::Italic(ref blocks) => {
                try!(to.style(Style::Italic));
                for block in blocks.iter() {
                    res += try!(block.draw(to));
                }
                try!(to.style(Style::NoItalic));
            },
            Block::Code(ref blocks) => {
                try!(to.style(Style::Invert));
                for block in blocks.iter() {
                    res += try!(block.draw(to));
                }
                try!(to.style(Style::NoInvert));
            },
            Block::Link(ref blocks, ref _link) => {
                try!(to.style(Style::Underline));
                for block in blocks.iter() {
                    res += try!(block.draw(to));
                }
                try!(to.style(Style::NoUnderline));
            },
        }

        Ok(res)
    }

    fn enter(&self, path: &str) -> Option<PathBuf> {
        match *self {
            Block::Text(_c) => {},
            Block::Bold(ref blocks) => {
                for block in blocks.iter() {
                    if let Some(ret) = block.enter(path) {
                        return Some(ret);
                    }
                }
            },
            Block::Italic(ref blocks) => {
                for block in blocks.iter() {
                    if let Some(ret) = block.enter(path) {
                        return Some(ret);
                    }
                }
            },
            Block::Code(ref blocks) => {
                for block in blocks.iter() {
                    if let Some(ret) = block.enter(path) {
                        return Some(ret);
                    }
                }
            },
            Block::Link(ref _blocks, ref link) => {
                let mut ret = PathBuf::from(path);
                ret.pop();
                ret.push(link);
                return Some(ret);
            },
        }

        None
    }
}

struct Buffer {
    lines: Vec<Vec<Block>>,
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
            .map(|x| { Block::parse(&mut x.chars()) })
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
                for block in line[.. w as usize].iter() {
                    bytes_written += try!(block.draw(to));
                }
            } else {
                for block in line.iter() {
                    bytes_written += try!(block.draw(to));
                }
            }

            y += 1;
            try!(to.goto(0, y));
            if y >= h {
                break;
            }
        }

        Ok(bytes_written)
    }

    fn enter(&mut self, path: &str, w: u16, h: u16) -> Option<PathBuf> {
        let mut y = 0;

        let lines = if self.lines.len() <= h as usize {
            &self.lines
        } else if self.y_off as usize >= self.lines.len() {
            &self.lines[0..0]
        } else {
            &self.lines[self.y_off as usize..]
        };

        for line in lines {
            if line.len() > w as usize {
                for block in line[.. w as usize].iter() {
                    if let Some(ret) = block.enter(path) {
                        return Some(ret);
                    }
                }
            } else {
                for block in line.iter() {
                    if let Some(ret) = block.enter(path) {
                        return Some(ret);
                    }
                }
            }

            y += 1;
            if y >= h {
                break;
            }
        }

        None
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

fn run(path: &str, file: &mut Read, controls: &mut Read, stdout: &mut StdoutLock, stderr: &mut Stderr) {
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
    stdout.write(path.as_bytes()).try(stderr);
    stdout.write(b" Press q to exit.").try(stderr);
    stdout.reset().try(stderr);
    stdout.flush().try(stderr);

    for c in controls.keys() {
        match c.unwrap() {
            Key::Char('q') => {
                stdout.clear().try(stderr);
                stdout.reset().try(stderr);
                stdout.goto(0, 0).try(stderr);
                return
            },
            Key::Char('b') => for _i in 1..h {
                buffer.scroll_up()
            },
            Key::Char(' ') => for _i in 1..h {
                buffer.scroll_down(h)
            },
            Key::Up | Key::Char('k') => buffer.scroll_up(),
            Key::Down | Key::Char('j') => buffer.scroll_down(h),
            Key::Char('\r') | Key::Char('\n') => {
                if let Some(link_path) = buffer.enter(path, w, h - 1) {
                    let mut new_file = File::open(link_path).try(stderr);
                    buffer = Buffer::new();
                    buffer.read(&mut new_file).try(stderr);
                }
            }
            _ => {},
        }

        stdout.clear().try(stderr);
        stdout.reset().try(stderr);
        buffer.draw(&mut stdout, w, h - 1).try(stderr);
        stdout.goto(0, h - 1).try(stderr);
        stdout.bg_color(Color::White).try(stderr);
        stdout.color(Color::Black).try(stderr);
        stdout.write(path.as_bytes()).try(stderr);
        stdout.write(b" Press q to exit.").try(stderr);
        stdout.reset().try(stderr);
        stdout.flush().try(stderr);
    }
}
